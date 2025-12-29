use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use tree_sitter::Parser;

/// Configuration for transpilation.
#[derive(Debug, Clone)]
pub struct Options {
    /// Name of the generated Python function (default: "Template")
    pub function_name: String,
    /// Include injection metadata for IDE integration
    pub include_injections: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            function_name: "Template".to_string(),
            include_injections: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineType {
    /// Python control flow (if, for, while, etc.)
    Control,
    /// Block terminator (end)
    End,
    /// Python statement detected by tree-sitter
    Python,
    /// Content (text that should be output literally, including HTML)
    Content,
    /// Explicit t-string escape (t"..." or t"""...""")
    TString,
    /// Comment (# ...)
    Comment,
    /// Empty/whitespace only
    Empty,
    /// Component opening tag: <{Component} attrs>
    ComponentOpen,
    /// Component closing tag: </{Component}>
    ComponentClose,
}

/// Parsed component opening tag
#[derive(Debug, Clone)]
struct ComponentTag {
    name: String,
    name_offset: usize, // byte offset of name start within the line
    attrs: Vec<ComponentAttr>,
    is_self_closing: bool,
    /// Content after > on the same line (for inline content)
    trailing_content: Option<String>,
}

/// Component attribute with source position info for injection
#[derive(Debug, Clone, PartialEq)]
struct ComponentAttr {
    name: String,
    value: String,       // Python expression
    value_offset: usize, // byte offset of value start within the line
    is_spread: bool,     // true for {**expr}
    is_shorthand: bool,  // true for {name} shorthand
}

#[derive(Debug, Clone)]
struct Line {
    line_type: LineType,
    text: String,
    line_number: usize,
    /// Character offset (UTF-16 code units) for IDE compatibility
    char_offset: usize,
    /// For TString lines, the extracted content without t"..." wrapper
    tstring_content: Option<String>,
    /// For ComponentOpen lines, the parsed component tag
    component: Option<ComponentTag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Mapping {
    pub gen_line: usize,
    pub gen_col: usize,
    pub src_line: usize,
    pub src_col: usize,
}

/// Python injection range with prefix/suffix to wrap source into valid Python
#[derive(Debug, Clone, Serialize)]
pub struct PythonInjection {
    pub start: usize,
    pub end: usize,
    pub prefix: String,
    pub suffix: String,
}

/// HTML injection range (expressions already excluded)
#[derive(Debug, Clone, Serialize)]
pub struct HtmlInjection {
    pub start: usize,
    pub end: usize,
}

/// Language injection metadata for IDE integration
#[derive(Debug, Clone, Serialize, Default)]
pub struct Injections {
    pub python: Vec<PythonInjection>,
    pub html: Vec<HtmlInjection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranspileResult {
    pub code: String,
    pub mappings: Vec<Mapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub injections: Option<Injections>,
}

lazy_static! {
    static ref END_LINE: Regex = Regex::new(r"^[ \t]*end[ \t]*$").unwrap();
    // Regex for control flow keywords that tree-sitter can't parse standalone
    static ref ELSE_LINE: Regex = Regex::new(r"^[ \t]*else[ \t]*:[ \t]*$").unwrap();
    static ref ELIF_LINE: Regex = Regex::new(r"^[ \t]*elif[ \t]+.+:[ \t]*$").unwrap();
    static ref CASE_LINE: Regex = Regex::new(r"^[ \t]*case[ \t]+.+:[ \t]*$").unwrap();
    // Exception handling keywords (try: alone can't be parsed by tree-sitter)
    static ref TRY_LINE: Regex = Regex::new(r"^[ \t]*try[ \t]*:[ \t]*$").unwrap();
    static ref EXCEPT_LINE: Regex = Regex::new(r"^[ \t]*except[ \t]*.*:[ \t]*$").unwrap();
    static ref FINALLY_LINE: Regex = Regex::new(r"^[ \t]*finally[ \t]*:[ \t]*$").unwrap();
    // Multi-line string assignments (tree-sitter can't parse incomplete strings)
    static ref MULTILINE_STRING_START: Regex = Regex::new(r#"^[ \t]*[a-zA-Z_][a-zA-Z0-9_]*[ \t]*=[ \t]*("""|''')"#).unwrap();
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

/// Check if a line is an import statement
fn is_import_statement(parser: &mut Parser, text: &str) -> bool {
    match parse_python_line(parser, text) {
        Some((kind, _)) => matches!(kind.as_str(), "import_statement" | "import_from_statement"),
        None => false,
    }
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

fn is_comment(text: &str) -> bool {
    text.trim().starts_with('#')
}

/// Track bracket depth and multi-line string/tag state for statement detection
#[derive(Debug, Default)]
struct BracketState {
    parens: i32,             // ()
    brackets: i32,           // []
    braces: i32,             // {}
    backslash: bool,         // line continuation
    in_triple_string: bool,  // inside """ or '''
    triple_string_char: char, // which quote type
    in_html_tag: bool,       // inside unclosed HTML tag (< without >)
}

impl BracketState {
    fn is_continuation(&self) -> bool {
        self.parens > 0 || self.brackets > 0 || self.braces > 0 || self.backslash || self.in_triple_string
    }

    fn is_in_html_tag(&self) -> bool {
        self.in_html_tag
    }

    /// Check if a line starts an HTML tag and track whether it's closed
    fn update_html_tag(&mut self, line: &str) {
        let trimmed = line.trim();

        if self.in_html_tag {
            // We're continuing a multi-line tag, check if it closes
            if trimmed.ends_with('>') || trimmed.ends_with("/>") {
                self.in_html_tag = false;
            }
            return;
        }

        // Check if this line starts an HTML tag that doesn't close on this line
        // Look for < followed by tag name, but not closed with >
        if trimmed.starts_with('<') && !trimmed.starts_with("</") && !trimmed.starts_with("<{") {
            // Check if the tag closes on this line
            let has_close = trimmed.ends_with('>') || trimmed.contains('>');
            if !has_close {
                self.in_html_tag = true;
            }
        }
    }

    fn update(&mut self, line: &str) {
        self.backslash = line.trim_end().ends_with('\\');

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut in_single_string = false;
        let mut single_string_char = ' ';

        while i < chars.len() {
            // Handle triple-quoted strings
            if !in_single_string && i + 2 < chars.len() {
                let triple = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
                if triple == "\"\"\"" || triple == "'''" {
                    if self.in_triple_string && chars[i] == self.triple_string_char {
                        // Closing triple quote
                        self.in_triple_string = false;
                        i += 3;
                        continue;
                    } else if !self.in_triple_string {
                        // Opening triple quote
                        self.in_triple_string = true;
                        self.triple_string_char = chars[i];
                        i += 3;
                        continue;
                    }
                }
            }

            // Skip content inside triple strings
            if self.in_triple_string {
                i += 1;
                continue;
            }

            let c = chars[i];

            // Handle single-quoted strings
            if !in_single_string && (c == '"' || c == '\'') {
                in_single_string = true;
                single_string_char = c;
                i += 1;
                continue;
            }
            if in_single_string {
                if c == '\\' && i + 1 < chars.len() {
                    i += 2; // Skip escaped char
                    continue;
                }
                if c == single_string_char {
                    in_single_string = false;
                }
                i += 1;
                continue;
            }

            // Track brackets outside of strings
            match c {
                '(' => self.parens += 1,
                ')' => self.parens = (self.parens - 1).max(0),
                '[' => self.brackets += 1,
                ']' => self.brackets = (self.brackets - 1).max(0),
                '{' => self.braces += 1,
                '}' => self.braces = (self.braces - 1).max(0),
                '#' => break, // Stop at comments
                _ => {}
            }
            i += 1;
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

/// Parse a component opening tag: <{Component} attr={val} attr2="literal">
/// Tracks byte offsets for IDE injection support
fn parse_component_open(text: &str) -> Option<ComponentTag> {
    // Calculate leading whitespace offset
    let leading_ws = text.len() - text.trim_start().len();
    let trimmed = text.trim();
    if !trimmed.starts_with("<{") {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut i = 2; // skip <{

    // Extract component name
    let name_start = i;
    let name_offset = leading_ws + name_start;
    let mut brace_depth = 1;
    while i < bytes.len() && brace_depth > 0 {
        match bytes[i] {
            b'{' => brace_depth += 1,
            b'}' => brace_depth -= 1,
            _ => {}
        }
        if brace_depth > 0 {
            i += 1;
        }
    }
    let name = String::from_utf8_lossy(&bytes[name_start..i]).to_string();
    i += 1; // skip closing }

    let mut attrs = Vec::new();
    let mut is_self_closing = false;

    // Parse attributes until > or />
    while i < bytes.len() {
        // Skip whitespace
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        // Check for self-closing /> or closing >
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'>' {
            is_self_closing = true;
            i += 2;
            break;
        }
        if bytes[i] == b'>' {
            i += 1;
            break;
        }

        // Check for {shorthand} or {**spread}
        if bytes[i] == b'{' {
            i += 1;
            let expr_start = i;
            let expr_offset = leading_ws + expr_start;
            let mut bd = 1;
            while i < bytes.len() && bd > 0 {
                match bytes[i] {
                    b'{' => bd += 1,
                    b'}' => bd -= 1,
                    _ => {}
                }
                if bd > 0 { i += 1; }
            }
            let expr = String::from_utf8_lossy(&bytes[expr_start..i]).to_string();
            i += 1;
            if expr.starts_with("**") {
                attrs.push(ComponentAttr {
                    name: "**".to_string(),
                    value: expr[2..].to_string(),
                    value_offset: expr_offset + 2, // skip **
                    is_spread: true,
                    is_shorthand: false,
                });
            } else {
                // Shorthand: {title} -> title=title
                attrs.push(ComponentAttr {
                    name: expr.clone(),
                    value: expr,
                    value_offset: expr_offset,
                    is_spread: false,
                    is_shorthand: true,
                });
            }
            continue;
        }

        // Parse attribute name
        let attr_start = i;
        while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-' || bytes[i] == b':') {
            i += 1;
        }
        let attr_name = String::from_utf8_lossy(&bytes[attr_start..i]).to_string();

        if attr_name.is_empty() {
            i += 1; // skip unknown char
            continue;
        }

        // Check for = followed by value
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            if i < bytes.len() && bytes[i] == b'{' {
                // attr={expr}
                i += 1;
                let expr_start = i;
                let expr_offset = leading_ws + expr_start;
                let mut bd = 1;
                while i < bytes.len() && bd > 0 {
                    match bytes[i] {
                        b'{' => bd += 1,
                        b'}' => bd -= 1,
                        _ => {}
                    }
                    if bd > 0 { i += 1; }
                }
                let expr = String::from_utf8_lossy(&bytes[expr_start..i]).to_string();
                i += 1;
                attrs.push(ComponentAttr {
                    name: attr_name,
                    value: expr,
                    value_offset: expr_offset,
                    is_spread: false,
                    is_shorthand: false,
                });
            } else if i < bytes.len() && bytes[i] == b'"' {
                // attr="literal" - no injection needed for string literals
                i += 1;
                let val_start = i;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                let val = String::from_utf8_lossy(&bytes[val_start..i]).to_string();
                i += 1;
                attrs.push(ComponentAttr {
                    name: attr_name,
                    value: format!("\"{}\"", val),
                    value_offset: 0, // no injection for literals
                    is_spread: false,
                    is_shorthand: false,
                });
            } else if i < bytes.len() && bytes[i] == b'\'' {
                // attr='literal' - no injection needed
                i += 1;
                let val_start = i;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                let val = String::from_utf8_lossy(&bytes[val_start..i]).to_string();
                i += 1;
                attrs.push(ComponentAttr {
                    name: attr_name,
                    value: format!("\"{}\"", val),
                    value_offset: 0,
                    is_spread: false,
                    is_shorthand: false,
                });
            }
        } else {
            // Boolean attribute without value - no injection needed
            attrs.push(ComponentAttr {
                name: attr_name,
                value: "True".to_string(),
                value_offset: 0,
                is_spread: false,
                is_shorthand: false,
            });
        }
    }

    // Check for trailing content after >
    let trailing_content = if i < bytes.len() {
        let rest = String::from_utf8_lossy(&bytes[i..]).to_string();
        let rest_trimmed = rest.trim();
        if !rest_trimmed.is_empty() {
            Some(rest_trimmed.to_string())
        } else {
            None
        }
    } else {
        None
    };

    Some(ComponentTag {
        name,
        name_offset,
        attrs,
        is_self_closing,
        trailing_content,
    })
}

/// Parse a component closing tag: </{Component}>
fn parse_component_close(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if !trimmed.starts_with("</{") {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 3; // skip </{

    // Extract component name
    let name_start = i;
    let mut brace_depth = 1;
    while i < chars.len() && brace_depth > 0 {
        match chars[i] {
            '{' => brace_depth += 1,
            '}' => brace_depth -= 1,
            _ => {}
        }
        if brace_depth > 0 {
            i += 1;
        }
    }
    let name: String = chars[name_start..i].iter().collect();

    // Verify it ends with }>
    if i + 1 < chars.len() && chars[i] == '}' && chars[i + 1] == '>' {
        Some(name)
    } else {
        None
    }
}

fn classify_line(parser: &mut Parser, text: &str) -> LineType {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return LineType::Empty;
    }

    if END_LINE.is_match(text) {
        return LineType::End;
    }

    // Check for component tags before other patterns
    if trimmed.starts_with("</{") {
        return LineType::ComponentClose;
    }
    if trimmed.starts_with("<{") {
        return LineType::ComponentOpen;
    }

    // else/elif/case/try/except/finally need special handling (tree-sitter can't parse them standalone)
    if ELSE_LINE.is_match(text) || ELIF_LINE.is_match(text) || CASE_LINE.is_match(text)
        || TRY_LINE.is_match(text) || EXCEPT_LINE.is_match(text) || FINALLY_LINE.is_match(text) {
        return LineType::Control;
    }

    if is_control_flow(parser, text) {
        return LineType::Control;
    }

    if is_comment(text) {
        return LineType::Comment;
    }

    if is_python_statement(parser, trimmed) {
        return LineType::Python;
    }

    // Check for multi-line string assignment start (tree-sitter can't parse incomplete strings)
    if MULTILINE_STRING_START.is_match(text) {
        return LineType::Python;
    }

    LineType::Content
}

/// Count UTF-16 code units in a string (for IDE compatibility)
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Convert a byte offset within a string to UTF-16 code unit offset
fn byte_offset_to_utf16(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset].encode_utf16().count()
}

fn lex(source: &str) -> Vec<Line> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("Failed to load Python grammar");

    let mut lines = Vec::new();
    let mut char_offset = 0; // UTF-16 code units for IDE compatibility
    let mut bracket_state = BracketState::default();
    let mut line_number = 0;

    let bytes = source.as_bytes();
    let mut start = 0;

    while start < bytes.len() {
        // Find end of line (LF or CRLF)
        let mut end = start;
        while end < bytes.len() && bytes[end] != b'\n' {
            end += 1;
        }

        // Get line content (excluding \r if present)
        let line_end = if end > start && bytes[end - 1] == b'\r' { end - 1 } else { end };
        let text = &source[start..line_end];

        let tstring_content = extract_tstring_content(text);
        let line_type = if tstring_content.is_some() {
            LineType::TString
        } else if bracket_state.is_continuation() {
            LineType::Python
        } else if bracket_state.is_in_html_tag() {
            // Inside a multi-line HTML tag, treat everything as content
            LineType::Content
        } else {
            classify_line(&mut parser, text)
        };

        // Parse component info if applicable
        let component = if line_type == LineType::ComponentOpen {
            parse_component_open(text)
        } else {
            None
        };

        lines.push(Line {
            line_type,
            text: text.to_string(),
            line_number,
            char_offset,
            tstring_content,
            component,
        });

        if matches!(line_type, LineType::Python | LineType::Control) {
            bracket_state.update(text);
        } else if line_type == LineType::Content {
            // Track HTML tag state for content lines
            bracket_state.update_html_tag(text);
            bracket_state.reset_backslash();
        } else {
            bracket_state.reset_backslash();
        }

        // Move past the newline - count UTF-16 code units
        let line_utf16_len = utf16_len(text);
        char_offset += line_utf16_len + 1; // +1 for newline
        start = if end < bytes.len() { end + 1 } else { end };
        line_number += 1;
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

/// A parameter with its optional preceding comment
struct ParamWithComment<'a> {
     comment: Option<&'a Line>,
    param: &'a Line,
}

/// Result of find_structure: (leading_comments, params, trailing_comments, body_start)
/// - leading_comments: Comments before any parameters (shown above def)
/// - params: Parameters with their preceding comments
/// - trailing_comments: Comments after params but before body (shown inside function)
/// - body_start: Line index where body begins
fn find_structure(lines: &[Line]) -> (Vec<&Line>, Vec<ParamWithComment>, Vec<&Line>, usize) {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_python::LANGUAGE.into()).expect("Failed to load Python grammar");

    let mut leading: Vec<&Line> = Vec::new();
    let mut params: Vec<ParamWithComment> = Vec::new();
    let mut trailing: Vec<&Line> = Vec::new();
    let mut seen_param = false;
    let mut body_start = 0;
    let mut pending_comment: Option<(&str, &Line)> = None;
    let mut in_docstring = false;
    let mut docstring_quote: &str = "";

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.text.trim();

        // Handle multi-line docstrings in header
        if in_docstring {
            leading.push(line);
            body_start = i + 1;
            // Check if this line ends the docstring
            if trimmed.ends_with(docstring_quote) || trimmed == docstring_quote {
                in_docstring = false;
            }
            continue;
        }

        // Check for docstring start (only before params)
        if !seen_param && (trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''")) {
            docstring_quote = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
            // Flush any pending comment
            if let Some((_, cmt_line)) = pending_comment.take() {
                leading.push(cmt_line);
            }
            leading.push(line);
            body_start = i + 1;
            // Check if docstring ends on same line
            let after_start = &trimmed[3..];
            if !after_start.ends_with(docstring_quote) {
                in_docstring = true;
            }
            continue;
        }

        // Empty lines before first param go to leading
        if trimmed.is_empty() && !seen_param {
            // If we have a pending comment, it goes to leading
            if let Some((_, cmt_line)) = pending_comment.take() {
                leading.push(cmt_line);
            }
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        // Empty lines after first param - save any pending comment as trailing
        if trimmed.is_empty() && seen_param {
            if let Some((_, cmt_line)) = pending_comment.take() {
                trailing.push(cmt_line);
            }
            body_start = i + 1;
            continue;
        }

        // Comments
        if trimmed.starts_with('#') {
            // If we already have a pending comment, flush it appropriately
            if let Some((_, prev_cmt_line)) = pending_comment.take() {
                if seen_param {
                    trailing.push(prev_cmt_line);
                } else {
                    leading.push(prev_cmt_line);
                }
            }
            pending_comment = Some((trimmed, line));
            body_start = i + 1;
            continue;
        }

        // Use tree-sitter to detect type annotations (but not function definitions)
        let is_func_def = trimmed.starts_with("def ") || trimmed.starts_with("async def ");
        if !is_func_def && is_type_annotation(&mut parser, &line.text) {
            seen_param = true;
            params.push(ParamWithComment {
                comment: pending_comment.take().map(|(_, line)| line),
                param: line,
            });
            body_start = i + 1;
            continue;
        }

        // Import statements go to leading (top of file, outside function)
        if is_import_statement(&mut parser, &line.text) {
            // Flush any pending comment to leading
            if let Some((_, cmt_line)) = pending_comment.take() {
                leading.push(cmt_line);
            }
            leading.push(line);
            body_start = i + 1;
            continue;
        }

        // Something else (body starts) - flush pending comment to trailing
        if let Some((_, cmt_line)) = pending_comment.take() {
            if seen_param {
                trailing.push(cmt_line);
            } else {
                leading.push(cmt_line);
            }
        }
        break;
    }

    // Handle any remaining pending comment
    if let Some((_, cmt_line)) = pending_comment {
        if seen_param {
            trailing.push(cmt_line);
        } else {
            leading.push(cmt_line);
        }
    }

    (leading, params, trailing, body_start)
}

/// Returns (indent_len, trimmed_end) - start of content after whitespace, end before newlines.
fn trim_bounds(text: &str) -> (usize, usize) {
    let start = text.bytes().take_while(|&b| b == b' ' || b == b'\t').count();
    let end = text.trim_end_matches(['\r', '\n']).len();
    (start, end)
}

/// Extract parameter and optional inline comment from a parameter line.
/// Returns (param, Option<comment>) where comment includes the # prefix.
fn split_param_comment(text: &str) -> (String, Option<String>) {
    let mut in_string = false;
    let mut string_char = b' ';
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        if in_string {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == string_char {
                in_string = false;
            }
            i += 1;
        } else {
            if b == b'"' || b == b'\'' {
                in_string = true;
                string_char = b;
                i += 1;
            } else if b == b'#' {
                // Found comment outside string
                let param = text[..i].trim_end().to_string();
                let comment = text[i..].trim_end().to_string();
                return (param, Some(comment));
            } else {
                i += 1;
            }
        }
    }

    (text.to_string(), None)
}

/// Buffered content with source location tracking.
#[derive(Debug, Default)]
struct ContentBuffer {
    lines: Vec<String>,
    start: usize,
    end: usize,
    first_line: usize,
}

impl ContentBuffer {
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    fn push(&mut self, line: &str, char_offset: usize, line_number: usize) {
        if self.lines.is_empty() {
            self.start = char_offset;
            self.first_line = line_number;
        }
        self.lines.push(line.to_string());
        self.end = char_offset + utf16_len(line);
    }

    fn take(&mut self) -> Option<(String, usize, usize, usize)> {
        if self.lines.is_empty() {
            return None;
        }
        let result = (self.lines.join("\n"), self.start, self.end, self.first_line);
        self.lines.clear();
        self.start = 0;
        self.end = 0;
        self.first_line = 0;
        Some(result)
    }
}

struct Codegen {
    output: Vec<String>,
    mappings: Vec<Mapping>,
    injections: Option<Injections>,
    line: usize,
    used_helpers: UsedHelpers,
}

impl Codegen {
    fn new(include_injections: bool) -> Self {
        Self {
            output: Vec::new(),
            mappings: Vec::new(),
            injections: if include_injections { Some(Injections::default()) } else { None },
            line: 0,
            used_helpers: UsedHelpers::default(),
        }
    }

    fn emit(&mut self, code: String, src_line: usize, src_col: usize, gen_col: usize) {
        self.output.push(code);
        self.mappings.push(Mapping { gen_line: self.line, gen_col, src_line, src_col });
        self.line += 1;
    }

    fn emit_with_injection(&mut self, code: String, src_line: usize, src_col: usize, gen_col: usize,
                           byte_start: usize, byte_end: usize, prefix: String, suffix: String) {
        self.output.push(code);
        self.mappings.push(Mapping { gen_line: self.line, gen_col, src_line, src_col });
        if let Some(ref mut inj) = self.injections {
            inj.python.push(PythonInjection { start: byte_start, end: byte_end, prefix, suffix });
        }
        self.line += 1;
    }

    fn emit_empty(&mut self, src_line: usize) {
        self.output.push(String::new());
        self.mappings.push(Mapping { gen_line: self.line, gen_col: 0, src_line, src_col: 0 });
        self.line += 1;
    }

    fn emit_raw(&mut self, code: String) {
        self.output.push(code);
        self.line += 1;
    }

    fn flush_content(&mut self, buf: &mut ContentBuffer, level: usize, parts_var: &str, function_has_html: &mut Vec<bool>) {
        if let Some((content, start, end, first_line)) = buf.take() {
            let indent = "    ";
            // Transform special attribute patterns
            let (transformed, helpers) = transform_html_content(&content);
            self.used_helpers.merge(&helpers);
            let prefix = format!("{}{}.append(f\"\"\"", indent.repeat(level), parts_var);
            let suffix = "\"\"\")";
            self.output.push(format!("{}{}{}", prefix, transformed, suffix));
            self.mappings.push(Mapping { gen_line: self.line, gen_col: prefix.len(), src_line: first_line, src_col: 0 });
            if let Some(ref mut inj) = self.injections {
                inj.python.push(PythonInjection { start, end, prefix, suffix: format!("{}\n", suffix) });
                for (seg_start, seg_end) in find_html_segments(&content) {
                    // Convert byte offsets within content to UTF-16 offsets
                    let utf16_seg_start = byte_offset_to_utf16(&content, seg_start);
                    let utf16_seg_end = byte_offset_to_utf16(&content, seg_end);
                    inj.html.push(HtmlInjection { start: start + utf16_seg_start, end: start + utf16_seg_end });
                }
            }
            self.line += 1;
            // Mark the innermost function as having HTML content
            if let Some(last) = function_has_html.last_mut() {
                *last = true;
            }
        }
    }

    fn into_result(self) -> TranspileResult {
        let mut code = self.output.join("\n");
        if !code.is_empty() && !code.ends_with('\n') {
            code.push('\n');
        }
        TranspileResult { code, mappings: self.mappings, injections: self.injections }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BlockType {
    Block,
    /// Block continuation (else, elif, except, finally) - part of a compound statement
    Continuation,
    Match,
    Case,
    Component {
        name: String,
        attrs: Vec<ComponentAttr>,
        slot_var: String,
    },
    /// Function definition - needs _parts setup if it contains HTML
    Function {
        /// Position in output where _parts = [] should be inserted if needed
        insert_pos: usize,
    },
    /// Class definition - no special handling needed
    Class,
}

fn find_html_segments(text: &str) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut segment_start = 0;
    let mut brace_depth = 0;

    for (i, c) in text.char_indices() {
        match c {
            '{' if brace_depth == 0 => {
                if i > segment_start {
                    segments.push((segment_start, i));
                }
                brace_depth = 1;
            }
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    segment_start = i + 1; // +1 to skip the '}'
                }
            }
            _ => {}
        }
    }

    // Add final segment if not inside braces
    if brace_depth == 0 && segment_start < text.len() {
        segments.push((segment_start, text.len()));
    }

    segments
}

/// Track which helper functions are used
#[derive(Debug, Default, Clone)]
struct UsedHelpers {
    attr: bool,
    class: bool,
    style: bool,
    spread: bool,
}

impl UsedHelpers {
    fn merge(&mut self, other: &UsedHelpers) {
        self.attr |= other.attr;
        self.class |= other.class;
        self.style |= other.style;
        self.spread |= other.spread;
    }
}

/// Transform HTML attributes with special patterns:
/// - `attr={expr}` (no quotes) → `{_attr('attr', expr)}`
/// - `class={expr}` → `class="{_class(expr)}"`
/// - `style={expr}` → `style="{_style(expr)}"`
/// - `{**expr}` → `{_spread(expr)}`
///
/// Only transforms at HTML level - Python expressions inside {} are passed through unchanged.
fn transform_html_content(content: &str) -> (String, UsedHelpers) {
    let mut helpers = UsedHelpers::default();
    let mut result = String::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for {**expr} spread pattern (at HTML level)
        if i < chars.len() - 2 && chars[i] == '{' && chars[i + 1] == '*' && chars[i + 2] == '*' {
            i += 3; // skip {**
            let expr_start = i;
            let mut brace_depth = 1;
            while i < chars.len() && brace_depth > 0 {
                match chars[i] {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
                if brace_depth > 0 {
                    i += 1;
                }
            }
            let expr: String = chars[expr_start..i].iter().collect();
            i += 1; // skip closing }
            helpers.spread = true;
            helpers.attr = true; // _spread uses _attr
            result.push_str(&format!("{{_spread({})}}", expr));
        }
        // Regular Python expression {expr} - pass through unchanged
        else if chars[i] == '{' {
            result.push('{');
            i += 1;
            let mut brace_depth = 1;
            while i < chars.len() && brace_depth > 0 {
                match chars[i] {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
                result.push(chars[i]);
                i += 1;
            }
        }
        // Look for pattern: attrname={ (HTML attribute with expression value)
        else if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == '-' || chars[i] == '@' || chars[i] == ':' {
            let attr_start = i;
            // Consume attribute name (allows alphanumeric, _, -, @, :)
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-' || chars[i] == '@' || chars[i] == ':') {
                i += 1;
            }
            let attr_name: String = chars[attr_start..i].iter().collect();

            // Check for ={
            if i < chars.len() - 1 && chars[i] == '=' && chars[i + 1] == '{' {
                // This is attr={expr} pattern - find the closing brace
                i += 2; // skip ={
                let expr_start = i;
                let mut brace_depth = 1;
                while i < chars.len() && brace_depth > 0 {
                    match chars[i] {
                        '{' => brace_depth += 1,
                        '}' => brace_depth -= 1,
                        _ => {}
                    }
                    if brace_depth > 0 {
                        i += 1;
                    }
                }
                let expr: String = chars[expr_start..i].iter().collect();
                i += 1; // skip closing }

                // Transform based on attribute type
                if attr_name == "class" {
                    helpers.class = true;
                    result.push_str(&format!("class=\"{{_class({})}}\"", expr));
                } else if attr_name == "style" {
                    helpers.style = true;
                    result.push_str(&format!("style=\"{{_style({})}}\"", expr));
                } else {
                    helpers.attr = true;
                    result.push_str(&format!("{{_attr('{}', {})}}", attr_name, expr));
                }
            } else {
                // Not our pattern, just copy the attr name
                result.push_str(&attr_name);
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    (result, helpers)
}

pub fn transpile(source: &str) -> TranspileResult {
    transpile_with(source, Options::default())
}

pub fn transpile_with(source: &str, options: Options) -> TranspileResult {
    let lines = lex(source);
    let (leading, params, trailing, body_start) = find_structure(&lines);
    let is_async = has_await(&lines) || has_async_construct(&lines);

    let mut gen = Codegen::new(options.include_injections);
    let indent = "    ";
    let def_kw = if is_async { "async def" } else { "def" };
    let name = &options.function_name;

    for line in &leading {
        let (start, end) = trim_bounds(&line.text);
        if start < end {
            gen.emit_with_injection(
                line.text[start..end].to_string(),
                line.line_number, start, 0,
                line.char_offset + byte_offset_to_utf16(&line.text, start),
                line.char_offset + byte_offset_to_utf16(&line.text, end),
                String::new(), "\n".to_string(),
            );
        } else {
            gen.emit_empty(line.line_number);
        }
    }

    if !params.is_empty() {
        // Extract parameters with their comments (both preceding and inline)
        let param_parts: Vec<(Option<String>, String, Option<String>)> = params.iter()
            .map(|pwc| {
                let (s, e) = trim_bounds(&pwc.param.text);
                let (param_text, inline_comment) = split_param_comment(&pwc.param.text[s..e]);
                let comment_text = pwc.comment.map(|line| {
                    let (cs, ce) = trim_bounds(&line.text);
                    line.text[cs..ce].to_string()
                });
                (comment_text, param_text, inline_comment)
            })
            .collect();

        let has_comments = param_parts.iter().any(|(pre, _, inl)| pre.is_some() || inl.is_some());

        if has_comments {
            // Multi-line function signature with comments
            gen.emit_raw(format!("{} {}(", def_kw, name));
            for (i, (pre_comment, param, inline_comment)) in param_parts.iter().enumerate() {
                let is_last = i == params.len() - 1;
                // Preceding comment (on its own line)
                if let Some(cmt) = pre_comment {
                    gen.emit_raw(format!("{}    {}", indent, cmt));
                }
                // Inline comment (after parameter)
                if let Some(cmt) = inline_comment {
                    gen.emit_raw(format!("{}    {}", indent, cmt));
                }
                if is_last {
                    gen.emit_raw(format!("{}    {},", indent, param));
                } else {
                    gen.emit_raw(format!("{}    {},", indent, param));
                }
            }
            gen.emit_raw("):".to_string());
        } else {
            // Single-line function signature
            let param_strs: Vec<_> = param_parts.iter().map(|(_, p, _)| p.clone()).collect();
            gen.emit(
                format!("{} {}({}):", def_kw, name, param_strs.join(", ")),
                params[0].param.line_number, trim_bounds(&params[0].param.text).0,
                def_kw.len() + 1 + name.len() + 1,
            );
        }
        if let Some(ref mut inj) = gen.injections {
            let func_prefix = format!("{} {}(", def_kw, name);
            for (i, pwc) in params.iter().enumerate() {
                let (s, e) = trim_bounds(&pwc.param.text);
                inj.python.push(PythonInjection {
                    start: pwc.param.char_offset + byte_offset_to_utf16(&pwc.param.text, s),
                    end: pwc.param.char_offset + byte_offset_to_utf16(&pwc.param.text, e),
                    prefix: if i == 0 { func_prefix.clone() } else { ", ".to_string() },
                    suffix: if i == params.len() - 1 { "):\n    _parts = []\n".to_string() } else { String::new() },
                });
            }
        }
    } else {
        gen.emit(format!("{} {}():", def_kw, name), body_start.min(lines.len().saturating_sub(1)), 0, 0);
        if let Some(ref mut inj) = gen.injections {
            inj.python.push(PythonInjection {
                start: 0, end: 0,
                prefix: format!("{} {}():\n    _parts = []\n", def_kw, name),
                suffix: String::new(),
            });
        }
    }

    gen.emit_raw(format!("{}_parts = []", indent));

    // Emit trailing comments (comments after params but before body)
    for line in &trailing {
        let (start, end) = trim_bounds(&line.text);
        if start < end {
            gen.emit_raw(format!("{}{}", indent, &line.text[start..end]));
        }
    }

    let helpers_insert_pos = gen.output.len(); // Position right after _parts = [] and trailing comments

    let mut level = 1usize;
    let mut stack: Vec<BlockType> = Vec::new();
    let mut block_has_content: Vec<bool> = Vec::new();
    let mut buffer = ContentBuffer::default();
    let mut slot_counter = 0usize;
    let mut parts_var_stack: Vec<String> = vec!["_parts".to_string()];
    let mut function_has_html: Vec<bool> = Vec::new(); // Track if each function scope has HTML

    // Helper to get current parts variable
    let current_parts = |stack: &Vec<String>| stack.last().cloned().unwrap_or_else(|| "_parts".to_string());

    for line in &lines[body_start..] {
        let (start, end) = trim_bounds(&line.text);
        let trimmed = &line.text[start..end];

        match line.line_type {
            LineType::Empty => {
                if buffer.is_empty() {
                    gen.emit_empty(line.line_number);
                } else {
                    buffer.push("", line.char_offset, line.line_number);
                }
            }

            LineType::Content => {
                buffer.push(&line.text[..end], line.char_offset, line.line_number);
                if let Some(last) = block_has_content.last_mut() { *last = true; }
            }

            LineType::TString => {
                if let Some(ref content) = line.tstring_content {
                    buffer.push(content, line.char_offset, line.line_number);
                    if let Some(last) = block_has_content.last_mut() { *last = true; }
                }
            }

            LineType::Control => {
                gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);
                let is_dedent = ["else", "elif", "except", "finally"].iter().any(|kw| trimmed.starts_with(kw));
                let is_case = trimmed.starts_with("case");
                let is_function = trimmed.starts_with("def ") || trimmed.starts_with("async def ");
                let is_class = trimmed.starts_with("class ");

                // For dedent keywords, check if the current block is empty and insert pass
                if is_dedent {
                    if let Some(has_content) = block_has_content.last() {
                        if !has_content {
                            gen.emit_raw(format!("{}pass", indent.repeat(level)));
                        }
                    }
                    // Mark block as having content (the dedent line itself is content)
                    if let Some(last) = block_has_content.last_mut() {
                        *last = true;
                    }
                }

                let lvl = if is_dedent {
                    level.saturating_sub(1).max(1)
                } else if is_case && matches!(stack.last(), Some(BlockType::Case)) {
                    // Check if previous case is empty
                    if let Some(has_content) = block_has_content.last() {
                        if !has_content {
                            gen.emit_raw(format!("{}pass", indent.repeat(level)));
                        }
                    }
                    stack.pop();
                    block_has_content.pop();
                    level = level.saturating_sub(1);
                    level
                } else {
                    level
                };

                gen.emit_with_injection(
                    format!("{}{}", indent.repeat(lvl), trimmed),
                    line.line_number, start, indent.len() * lvl,
                    line.char_offset + byte_offset_to_utf16(&line.text, start),
                    line.char_offset + byte_offset_to_utf16(&line.text, end),
                    indent.repeat(lvl), "\n".to_string(),
                );

                if is_dedent {
                    // Dedent keywords (else, elif, except, finally) start a continuation block
                    // We decrement level for the keyword line, but increment back for the body
                    stack.push(BlockType::Continuation);
                    block_has_content.push(false);
                    // Level stays the same: we emitted at level-1, but body is at current level
                    // No change to level needed
                } else {
                    // A child block counts as content for the parent block
                    if let Some(last) = block_has_content.last_mut() {
                        *last = true;
                    }

                    let block_type = if is_case {
                        BlockType::Case
                    } else if trimmed.starts_with("match") {
                        BlockType::Match
                    } else if is_function {
                        // Record position where _parts = [] should be inserted if needed
                        function_has_html.push(false);
                        BlockType::Function { insert_pos: gen.output.len() }
                    } else if is_class {
                        BlockType::Class
                    } else {
                        BlockType::Block
                    };
                    stack.push(block_type);
                    block_has_content.push(false);
                    level += 1;
                }
            }

            LineType::End => {
                gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);

                // First, check if current innermost block is empty and insert pass
                if let Some(&has_content) = block_has_content.last() {
                    if !has_content {
                        gen.emit_raw(format!("{}pass", indent.repeat(level)));
                    }
                }

                // Pop all Continuation blocks (else, elif, except, finally)
                // Don't decrement level - Continuations don't add a level
                while matches!(stack.last(), Some(BlockType::Continuation)) {
                    stack.pop();
                    block_has_content.pop();
                    // Don't decrement level here - we didn't increment when pushing Continuation
                }

                // Handle Case blocks - insert pass for empty cases
                while matches!(stack.last(), Some(BlockType::Case)) {
                    stack.pop();
                    block_has_content.pop();
                    level = level.saturating_sub(1);
                }

                // Handle the main block
                match stack.pop() {
                    Some(BlockType::Function { insert_pos }) => {
                        let has_html = function_has_html.pop().unwrap_or(false);
                        let main_empty = block_has_content.pop() == Some(false);
                        if has_html {
                            // Insert _parts = [] at the saved position
                            gen.output.insert(insert_pos, format!("{}_parts = []", indent.repeat(level)));
                            gen.line += 1;
                            // Emit return statement
                            gen.emit_raw(format!("{}return \"\".join(_parts)", indent.repeat(level)));
                        } else if main_empty {
                            gen.emit_raw(format!("{}pass", indent.repeat(level)));
                        }
                        level = level.saturating_sub(1);
                    }
                    Some(BlockType::Class) => {
                        let main_empty = block_has_content.pop() == Some(false);
                        if main_empty {
                            gen.emit_raw(format!("{}pass", indent.repeat(level)));
                        }
                        level = level.saturating_sub(1);
                    }
                    Some(BlockType::Block) | Some(BlockType::Match) => {
                        block_has_content.pop();
                        level = level.saturating_sub(1);
                    }
                    Some(BlockType::Continuation) | Some(BlockType::Case) | Some(BlockType::Component { .. }) => {
                        block_has_content.pop();
                        level = level.saturating_sub(1);
                    }
                    None => {}
                }
                level = level.max(1);
            }

            LineType::ComponentOpen => {
                gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);
                if let Some(ref comp) = line.component {
                    // Helper to format args
                    let format_args = |attrs: &[ComponentAttr]| -> String {
                        attrs.iter()
                            .map(|a| if a.is_spread { format!("**{}", a.value) } else { format!("{}={}", a.name, a.value) })
                            .collect::<Vec<_>>()
                            .join(", ")
                    };

                    // Emit Python injections for component name and attributes
                    if let Some(ref mut inj) = gen.injections {
                        // Inject component name
                        let name_start = line.char_offset + byte_offset_to_utf16(&line.text, comp.name_offset);
                        let name_end = name_start + utf16_len(&comp.name);
                        inj.python.push(PythonInjection {
                            start: name_start,
                            end: name_end,
                            prefix: "_ = ".to_string(),
                            suffix: "\n".to_string(),
                        });

                        // Inject attribute expressions (only for non-literals)
                        for attr in &comp.attrs {
                            if attr.value_offset > 0 {
                                let val_start = line.char_offset + byte_offset_to_utf16(&line.text, attr.value_offset);
                                let val_end = val_start + utf16_len(&attr.value);
                                inj.python.push(PythonInjection {
                                    start: val_start,
                                    end: val_end,
                                    prefix: "_ = ".to_string(),
                                    suffix: "\n".to_string(),
                                });
                            }
                        }
                    }

                    if comp.is_self_closing {
                        // Self-closing: <{Component} attrs />
                        // Generate: _parts.append(Component(attrs))
                        let args = format_args(&comp.attrs);
                        gen.emit_raw(format!("{}{}.append({}({}))",
                            indent.repeat(level), current_parts(&parts_var_stack), comp.name, args));
                        if let Some(last) = block_has_content.last_mut() { *last = true; }
                    } else {
                        // Opening tag with children
                        let slot_var = format!("_slot_{}", slot_counter);
                        slot_counter += 1;

                        // Initialize slot parts list
                        gen.emit_raw(format!("{}{}_parts = []", indent.repeat(level), slot_var));

                        // Handle trailing content on same line as opening tag
                        if let Some(ref trailing) = comp.trailing_content {
                            // Check if trailing is the closing tag
                            if let Some(close_name) = parse_component_close(trailing) {
                                if close_name == comp.name {
                                    // Inline component: <{C}>content</{C}> - content is empty here
                                    let args = format_args(&comp.attrs);
                                    gen.emit_raw(format!("{}{}.append({}({}))",
                                        indent.repeat(level), current_parts(&parts_var_stack), comp.name, args));
                                    if let Some(last) = block_has_content.last_mut() { *last = true; }
                                    continue;
                                }
                            }
                            // Has trailing content - add to slot buffer
                            buffer.push(trailing, line.char_offset, line.line_number);
                        }

                        // Push component block (no level increase - slots don't add Python indentation)
                        stack.push(BlockType::Component {
                            name: comp.name.clone(),
                            attrs: comp.attrs.clone(),
                            slot_var: slot_var.clone(),
                        });
                        block_has_content.push(false);
                        parts_var_stack.push(format!("{}_parts", slot_var));
                    }
                }
            }

            LineType::ComponentClose => {
                gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);

                // Find matching component block
                if let Some(BlockType::Component { name, attrs, slot_var }) = stack.pop() {
                    block_has_content.pop();
                    parts_var_stack.pop();
                    // No level change - component slots don't affect Python indentation

                    // Join slot parts
                    gen.emit_raw(format!("{}{} = \"\".join({}_parts)",
                        indent.repeat(level), slot_var, slot_var));

                    // Generate component call with slot
                    let mut args: Vec<String> = attrs.iter()
                        .map(|a| if a.is_spread { format!("**{}", a.value) } else { format!("{}={}", a.name, a.value) })
                        .collect();
                    args.push(format!("slot={}", slot_var));

                    let parent_parts = current_parts(&parts_var_stack);
                    gen.emit_raw(format!("{}{}.append({}({}))",
                        indent.repeat(level), parent_parts, name, args.join(", ")));

                    if let Some(last) = block_has_content.last_mut() { *last = true; }
                }
            }

            LineType::Comment | LineType::Python => {
                gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);
                gen.emit_with_injection(
                    format!("{}{}", indent.repeat(level), trimmed),
                    line.line_number, start, indent.len() * level,
                    line.char_offset + byte_offset_to_utf16(&line.text, start),
                    line.char_offset + byte_offset_to_utf16(&line.text, end),
                    indent.repeat(level), "\n".to_string(),
                );
                if line.line_type == LineType::Python {
                    if let Some(last) = block_has_content.last_mut() { *last = true; }
                }
            }
        }
    }

