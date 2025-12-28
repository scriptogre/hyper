use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use tree_sitter::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    /// HTML tag line (starts with <)
    Html,
    /// Python control flow (if, for, while, etc.)
    Control,
    /// Block terminator (end)
    End,
    /// Python statement detected by tree-sitter
    Python,
    /// Content (text that should be output literally)
    Content,
    /// Explicit t-string escape (t"..." or t"""...""")
    TString,
    /// Comment (# ...)
    Comment,
    /// Empty/whitespace only
    Empty,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub line_type: LineType,
    pub text: String,
    pub line_number: usize,
    pub byte_offset: usize,
    /// For TString lines, the extracted content without t"..." wrapper
    pub tstring_content: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceMapping {
    pub gen_line: usize,
    pub gen_col: usize,
    pub src_line: usize,
    pub src_col: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PythonPiece {
    pub prefix: String,
    pub suffix: String,
    pub src_start: usize,
    pub src_end: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranspileResult {
    pub python_code: String,
    pub source_mappings: Vec<SourceMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_pieces: Option<Vec<PythonPiece>>,
}

lazy_static! {
    static ref HTML_LINE: Regex = Regex::new(r"^[ \t]*<").unwrap();
    static ref END_LINE: Regex = Regex::new(r"^[ \t]*end[ \t]*$").unwrap();
    // Regex for control flow keywords that tree-sitter can't parse standalone
    static ref ELSE_LINE: Regex = Regex::new(r"^[ \t]*else[ \t]*:[ \t]*$").unwrap();
    static ref ELIF_LINE: Regex = Regex::new(r"^[ \t]*elif[ \t]+.+:[ \t]*$").unwrap();
    static ref CASE_LINE: Regex = Regex::new(r"^[ \t]*case[ \t]+.+:[ \t]*$").unwrap();
}

/// Parse a line with tree-sitter and return the AST node kind of the first statement
fn parse_python_line(parser: &mut Parser, text: &str) -> Option<(String, tree_sitter::Tree)> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let tree = parser.parse(trimmed, None)?;
    let root = tree.root_node();

    // If there are errors, it's not valid Python
    if root.has_error() {
        return None;
    }

    if root.kind() != "module" {
        return None;
    }

    // Get the first child (the actual statement)
    let child = root.child(0)?;
    Some((child.kind().to_string(), tree))
}

/// Check if a line is a control flow statement using tree-sitter
fn is_control_flow(parser: &mut Parser, text: &str) -> bool {
    match parse_python_line(parser, text) {
        Some((kind, _)) => matches!(kind.as_str(),
            "if_statement" |
            "for_statement" |
            "while_statement" |
            "match_statement" |
            "with_statement" |
            "try_statement" |
            "function_definition" |
            "class_definition" |
            "elif_clause" |
            "else_clause" |
            "except_clause" |
            "finally_clause" |
            "case_clause"
        ),
        None => false,
    }
}

/// Check if a line is a type annotation (parameter declaration) using tree-sitter
fn is_type_annotation(parser: &mut Parser, text: &str) -> bool {
    let trimmed = text.trim();
    let tree = match parser.parse(trimmed, None) {
        Some(t) => t,
        None => return false,
    };

    let root = tree.root_node();
    if root.has_error() {
        return false;
    }

    // Look for type annotation pattern: identifier ":" type
    // In tree-sitter Python, this is typically a "type" node under expression_statement
    fn find_type_annotation(node: tree_sitter::Node) -> bool {
        if node.kind() == "type" {
            return true;
        }
        // Also check for "annotated_assignment" which is "x: int = value"
        if node.kind() == "annotated_assignment" {
            return true;
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if find_type_annotation(child) {
                    return true;
                }
            }
        }
        false
    }

    find_type_annotation(root)
}

