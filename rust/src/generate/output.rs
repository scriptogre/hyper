/// Injection language for IDE language injection
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Html,
}

/// Source-to-compiled span. Source offsets are UTF-16 (after
/// `segments_source_to_utf16` runs); compiled offsets are UTF-16.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Segment {
    pub language: Language,
    pub source_start: usize,
    pub source_end: usize,
    pub compiled_start: usize,
    pub compiled_end: usize,
    /// Whether this segment should produce an IDE injection.
    /// False for highlight-only segments (e.g. a component's closing-tag name).
    pub needs_injection: bool,
    /// Optional prefix for HTML injections (e.g. `<x` for component attribute fragments
    /// that need a synthetic tag name so the HTML parser can highlight attributes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_prefix: Option<String>,
}

/// Expression brace position in source (UTF-16 offsets)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExpressionBrace {
    pub open: usize,
    pub close: usize,
}

/// Convert byte offset pairs to UTF-16 offsets for expression braces.
pub fn convert_braces_to_utf16(
    source: &str,
    byte_braces: &[(usize, usize)],
) -> Vec<ExpressionBrace> {
    let byte_to_utf16 = build_byte_to_utf16_map(source);
    byte_braces
        .iter()
        .map(|(open, close)| ExpressionBrace {
            open: byte_to_utf16[*open],
            close: byte_to_utf16[*close],
        })
        .collect()
}

/// Validate Python injection segments by checking that source text matches compiled text.
/// JetBrains inserts SOURCE text at each injection point. If source ≠ compiled,
/// the virtual Python file is malformed (e.g. `render_class(class)` instead of
/// `render_class(class_)`). Drop any mismatched segments to prevent this.
pub fn validate_python_segments(source: &str, compiled: &str, segments: &mut Vec<Segment>) {
    segments.retain(|s| {
        if s.language != Language::Python || !s.needs_injection {
            return true;
        }
        let source_text = match source.get(s.source_start..s.source_end) {
            Some(t) => t,
            None => return false,
        };
        let compiled_text = substring_utf16(compiled, s.compiled_start, s.compiled_end);
        // Normalize: strip leading whitespace from each line for comparison.
        // This allows multiline statements where only indentation differs.
        normalize_indent(source_text) == normalize_indent(&compiled_text)
    });
}

fn normalize_indent(s: &str) -> String {
    s.lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Convert each segment's `source_start`/`source_end` from byte to UTF-16 offsets.
/// Compiled offsets are left untouched (already UTF-16).
pub fn segments_source_to_utf16(source: &str, segments: &mut [Segment]) {
    let byte_to_utf16 = build_byte_to_utf16_map(source);
    for seg in segments {
        seg.source_start = byte_to_utf16[seg.source_start];
        seg.source_end = byte_to_utf16[seg.source_end];
    }
}

/// Build a mapping from byte offset → UTF-16 code unit offset for a string.
/// The returned Vec has len = s.len() + 1 (to handle end-of-string positions).
fn build_byte_to_utf16_map(s: &str) -> Vec<usize> {
    let mut map = vec![0usize; s.len() + 1];
    let mut utf16_pos = 0;
    for (byte_pos, ch) in s.char_indices() {
        map[byte_pos] = utf16_pos;
        utf16_pos += ch.len_utf16();
    }
    map[s.len()] = utf16_pos;
    map
}

/// Extract substring by UTF-16 positions
fn substring_utf16(s: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }

    let utf16_units: Vec<u16> = s.encode_utf16().collect();
    let end = end.min(utf16_units.len());
    let start = start.min(end);

    String::from_utf16_lossy(&utf16_units[start..end])
}

/// Output buffer that accumulates generated code with segments.
///
/// Supports formatting-aware position tracking via `skip_next()` and
/// `begin_dedent()` / `end_dedent()`. When active, `push()` discards
/// characters (leading whitespace or anchor-indent spaces) so that
/// `position()` always returns the correct compiled offset — no
/// post-hoc range patching needed.
pub struct Output {
    lines: Vec<String>,
    current_line: String,
    line_number: usize,
    segments: Vec<Segment>,
    // Runtime helpers emitted so far; drives the `from hyper import ...` line.
    helpers: std::collections::BTreeSet<String>,
    // Formatting-aware position tracking
    skip_remaining: usize, // characters left to skip (for leading whitespace)
    dedent_amount: usize,  // spaces to strip at each line start (0 = inactive)
    dedent_skip_remaining: usize, // spaces left to strip on current content line
}