    gen.flush_content(&mut buffer, level, &current_parts(&parts_var_stack), &mut function_has_html);

    // Insert helper functions right after _parts = [], only if used
    // We need to insert them early since they must be defined before use
    let mut helper_lines = Vec::new();
    if gen.used_helpers.attr {
        helper_lines.push(format!("{}def _attr(n, v):", indent));
        helper_lines.push(format!("{}    if v is True: return f' {{n}}'", indent));
        helper_lines.push(format!("{}    if v is False or v is None: return ''", indent));
        helper_lines.push(format!("{}    return f' {{n}}=\"{{v}}\"'", indent));
    }
    if gen.used_helpers.class {
        helper_lines.push(format!("{}def _class(v):", indent));
        helper_lines.push(format!("{}    if isinstance(v, str): return v", indent));
        helper_lines.push(format!("{}    if isinstance(v, list): return ' '.join(filter(None, (_class(i) if isinstance(i, (list, dict)) else (str(i) if i else '') for i in v)))", indent));
        helper_lines.push(format!("{}    if isinstance(v, dict): return ' '.join(k for k, x in v.items() if x)", indent));
        helper_lines.push(format!("{}    return str(v) if v else ''", indent));
    }
    if gen.used_helpers.style {
        helper_lines.push(format!("{}def _style(v):", indent));
        helper_lines.push(format!("{}    if isinstance(v, str): return v", indent));
        helper_lines.push(format!("{}    if isinstance(v, dict): return ';'.join(f'{{k}}:{{x}}' for k, x in v.items())", indent));
        helper_lines.push(format!("{}    return str(v) if v else ''", indent));
    }
    if gen.used_helpers.spread {
        helper_lines.push(format!("{}def _spread(d):", indent));
        helper_lines.push(format!("{}    if not d: return ''", indent));
        helper_lines.push(format!("{}    return ''.join(_attr(k, v) for k, v in d.items())", indent));
    }

