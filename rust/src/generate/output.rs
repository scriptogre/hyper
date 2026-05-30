use crate::ast::Position;

/// Line-level source mapping
#[derive(Debug, Clone)]
pub struct Mapping {
    pub gen_line: usize,
    pub gen_col: usize,
    pub src_line: usize,
    pub src_col: usize,
}

/// Range type for IDE injection
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum RangeType {
    Python,
    Html,
}

/// Range mapping source to compiled positions
#[derive(Debug, Clone, serde::Serialize)]
pub struct Range {
    pub range_type: RangeType,
    pub source_start: usize,
    pub source_end: usize,
    pub compiled_start: usize,
    pub compiled_end: usize,
    /// Whether this range should produce an IDE injection.
    /// Set to false for ranges that don't need language injection (like parameters in frontmatter).
    #[serde(skip)]
    pub needs_injection: bool,
    /// Optional prefix for HTML injections (e.g. `<x` for component attribute fragments
    /// that need a synthetic tag name so the HTML parser can highlight attributes).
    #[serde(skip)]
    #[serde(default)]
    pub html_prefix: Option<String>,
}

impl Range {
    /// Create a new range with default html_prefix (None).
    pub fn new(
        range_type: RangeType,
        source_start: usize,
        source_end: usize,
        compiled_start: usize,
        compiled_end: usize,
        needs_injection: bool,
    ) -> Self {
        Self {
            range_type,
            source_start,
            source_end,
            compiled_start,
            compiled_end,
            needs_injection,
            html_prefix: None,
        }
    }
}

/// Computed injection with prefix/suffix for IDE language injection.
/// JetBrains concatenates: prefix + source_content + suffix for each injection.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Injection {
    #[serde(rename = "type")]
    pub injection_type: String,
    pub start: usize, // source start (UTF-16)
    pub end: usize,   // source end (UTF-16)
    pub prefix: String,
    pub suffix: String,
}

/// Expression brace position in source (UTF-16 offsets)
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExpressionBrace {
    pub open: usize,
    pub close: usize,
}

/// Tag highlight kind for IDE syntax coloring of component/slot tags.
#[derive(Debug, Clone, serde::Serialize)]
pub enum TagHighlightKind {
    #[serde(rename = "tag_punctuation")]
    TagPunctuation,
    #[serde(rename = "component_name")]
    ComponentName,
    #[serde(rename = "slot_keyword")]
    SlotKeyword,
    #[serde(rename = "slot_name")]
    SlotName,
}

/// A highlight range for component/slot tag syntax (UTF-16 offsets).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TagHighlight {
    pub start: usize,
    pub end: usize,
    pub kind: TagHighlightKind,
}