/// Check if a line is a Python statement that executes code
fn is_python_statement(parser: &mut Parser, text: &str) -> bool {
    match parse_python_line(parser, text) {
        Some((kind, tree)) => {
            match kind.as_str() {
                // Statements that definitely execute code
                "expression_statement" => {
                    let root = tree.root_node();
                    if let Some(stmt) = root.child(0) {
                        if let Some(expr) = stmt.child(0) {
                            // Type annotations are parameters, not executable code
                            if expr.kind() == "type" {
                                return false;
                            }
                            return matches!(expr.kind(),
                                "call" |
                                "await" |
                                "assignment" |
                                "augmented_assignment" |
                                "named_expression" |
                                "yield" |
                                "yield from"
                            );
                        }
                    }
                    false
                }
                // Direct statement types that execute code
                "assignment" |
                "augmented_assignment" |
                "import_statement" |
                "import_from_statement" |
                "assert_statement" |
                "pass_statement" |
                "break_statement" |
                "continue_statement" |
                "return_statement" |
                "raise_statement" |
                "delete_statement" |
                "global_statement" |
                "nonlocal_statement" |
                "print_statement" => true,
                // Everything else is not an executable statement
                _ => false,
            }
        }
        None => false,
    }
}

/// Check if a line is a comment
fn is_comment(parser: &mut Parser, text: &str) -> bool {
    match parse_python_line(parser, text) {
        Some((kind, _)) => kind == "comment",
        None => {
            // Fallback: check if line starts with #
            text.trim().starts_with('#')
        }
    }
}

/// Track bracket depth for multi-line statement detection
#[derive(Debug, Default)]
struct BracketState {
    parens: i32,      // ()
    brackets: i32,    // []
    braces: i32,      // {}
    backslash: bool,  // line continuation
}

impl BracketState {
    fn is_continuation(&self) -> bool {
        self.parens > 0 || self.brackets > 0 || self.braces > 0 || self.backslash
    }

    fn update(&mut self, line: &str) {
        self.backslash = line.trim_end().ends_with('\\');

        let mut in_string = false;
        let mut string_char = ' ';
        let mut prev_char = ' ';

        for c in line.chars() {
            // Track string state (simplified - doesn't handle triple quotes)
            if !in_string && (c == '"' || c == '\'') {
                in_string = true;
                string_char = c;
            } else if in_string && c == string_char && prev_char != '\\' {
                in_string = false;
            }

            if !in_string {
                match c {
                    '(' => self.parens += 1,
                    ')' => self.parens = (self.parens - 1).max(0),
                    '[' => self.brackets += 1,
                    ']' => self.brackets = (self.brackets - 1).max(0),
                    '{' => self.braces += 1,
                    '}' => self.braces = (self.braces - 1).max(0),
                    _ => {}
                }
            }
            prev_char = c;
        }
    }

    fn reset_backslash(&mut self) {
        self.backslash = false;
    }
}

/// Extract content from t-string syntax: t"content" or t"""content"""
fn extract_tstring_content(text: &str) -> Option<String> {
    let trimmed = text.trim();

    // Try triple quotes first
    if let Some(rest) = trimmed.strip_prefix("t\"\"\"") {
        if let Some(content) = rest.strip_suffix("\"\"\"") {
            return Some(content.to_string());
        }
    }
    if let Some(rest) = trimmed.strip_prefix("t'''") {
        if let Some(content) = rest.strip_suffix("'''") {
            return Some(content.to_string());
        }
    }
    // Single quotes
    if let Some(rest) = trimmed.strip_prefix("t\"") {
        if let Some(content) = rest.strip_suffix('"') {
            return Some(content.to_string());
        }
    }
    if let Some(rest) = trimmed.strip_prefix("t'") {
        if let Some(content) = rest.strip_suffix('\'') {
            return Some(content.to_string());
        }
    }

    None
}