    // Insert helpers at the correct position
    if !helper_lines.is_empty() {
        let insert_pos = helpers_insert_pos.min(gen.output.len());
        for (i, line) in helper_lines.into_iter().enumerate() {
            gen.output.insert(insert_pos + i, line);
            gen.line += 1;
        }
    }

    gen.emit_raw(format!("{}return \"\".join(_parts)", indent));
    gen.into_result()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let result = transpile("name: str\n\n<div>Hello {name}</div>\n");
        assert!(result.code.contains("def Template(name: str):"));
        assert!(result.code.contains("_parts.append(f\"\"\""));
        assert!(result.code.contains("return \"\".join(_parts)"));
    }

    #[test]
    fn test_async() {
        let result = transpile("id: int\n\ndata = await fetch(id)\n<div>{data}</div>\n");
        assert!(result.code.contains("async def Template(id: int):"));
    }

    #[test]
    fn test_control_flow() {
        let result = transpile("items: list\n\nfor item in items:\n    <li>{item}</li>\nend\n");
        assert!(result.code.contains("for item in items:"));
        assert!(!result.code.contains("pass")); // Block has content
    }

    #[test]
    fn test_empty_block() {
        let result = transpile("flag: bool\n\nif flag:\nend\n");
        assert!(result.code.contains("if flag:"));
        assert!(result.code.contains("        pass")); // Empty block needs pass (8 spaces = level 2)
    }