impl Output {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_line: String::new(),
            line_number: 0,
            segments: Vec::new(),
            helpers: std::collections::BTreeSet::new(),
            skip_remaining: 0,
            dedent_amount: 0,
            dedent_skip_remaining: 0,
        }
    }

    /// Record a runtime helper as used (e.g. `escape`, `render_class`, `safe`).
    pub fn use_helper(&mut self, name: &str) {
        self.helpers.insert(name.to_string());
    }

    pub fn helper_used(&self, name: &str) -> bool {
        self.helpers.contains(name)
    }

    /// Add text without mapping.
    ///
    /// When skip or dedent mode is active, characters are selectively
    /// discarded so that `position()` reflects the actual output.
    pub fn push(&mut self, text: &str) {
        // Fast path: no formatting active
        if self.skip_remaining == 0 && self.dedent_amount == 0 {
            self.current_line.push_str(text);
            return;
        }

        for ch in text.chars() {
            // Skip mode: discard characters entirely
            if self.skip_remaining > 0 {
                self.skip_remaining -= 1;
                continue;
            }

            // Dedent mode: skip spaces at content line starts
            if self.dedent_skip_remaining > 0 && ch == ' ' {
                self.dedent_skip_remaining -= 1;
                continue;
            }

            // Newline resets the dedent counter for the next line
            if ch == '\n' && self.dedent_amount > 0 {
                self.dedent_skip_remaining = self.dedent_amount;
            } else {
                // Non-space char stops dedent skipping for this line
                self.dedent_skip_remaining = 0;
            }

            self.current_line.push(ch);
        }
    }

    /// Add a newline
    pub fn newline(&mut self) {
        self.current_line.push('\n');
        self.lines.push(std::mem::take(&mut self.current_line));
        self.line_number += 1;
    }

    /// Add a segment
    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Get current UTF-16 position in output
    pub fn position(&self) -> usize {
        // Build the string so far and count UTF-16 code units
        let prev_lines: String = self.lines.join("");
        let so_far = prev_lines + &self.current_line;
        so_far.encode_utf16().count()
    }

    /// Finish and return the generated code
    pub fn finish(mut self) -> (String, Vec<Segment>) {
        // Push final line if not empty (no trailing newline for last line)
        if !self.current_line.is_empty() {
            self.lines.push(std::mem::take(&mut self.current_line));
        }

        let code = self.lines.join("");
        (code, self.segments)
    }

    /// Transfer segments from another Output, adjusting compiled positions by an offset.
    /// The offset is the compiled position where the other output's content starts in this output.
    pub fn transfer_segments(&mut self, other_segments: Vec<Segment>, compiled_offset: usize) {
        for mut seg in other_segments {
            seg.compiled_start += compiled_offset;
            seg.compiled_end += compiled_offset;
            self.segments.push(seg);
        }
    }

    /// Get the accumulated segments (for extracting from a temporary buffer)
    pub fn take_segments(&mut self) -> Vec<Segment> {
        std::mem::take(&mut self.segments)
    }

    /// Skip the next `n` characters pushed to this buffer.
    /// They won't appear in output or affect `position()`.
    /// Used to discard leading whitespace from combined content blocks.
    pub fn skip_next(&mut self, n: usize) {
        self.skip_remaining = n;
    }

    /// Begin dedent mode: at each content newline, skip up to `n` leading spaces.
    /// Does not affect the first line (only lines after a `\n` within pushed text).
    pub fn begin_dedent(&mut self, n: usize) {
        self.dedent_amount = n;
        self.dedent_skip_remaining = 0; // first line is not dedented
    }

    /// End dedent mode.
    pub fn end_dedent(&mut self) {
        self.dedent_amount = 0;
        self.dedent_skip_remaining = 0;
    }

    /// Remove trailing whitespace (spaces, tabs, newlines) from the current
    /// line buffer. Used to clean up trailing content in combined blocks.
    pub fn trim_trailing(&mut self) {
        let trimmed_len = self.current_line.trim_end_matches([' ', '\t', '\n']).len();
        self.current_line.truncate(trimmed_len);
    }

    /// Remove trailing spaces and tabs only (preserve newlines) from the
    /// current line buffer. Used when the content naturally ends with `\n`.
    pub fn trim_trailing_spaces(&mut self) {
        let trimmed_len = self.current_line.trim_end_matches([' ', '\t']).len();
        self.current_line.truncate(trimmed_len);
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}