/// Convert byte-offset tag highlights to UTF-16 offsets.
pub fn convert_tag_highlights_to_utf16(
    source: &str,
    byte_highlights: &[(usize, usize, TagHighlightKind)],
) -> Vec<TagHighlight> {
    let byte_to_utf16 = build_byte_to_utf16_map(source);
    byte_highlights
        .iter()
        .map(|(start, end, kind)| TagHighlight {
            start: byte_to_utf16[*start],
            end: byte_to_utf16[*end],
            kind: kind.clone(),
        })
        .collect()
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

/// Validate Python injection ranges by checking that source text matches compiled text.
/// JetBrains inserts SOURCE text at each injection point. If source ≠ compiled,
/// the virtual Python file is malformed (e.g. `render_class(class)` instead of
/// `render_class(class_)`). Drop any mismatched ranges to prevent this.
pub fn validate_python_ranges(source: &str, compiled: &str, ranges: &mut Vec<Range>) {
    ranges.retain(|r| {
        if r.range_type != RangeType::Python || !r.needs_injection {
            return true;
        }
        let source_text = match source.get(r.source_start..r.source_end) {
            Some(s) => s,
            None => return false,
        };
        let compiled_text = substring_utf16(compiled, r.compiled_start, r.compiled_end);
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

/// Compute prefix/suffix injections from ranges + compiled code.
/// JetBrains concatenates: prefix1 + source1 + suffix1 + prefix2 + source2 + suffix2...
/// So we set suffix="" for all but the last injection per type.
///
/// For Python: prefix/suffix come from the compiled code, building a virtual Python file
/// where source expressions are embedded in their compiled context.
///
/// For HTML: prefix/suffix are empty — the virtual HTML file is just the source HTML
/// fragments concatenated. This gives JetBrains enough context for tag completion
/// and attribute suggestions.
///
/// Source positions in ranges are byte offsets. This function converts them to UTF-16
/// code unit offsets for the injection start/end fields, since JetBrains TextRange
/// uses UTF-16 offsets.
pub fn compute_injections(code: &str, source: &str, ranges: &[Range]) -> Vec<Injection> {
    // Build byte-to-UTF-16 mapping for source string
    let byte_to_utf16 = build_byte_to_utf16_map(source);
    let mut injections = Vec::new();

    // Process each type separately (python and html have independent virtual files)
    for range_type in [RangeType::Python, RangeType::Html] {
        let type_str = match range_type {
            RangeType::Python => "python",
            RangeType::Html => "html",
        };

        let mut type_ranges: Vec<_> = ranges
            .iter()
            .filter(|r| r.range_type == range_type && r.needs_injection)
            .collect();
        // Python: sort by COMPILED position since prefix computation walks
        // the compiled code sequentially. HTML: sort by source position.
        match range_type {
            RangeType::Python => type_ranges.sort_by_key(|r| r.compiled_start),
            RangeType::Html => type_ranges.sort_by_key(|r| r.source_start),
        }

        if type_ranges.is_empty() {
            continue;
        }

        match range_type {
            RangeType::Python => {
                // Python: prefix/suffix from compiled code for virtual Python file
                let mut prev_end = 0;
                let range_count = type_ranges.len();

                for (index, range) in type_ranges.iter().enumerate() {
                    let is_last = index == range_count - 1;

                    let prefix = substring_utf16(code, prev_end, range.compiled_start);
                    let suffix = if is_last {
                        substring_utf16_to_end(code, range.compiled_end)
                    } else {
                        String::new()
                    };

                    injections.push(Injection {
                        injection_type: type_str.to_string(),
                        start: byte_to_utf16[range.source_start],
                        end: byte_to_utf16[range.source_end],
                        prefix,
                        suffix,
                    });

                    prev_end = range.compiled_end;
                }
            }
            RangeType::Html => {
                // HTML: prefix is usually empty (source already contains HTML).
                // Component-attribute ranges carry an html_prefix (e.g. "<x") so the
                // virtual HTML fragment has a tag name and JetBrains can parse attrs.
                for range in &type_ranges {
                    injections.push(Injection {
                        injection_type: type_str.to_string(),
                        start: byte_to_utf16[range.source_start],
                        end: byte_to_utf16[range.source_end],
                        prefix: range.html_prefix.as_deref().unwrap_or("").to_string(),
                        suffix: String::new(),
                    });
                }
            }
        }
    }

    injections
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

/// Extract substring from UTF-16 position to end
fn substring_utf16_to_end(s: &str, start: usize) -> String {
    let utf16_units: Vec<u16> = s.encode_utf16().collect();
    if start >= utf16_units.len() {
        return String::new();
    }

    String::from_utf16_lossy(&utf16_units[start..])
}

/// Output buffer that accumulates generated code with mappings.
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
    mappings: Vec<Mapping>,
    ranges: Vec<Range>,
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
            mappings: Vec::new(),
            ranges: Vec::new(),
            skip_remaining: 0,
            dedent_amount: 0,
            dedent_skip_remaining: 0,
        }
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

    /// Add text with source mapping
    pub fn push_mapped(&mut self, text: &str, source_pos: Position) {
        let start_col = self.current_line.len();
        self.current_line.push_str(text);

        self.mappings.push(Mapping {
            gen_line: self.line_number,
            gen_col: start_col,
            src_line: source_pos.line,
            src_col: 0, // TODO: track column in Position
        });
    }

    /// Add a newline
    pub fn newline(&mut self) {
        self.current_line.push('\n');
        self.lines.push(std::mem::take(&mut self.current_line));
        self.line_number += 1;
    }

    /// Add a range mapping
    pub fn add_range(&mut self, range: Range) {
        self.ranges.push(range);
    }

    /// Get current UTF-16 position in output
    pub fn position(&self) -> usize {
        // Build the string so far and count UTF-16 code units
        let prev_lines: String = self.lines.join("");
        let so_far = prev_lines + &self.current_line;
        so_far.encode_utf16().count()
    }

    /// Finish and return the generated code
    pub fn finish(mut self) -> (String, Vec<Mapping>, Vec<Range>) {
        // Push final line if not empty (no trailing newline for last line)
        if !self.current_line.is_empty() {
            self.lines.push(std::mem::take(&mut self.current_line));
        }

        let code = self.lines.join("");
        (code, self.mappings, self.ranges)
    }

    /// Transfer ranges from another Output, adjusting compiled positions by an offset.
    /// The offset is the compiled position where the other output's content starts in this output.
    pub fn transfer_ranges(&mut self, other_ranges: Vec<Range>, compiled_offset: usize) {
        for mut range in other_ranges {
            range.compiled_start += compiled_offset;
            range.compiled_end += compiled_offset;
            self.ranges.push(range);
        }
    }

    /// Get the accumulated ranges (for extracting from a temporary buffer)
    pub fn take_ranges(&mut self) -> Vec<Range> {
        std::mem::take(&mut self.ranges)
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