fn classify_line(parser: &mut Parser, text: &str, in_continuation: bool) -> LineType {
    let trimmed = text.trim();

    // Empty lines
    if trimmed.is_empty() {
        return LineType::Empty;
    }

    // If we're in a multi-line continuation, this is Python
    if in_continuation {
        return LineType::Python;
    }

    // Check for explicit t-string escape first
    if extract_tstring_content(text).is_some() {
        return LineType::TString;
    }

    // End keyword (our special syntax, not Python)
    if END_LINE.is_match(text) {
        return LineType::End;
    }

    // HTML lines (start with <)
    if HTML_LINE.is_match(text) {
        return LineType::Html;
    }

    // Check for else/elif/case which tree-sitter can't parse standalone
    // (they need preceding if/match context)
    if ELSE_LINE.is_match(text) || ELIF_LINE.is_match(text) || CASE_LINE.is_match(text) {
        return LineType::Control;
    }

    // Use tree-sitter for Python classification
    // Check control flow first
    if is_control_flow(parser, text) {
        return LineType::Control;
    }

    // Check comments
    if is_comment(parser, text) {
        return LineType::Comment;
    }

    // Check executable Python statements
    if is_python_statement(parser, trimmed) {
        return LineType::Python;
    }

    // Default: treat as content
    LineType::Content
}

fn lex(source: &str) -> Vec<Line> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("Failed to load Python grammar");

    let mut lines = Vec::new();
    let mut byte_offset = 0;
    let mut bracket_state = BracketState::default();

    for (i, text) in source.lines().enumerate() {
        let in_continuation = bracket_state.is_continuation();
        let line_type = classify_line(&mut parser, text, in_continuation);

        let tstring_content = if line_type == LineType::TString {
            extract_tstring_content(text)
        } else {
            None
        };

        lines.push(Line {
            line_type,
            text: text.to_string(),
            line_number: i,
            byte_offset,
            tstring_content,
        });

        // Update bracket state for multi-line detection
        if matches!(line_type, LineType::Python | LineType::Control) {
            bracket_state.update(text);
        } else {
            bracket_state.reset_backslash();
        }

        byte_offset += text.len() + 1; // +1 for \n
    }

    lines
}

/// Check if a line contains await using tree-sitter
fn line_has_await(parser: &mut Parser, text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }

    let tree = match parser.parse(trimmed, None) {
        Some(t) => t,
        None => return false,
    };

    // Recursively check for await expressions
    fn has_await_node(node: tree_sitter::Node) -> bool {
        if node.kind() == "await" {
            return true;
        }
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if has_await_node(child) {
                    return true;
                }
            }
        }
        false
    }

    has_await_node(tree.root_node())
}

/// Check if a line is an async for/with construct
fn line_is_async_construct(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with("async for ") || trimmed.starts_with("async with ")
}

fn has_await(lines: &[Line]) -> bool {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("Failed to load Python grammar");

    lines.iter().any(|line| {
        matches!(line.line_type, LineType::Python | LineType::Control)
            && line_has_await(&mut parser, &line.text)
    })
}

fn has_async_construct(lines: &[Line]) -> bool {
    lines.iter().any(|line| {
        line.line_type == LineType::Control && line_is_async_construct(&line.text)
    })
}