    #[test]
    fn test_multiline_html() {
        let result = transpile("count: int\n\nif count == 0:\n    <span>\n        Empty\n    </span>\nend\n");
        assert!(result.code.contains("<span>"));
        assert!(result.code.contains("Empty"));
        assert_eq!(result.code.matches("f\"\"\"").count(), 1);
    }

    #[test]
    fn test_python_detection() {
        let result = transpile("<div>\n    x = 1\n    Hello World\n    print(\"hi\")\n</div>\n");
        assert!(result.code.contains("x = 1"));
        assert!(result.code.contains("print(\"hi\")"));
        assert!(result.code.contains("Hello World"));
    }

    #[test]
    fn test_tstring_escape() {
        let result = transpile("<div>\n    t\"x = 1\"\n</div>\n");
        assert!(result.code.contains("x = 1"));
        assert_eq!(result.code.matches("f\"\"\"").count(), 1);
    }

    #[test]
    fn test_comment() {
        let result = transpile("<div>\n    # comment\n    Hello\n</div>\n");
        assert!(result.code.contains("# comment"));
    }

    #[test]
    fn test_multiline_python() {
        let result = transpile("result = (\n    1 +\n    2\n)\n<div>{result}</div>\n");
        assert!(result.code.contains("result = ("));
        assert!(result.code.contains("1 +"));
    }

