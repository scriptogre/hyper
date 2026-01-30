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
}

/// Computed injection with prefix/suffix for IDE language injection.
/// JetBrains concatenates: prefix + source_content + suffix for each injection.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Injection {
    #[serde(rename = "type")]
    pub injection_type: String,
    pub start: usize,      // source start (UTF-16)
    pub end: usize,        // source end (UTF-16)
    pub prefix: String,
    pub suffix: String,
}

/// Compute prefix/suffix injections from ranges + compiled code.
/// JetBrains concatenates: prefix1 + source1 + suffix1 + prefix2 + source2 + suffix2...
/// So we set suffix="" for all but the last injection per type.
pub fn compute_injections(code: &str, ranges: &[Range]) -> Vec<Injection> {
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
        // Sort by SOURCE position since we're creating injections for the source file
        type_ranges.sort_by_key(|r| r.source_start);

        if type_ranges.is_empty() {
            continue;
        }

        let mut prev_end = 0;
        let range_count = type_ranges.len();

        for (index, range) in type_ranges.iter().enumerate() {
            let is_last = index == range_count - 1;

            // prefix = code from previous end to current compiled_start
            let prefix = substring_utf16(code, prev_end, range.compiled_start);

            // suffix = "" for all but last, code from last compiled_end to end for last
            let suffix = if is_last {
                substring_utf16_to_end(code, range.compiled_end)
            } else {
                String::new()
            };

            injections.push(Injection {
                injection_type: type_str.to_string(),
                start: range.source_start,
                end: range.source_end,
                prefix,
                suffix,
            });

            prev_end = range.compiled_end;
        }
    }

    injections
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

/// Output buffer that accumulates generated code with mappings
pub struct Output {
    lines: Vec<String>,
    current_line: String,
    line_number: usize,
    mappings: Vec<Mapping>,
    ranges: Vec<Range>,
}

impl Output {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            current_line: String::new(),
            line_number: 0,
            mappings: Vec::new(),
            ranges: Vec::new(),
        }
    }

    /// Add text without mapping
    pub fn push(&mut self, text: &str) {
        self.current_line.push_str(text);
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
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}