fn find_structure(lines: &[Line]) -> (Vec<&Line>, Vec<&Line>, usize) {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("Failed to load Python grammar");

    let mut leading = Vec::new();
    let mut params = Vec::new();
    let mut seen_param = false;
    let mut body_start = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.text.trim();

        if trimmed.is_empty() && !seen_param {
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        if !seen_param && trimmed.starts_with('#') {
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        // Use tree-sitter to detect type annotations
        if is_type_annotation(&mut parser, &line.text) {
            seen_param = true;
            params.push(line);
            body_start = i + 1;
            continue;
        }

        break;
    }

    (leading, params, body_start)
}

fn content_bounds(text: &str) -> (usize, usize) {
    let bytes = text.as_bytes();
    let start = bytes.iter().take_while(|&&b| b == b' ' || b == b'\t').count();
    let end = text.trim_end_matches(['\r', '\n']).len();
    (start, end)
}

/// Represents a buffered content chunk with source location info
#[derive(Debug)]
struct ContentChunk {
    lines: Vec<String>,
    src_start: usize,
    src_end: usize,
    first_line_number: usize,
}

impl ContentChunk {
    fn new() -> Self {
        ContentChunk {
            lines: Vec::new(),
            src_start: 0,
            src_end: 0,
            first_line_number: 0,
        }
    }

    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn push(&mut self, line: &str, byte_offset: usize, line_number: usize) {
        if self.lines.is_empty() {
            self.src_start = byte_offset;
            self.first_line_number = line_number;
        }
        self.lines.push(line.to_string());
        self.src_end = byte_offset + line.len();
    }

    fn clear(&mut self) {
        self.lines.clear();
        self.src_start = 0;
        self.src_end = 0;
        self.first_line_number = 0;
    }

    fn content(&self) -> String {
        self.lines.join("\n")
    }
}

pub fn transpile_ext(source: &str, include_injection: bool) -> TranspileResult {
    transpile_named(source, "Template", include_injection)
}

pub fn transpile_named(source: &str, name: &str, include_injection: bool) -> TranspileResult {
    let lines = lex(source);
    let (leading, params, body_start) = find_structure(&lines);
    let is_async = has_await(&lines) || has_async_construct(&lines);

    let mut output = Vec::new();
    let mut mappings = Vec::new();
    let mut python_pieces = if include_injection { Some(Vec::new()) } else { None };
    let mut out_line = 0;

    let indent = "    ";
    let def_kw = if is_async { "async def" } else { "def" };

    // Leading content (comments and empty lines)
    for line in &leading {
        let trimmed = line.text.trim();
        if !trimmed.is_empty() {
            let (start, end) = content_bounds(&line.text);
            output.push(line.text[start..end].to_string());
            mappings.push(SourceMapping {
                gen_line: out_line,
                gen_col: 0,
                src_line: line.line_number,
                src_col: start,
            });
            if let Some(ref mut p) = python_pieces {
                p.push(PythonPiece {
                    prefix: String::new(),
                    suffix: "\n".to_string(),
                    src_start: line.byte_offset + start,
                    src_end: line.byte_offset + end,
                });
            }
        } else {
            output.push(String::new());
            mappings.push(SourceMapping {
                gen_line: out_line,
                gen_col: 0,
                src_line: line.line_number,
                src_col: 0,
            });
            if let Some(ref mut p) = python_pieces {
                p.push(PythonPiece {
                    prefix: String::new(),
                    suffix: "\n".to_string(),
                    src_start: line.byte_offset,
                    src_end: line.byte_offset,
                });
            }
        }
        out_line += 1;
    }

    // Function definition
    if !params.is_empty() {
        let param_strs: Vec<_> = params
            .iter()
            .map(|p| {
                let (s, e) = content_bounds(&p.text);
                p.text[s..e].to_string()
            })
            .collect();

        output.push(format!("{} {}({}):", def_kw, name, param_strs.join(", ")));

        let first = params[0];
        let (start, _) = content_bounds(&first.text);
        mappings.push(SourceMapping {
            gen_line: out_line,
            gen_col: def_kw.len() + 1 + name.len() + 1,
            src_line: first.line_number,
            src_col: start,
        });

        // Add python_pieces for each parameter so IDE can resolve references
        if let Some(ref mut p) = python_pieces {
            let func_prefix = format!("{} {}(", def_kw, name);
            for (i, param) in params.iter().enumerate() {
                let (s, e) = content_bounds(&param.text);
                let prefix = if i == 0 {
                    func_prefix.clone()
                } else {
                    ", ".to_string()
                };
                let suffix = if i == params.len() - 1 {
                    "):\n    _parts = []\n".to_string()
                } else {
                    String::new()
                };
                p.push(PythonPiece {
                    prefix,
                    suffix,
                    src_start: param.byte_offset + s,
                    src_end: param.byte_offset + e,
                });
            }
        }
    } else {
        output.push(format!("{} {}():", def_kw, name));
        mappings.push(SourceMapping {
            gen_line: out_line,
            gen_col: 0,
            src_line: body_start.min(lines.len().saturating_sub(1)),
            src_col: 0,
        });
        // Add empty function definition piece for no-params case
        if let Some(ref mut p) = python_pieces {
            p.push(PythonPiece {
                prefix: format!("{} {}():\n    _parts = []\n", def_kw, name),
                suffix: String::new(),
                src_start: 0,
                src_end: 0,
            });
        }
    }
    out_line += 1;

    // Initialize parts accumulator
    output.push(format!("{}_parts = []", indent));
    out_line += 1;

    // Body - using buffer/flush approach
    let mut level = 1usize;
    let mut stack: Vec<&str> = Vec::new();
    let mut content_buffer = ContentChunk::new();

    // Helper to flush content buffer as _parts.append()
    let flush_buffer = |buffer: &mut ContentChunk,
                        output: &mut Vec<String>,
                        mappings: &mut Vec<SourceMapping>,
                        python_pieces: &mut Option<Vec<PythonPiece>>,
                        out_line: &mut usize,
                        level: usize,
                        indent: &str| {
        if buffer.is_empty() {
            return;
        }

        let content = buffer.content();
        let prefix = format!("{}_parts.append(f\"\"\"", indent.repeat(level));
        let suffix = "\"\"\")";

        output.push(format!("{}{}{}", prefix, content, suffix));

        mappings.push(SourceMapping {
            gen_line: *out_line,
            gen_col: prefix.len(),
            src_line: buffer.first_line_number,
            src_col: 0,
        });

        if let Some(ref mut p) = python_pieces {
            p.push(PythonPiece {
                prefix,
                suffix: format!("{}\n", suffix),
                src_start: buffer.src_start,
                src_end: buffer.src_end,
            });
        }

        *out_line += 1;
        buffer.clear();
    };

    for i in body_start..lines.len() {
        let line = &lines[i];
        let (start, end) = content_bounds(&line.text);

        match line.line_type {
            LineType::Empty => {
                // Empty lines in content buffer stay in buffer
                // Empty lines outside content buffer are preserved
                if content_buffer.is_empty() {
                    output.push(String::new());
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: 0,
                        src_line: line.line_number,
                        src_col: 0,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: String::new(),
                            suffix: "\n".to_string(),
                            src_start: line.byte_offset,
                            src_end: line.byte_offset,
                        });
                    }
                    out_line += 1;
                } else {
                    // Add empty line to buffer
                    content_buffer.push("", line.byte_offset, line.line_number);
                }
            }

            LineType::Control => {
                // Flush buffer before control flow
                flush_buffer(&mut content_buffer, &mut output, &mut mappings, &mut python_pieces, &mut out_line, level, indent);

                let trimmed = &line.text[start..end];
                let is_dedent = ["else", "elif", "except", "finally"]
                    .iter()
                    .any(|kw| trimmed.starts_with(kw));
                let is_case = trimmed.starts_with("case");

                if is_dedent {
                    let print_level = level.saturating_sub(1).max(1);
                    output.push(format!("{}{}", indent.repeat(print_level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * print_level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(print_level),
                            suffix: "\n".to_string(),
                            src_start: line.byte_offset + start,
                            src_end: line.byte_offset + end,
                        });
                    }
                } else if is_case {
                    if stack.last() == Some(&"case") {
                        stack.pop();
                        level = level.saturating_sub(1);
                    }
                    output.push(format!("{}{}", indent.repeat(level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(level),
                            suffix: "\n".to_string(),
                            src_start: line.byte_offset + start,
                            src_end: line.byte_offset + end,
                        });
                    }
                    stack.push("case");
                    level += 1;
                } else {
                    output.push(format!("{}{}", indent.repeat(level), trimmed));
                    mappings.push(SourceMapping {
                        gen_line: out_line,
                        gen_col: indent.len() * level,
                        src_line: line.line_number,
                        src_col: start,
                    });
                    if let Some(ref mut p) = python_pieces {
                        p.push(PythonPiece {
                            prefix: indent.repeat(level),
                            suffix: "\n".to_string(),
                            src_start: line.byte_offset + start,
                            src_end: line.byte_offset + end,
                        });
                    }
                    stack.push(if trimmed.starts_with("match") { "match" } else { "block" });
                    level += 1;
                }
                out_line += 1;
            }

            LineType::End => {
                // Flush buffer before end
                flush_buffer(&mut content_buffer, &mut output, &mut mappings, &mut python_pieces, &mut out_line, level, indent);

                while stack.last() == Some(&"case") {
                    stack.pop();
                    level = level.saturating_sub(1);
                }
                if !stack.is_empty() {
                    stack.pop();
                    level = level.saturating_sub(1);
                }
                level = level.max(1);
                output.push(format!("{}pass", indent.repeat(level)));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: 0,
                    src_line: line.line_number,
                    src_col: 0,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: format!("{}pass\n", indent.repeat(level)),
                        suffix: String::new(),
                        src_start: line.byte_offset,
                        src_end: line.byte_offset,
                    });
                }
                out_line += 1;
            }

            LineType::Html | LineType::Content => {
                // Add to content buffer, preserving the full line (including indentation)
                let (_, end) = content_bounds(&line.text);
                let content = &line.text[..end];
                content_buffer.push(content, line.byte_offset, line.line_number);
            }

            LineType::TString => {
                // Add the extracted content to buffer
                if let Some(ref content) = line.tstring_content {
                    content_buffer.push(content, line.byte_offset, line.line_number);
                }
            }

            LineType::Comment => {
                // Flush buffer before comment
                flush_buffer(&mut content_buffer, &mut output, &mut mappings, &mut python_pieces, &mut out_line, level, indent);

                let trimmed = &line.text[start..end];
                output.push(format!("{}{}", indent.repeat(level), trimmed));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: indent.len() * level,
                    src_line: line.line_number,
                    src_col: start,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: indent.repeat(level),
                        suffix: "\n".to_string(),
                        src_start: line.byte_offset + start,
                        src_end: line.byte_offset + end,
                    });
                }
                out_line += 1;
            }

            LineType::Python => {
                // Flush buffer before Python
                flush_buffer(&mut content_buffer, &mut output, &mut mappings, &mut python_pieces, &mut out_line, level, indent);

                let trimmed = &line.text[start..end];
                output.push(format!("{}{}", indent.repeat(level), trimmed));
                mappings.push(SourceMapping {
                    gen_line: out_line,
                    gen_col: indent.len() * level,
                    src_line: line.line_number,
                    src_col: start,
                });
                if let Some(ref mut p) = python_pieces {
                    p.push(PythonPiece {
                        prefix: indent.repeat(level),
                        suffix: "\n".to_string(),
                        src_start: line.byte_offset + start,
                        src_end: line.byte_offset + end,
                    });
                }
                out_line += 1;
            }
        }
    }

    // Final flush
    flush_buffer(&mut content_buffer, &mut output, &mut mappings, &mut python_pieces, &mut out_line, level, indent);

    // Return joined parts
    output.push(format!("{}return \"\".join(_parts)", indent));

    let mut code = output.join("\n");
    if !code.is_empty() && !code.ends_with('\n') {
        code.push('\n');
    }

    TranspileResult {
        python_code: code,
        source_mappings: mappings,
        python_pieces,
    }
}