    #[test]
    fn test_bare_identifier_is_content() {
        let result = transpile("<div>\n    Hello\n    World\n</div>\n");
        assert!(result.code.contains("Hello"));
        assert_eq!(result.code.matches("f\"\"\"").count(), 1);
    }

    #[test]
    fn test_function_call_is_python() {
        let result = transpile("<div>\n    log(\"msg\")\n    Hello\n</div>\n");
        assert!(result.code.contains("log(\"msg\")"));
    }

    #[test]
    fn test_import_is_python() {
        let result = transpile("from datetime import datetime\n<div>{datetime.now()}</div>\n");
        assert!(result.code.contains("from datetime import datetime"));
    }

    #[test]
    fn test_mixed_content_and_python() {
        let result = transpile("<div>\n    Welcome\n    name = \"Guest\"\n    Hello {name}\n</div>\n");
        assert!(result.code.contains("name = \"Guest\""));
        assert!(result.code.contains("Welcome"));
    }

    #[test]
    fn test_capitalized_words_are_content() {
        let result = transpile("<div>\n    If you see this\n    For example\n</div>\n");
        assert!(result.code.contains("If you see this"));
        assert_eq!(result.code.matches("f\"\"\"").count(), 1);
    }

    #[test]
    fn test_await_is_python() {
        let result = transpile("data = await fetch_data()\n<div>{data}</div>\n");
        assert!(result.code.contains("async def"));
        assert!(result.code.contains("await fetch_data()"));
    }

    #[test]
    fn test_augmented_assignment() {
        let result = transpile("<div>\n    counter += 1\n    Total: {counter}\n</div>\n");
        assert!(result.code.contains("counter += 1"));
    }

    #[test]
    fn test_html_injections() {
        let options = Options { include_injections: true, ..Options::default() };
        let source = "name: str\n\n<div>Hello {name}!</div>\n";
        let result = transpile_with(source, options);
        let inj = result.injections.unwrap();
        assert_eq!(inj.html.len(), 2);
        assert!(source[inj.html[0].start..inj.html[0].end].contains("<div>Hello "));
        assert!(source[inj.html[1].start..inj.html[1].end].contains("!</div>"));
    }

    #[test]
    fn test_html_injections_nested_braces() {
        let options = Options { include_injections: true, ..Options::default() };
        let source = "data: dict\n\n<span>{data['key']}</span>\n";
        let result = transpile_with(source, options);
        let inj = result.injections.unwrap();
        assert_eq!(inj.html.len(), 2);

        let seg1 = &source[inj.html[0].start..inj.html[0].end];
        let seg2 = &source[inj.html[1].start..inj.html[1].end];

        assert_eq!(seg1.trim(), "<span>");
        assert_eq!(seg2.trim(), "</span>");
    }

    #[test]
    fn test_html_injections_multiline() {
        let options = Options { include_injections: true, ..Options::default() };
        let source = "user: dict\n\n<div data-id=\"{user['id']}\">\n  Name: {user['name']}\n</div>\n";
        let result = transpile_with(source, options);
        let inj = result.injections.unwrap();
        assert!(inj.html.len() >= 3);
    }