pub fn transpile(source: &str) -> TranspileResult {
    transpile_ext(source, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let result = transpile("name: str\n\n<div>Hello {name}</div>\n");
        println!("Generated:\n{}", result.python_code);
        assert!(result.python_code.contains("def Template(name: str):"));
        assert!(result.python_code.contains("_parts = []"));
        assert!(result.python_code.contains("_parts.append(f\"\"\""));
        assert!(result.python_code.contains("{name}"));
        assert!(result.python_code.contains("return \"\".join(_parts)"));
    }

    #[test]
    fn test_async() {
        let result = transpile("id: int\n\ndata = await fetch(id)\n<div>{data}</div>\n");
        println!("Generated:\n{}", result.python_code);
        assert!(result.python_code.contains("async def Template(id: int):"));
    }

    #[test]
    fn test_control_flow() {
        let result = transpile("items: list\n\nfor item in items:\n    <li>{item}</li>\nend\n");
        assert!(result.python_code.contains("for item in items:"));
        assert!(result.python_code.contains("pass"));
    }

    #[test]
    fn test_multiline_html() {
        let source = r#"count: int

if count == 0:
    <span class="empty">
        Empty
    </span>
end
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Should contain a single t-string with multiline content (indentation preserved)
        assert!(result.python_code.contains("<span class=\"empty\">"));
        assert!(result.python_code.contains("Empty"));
        assert!(result.python_code.contains("</span>"));
        // All should be in a single t-string, not separate ones
        let fstring_count = result.python_code.matches("f\"\"\"").count();
        assert_eq!(fstring_count, 1, "Should have exactly one f-string in the if block");
    }

    #[test]
    fn test_python_detection() {
        let source = r#"<div>
    x = 1
    Hello World
    print("hi")
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // x = 1 and print("hi") should be Python, Hello World should be content
        assert!(result.python_code.contains("x = 1"));
        assert!(result.python_code.contains("print(\"hi\")"));
        assert!(result.python_code.contains("Hello World"));
    }

    #[test]
    fn test_tstring_escape() {
        let source = r#"<div>
    t"x = 1"
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // t"x = 1" should become content "x = 1" inside the t-string
        assert!(result.python_code.contains("x = 1"));
        // Should be inside an f-string (only one f-string in output)
        let fstring_count = result.python_code.matches("f\"\"\"").count();
        assert_eq!(fstring_count, 1, "Should have exactly one f-string");
        // The f-string should contain "x = 1" as content
        assert!(result.python_code.contains("f\"\"\"<div>\n    x = 1\n</div>\"\"\"") ||
                result.python_code.contains("f\"\"\"<div>\nx = 1\n</div>\"\"\""));
    }

    #[test]
    fn test_comment_stripped() {
        let source = r#"<div>
    # This is a comment
    Hello
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Comment should be preserved as Python comment, separate from content
        assert!(result.python_code.contains("# This is a comment"));
    }

    #[test]
    fn test_multiline_python_parens() {
        let source = r#"result = (
    1 +
    2 +
    3
)
<div>{result}</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Multi-line Python should be recognized
        assert!(result.python_code.contains("result = ("));
        assert!(result.python_code.contains("1 +"));
        assert!(result.python_code.contains("<div>{result}</div>"));
    }

    #[test]
    fn test_bare_identifier_is_content() {
        let source = r#"<div>
    Hello
    World
    SomeText
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Bare identifiers should be content, not Python
        assert!(result.python_code.contains("Hello"));
        assert!(result.python_code.contains("World"));
        // All in one f-string
        let fstring_count = result.python_code.matches("f\"\"\"").count();
        assert_eq!(fstring_count, 1);
    }

    #[test]
    fn test_function_call_is_python() {
        let source = r#"<div>
    log("debug message")
    Hello
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Function call should be Python (separate from content)
        // Should have multiple t-strings since log() interrupts
        assert!(result.python_code.contains("log(\"debug message\")"));
    }

    #[test]
    fn test_import_is_python() {
        let source = r#"from datetime import datetime
<div>Today is {datetime.now()}</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        assert!(result.python_code.contains("from datetime import datetime"));
    }

    #[test]
    fn test_mixed_content_and_python() {
        let source = r#"<div>
    Welcome
    name = "Guest"
    Hello {name}
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // name = "Guest" should be Python
        assert!(result.python_code.contains("name = \"Guest\""));
        // Welcome and Hello should be content
        assert!(result.python_code.contains("Welcome"));
        assert!(result.python_code.contains("Hello {name}"));
    }

    #[test]
    fn test_capitalized_words_are_content() {
        let source = r#"<div>
    If you see this
    For example
    While waiting
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // Capitalized words should be content (not keywords)
        assert!(result.python_code.contains("If you see this"));
        assert!(result.python_code.contains("For example"));
        // All in one f-string
        let fstring_count = result.python_code.matches("f\"\"\"").count();
        assert_eq!(fstring_count, 1);
    }

    #[test]
    fn test_await_is_python() {
        let source = r#"data = await fetch_data()
<div>{data}</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        assert!(result.python_code.contains("async def"));
        assert!(result.python_code.contains("data = await fetch_data()"));
    }

    #[test]
    fn test_augmented_assignment() {
        let source = r#"<div>
    counter += 1
    Total: {counter}
</div>
"#;
        let result = transpile(source);
        println!("Generated:\n{}", result.python_code);
        // += should be recognized as Python
        assert!(result.python_code.contains("counter += 1"));
    }
}