    #[test]
    fn test_spread_attributes() {
        let result = transpile("attrs: dict\n\n<a {**attrs}>Link</a>\n");
        assert!(result.code.contains("_spread(attrs)"));
    }

    #[test]
    fn test_boolean_attribute() {
        let result = transpile("disabled: bool\n\n<button disabled={disabled}>Click</button>\n");
        assert!(result.code.contains("_attr('disabled', disabled)"));
    }

    #[test]
    fn test_class_attribute() {
        let result = transpile("classes: list\n\n<div class={classes}>Content</div>\n");
        assert!(result.code.contains("_class(classes)"));
    }

    #[test]
    fn test_style_attribute() {
        let result = transpile("styles: dict\n\n<p style={styles}>Text</p>\n");
        assert!(result.code.contains("_style(styles)"));
    }

    #[test]
    fn test_component_self_closing() {
        let result = transpile("<{Button} type=\"submit\" />\n");
        assert!(result.code.contains("_parts.append(Button(type=\"submit\"))"));
    }

    #[test]
    fn test_component_with_slot() {
        let result = transpile("<{Card} title={title}>\n    <p>Content</p>\n</{Card}>\n");
        assert!(result.code.contains("_slot_0_parts = []"));
        assert!(result.code.contains("_slot_0 = \"\".join(_slot_0_parts)"));
        assert!(result.code.contains("Card(title=title, slot=_slot_0)"));
    }

    #[test]
    fn test_component_with_control_flow() {
        let result = transpile("<{List}>\n    for i in items:\n        <li>{i}</li>\n    end\n</{List}>\n");
        assert!(result.code.contains("for i in items:"));
        assert!(result.code.contains("_slot_0_parts.append"));
        assert!(result.code.contains("List(slot=_slot_0)"));
    }

    #[test]
    fn test_nested_components() {
        let result = transpile("<{Outer}>\n    <{Inner} />\n</{Outer}>\n");
        assert!(result.code.contains("_slot_0_parts.append(Inner())"));
        assert!(result.code.contains("Outer(slot=_slot_0)"));
    }

    #[test]
    fn test_component_injections() {
        let options = Options { include_injections: true, ..Options::default() };
        let source = "<{Card} title={title} />\n";
        let result = transpile_with(source, options);
        let inj = result.injections.unwrap();

        // Should have injections for: function def, Card, title
        assert!(inj.python.len() >= 3);

        // Check component name injection (Card at position 2)
        let card_inj = inj.python.iter().find(|p| p.start == 2).unwrap();
        assert_eq!(&source[card_inj.start..card_inj.end], "Card");

        // Check attribute expression injection (title at position 15)
        let title_inj = inj.python.iter().find(|p| p.start == 15).unwrap();
        assert_eq!(&source[title_inj.start..title_inj.end], "title");
    }
}
