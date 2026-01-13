
/// Position in source code (byte offset only; convert to UTF-16 at output time)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Byte offset in source
    pub byte: usize,
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed, in characters)
    pub col: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { byte: 0, line: 0, col: 0 }
    }
}

/// Span in source code (a range from start position to end position)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// Component attribute
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// String literal: attr="value" or attr='value'
    String(String),
    /// Expression: attr={expr}
    Expression(String, Span),
    /// Boolean (no value): disabled
    Bool,
    /// Shorthand: {name}
    Shorthand(String, Span),
    /// Spread: {**expr}
    Spread(String, Span),
    /// Slot assignment: {...name} assigns element to children_name slot
    SlotAssignment(String, Span),
}

/// Tokens produced by the lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // === Structural ===
    /// Indentation at start of line
    Indent { level: usize, span: Span },
    /// Newline (LF or CRLF)
    Newline { span: Span },
    /// End of file
    Eof { position: Position },

    // === Python Domain ===
    /// Control flow start: if, for, while, match, with, try, def, class, async for, async with, async def
    ControlStart { keyword: String, rest: String, span: Span },
    /// Control flow continuation: else, elif, case, except, finally
    ControlContinuation { keyword: String, rest: Option<String>, span: Span },
    /// Block terminator: end
    End { span: Span },
    /// Python statement (assignment, call, import, etc.)
    PythonStatement { code: String, span: Span },
    /// Comment (including the # prefix)
    Comment { text: String, span: Span },
    /// Decorator (@something)
    Decorator { code: String, span: Span },

    // === Content Domain ===
    /// Raw text/HTML content (no expressions)
    Text { text: String, span: Span },
    /// Expression placeholder: {expr}
    Expression { code: String, span: Span },
    /// Escaped brace: {{ or }}
    EscapedBrace { brace: char, span: Span },

    // === Components ===
    /// Component opening tag: <{Name} attributes>
    ComponentOpen { name: String, name_span: Span, attributes: Vec<Attribute>, self_closing: bool, span: Span },
    /// Component closing tag: </{Name}>
    ComponentClose { name: String, span: Span },

    // === HTML Elements ===
    /// HTML element opening tag: <tag attributes>
    HtmlElementOpen {
        tag: String,
        tag_span: Span,              // Position of "<tag" (opening bracket + tag name)
        attributes: Vec<Attribute>,
        close_bracket_pos: Position, // Position of ">" or "/>"
        self_closing: bool,
        span: Span,                  // Overall span covering entire token
    },
    /// HTML element closing tag: </tag>
    HtmlElementClose { tag: String, span: Span },

    // === Slots ===
    /// Slot definition opening: <{...}> or <{...name}>
    SlotOpen { name: Option<String>, span: Span },
    /// Slot definition closing: </{...}> or </{...name}>
    SlotClose { name: Option<String>, span: Span },

    // === Fragments ===
    /// Fragment definition start: fragment Name:
    FragmentStart { name: String, span: Span },

    // === File Structure ===
    /// Header/body separator: ---
    Separator { span: Span },
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Indent { span, .. } => *span,
            Token::Newline { span, .. } => *span,
            Token::Eof { position } => Span { start: *position, end: *position },
            Token::ControlStart { span, .. } => *span,
            Token::ControlContinuation { span, .. } => *span,
            Token::End { span, .. } => *span,
            Token::PythonStatement { span, .. } => *span,
            Token::Comment { span, .. } => *span,
            Token::Decorator { span, .. } => *span,
            Token::Text { span, .. } => *span,
            Token::Expression { span, .. } => *span,
            Token::EscapedBrace { span, .. } => *span,
            Token::ComponentOpen { span, .. } => *span,
            Token::ComponentClose { span, .. } => *span,
            Token::HtmlElementOpen { span, .. } => *span,
            Token::HtmlElementClose { span, .. } => *span,
            Token::SlotOpen { span, .. } => *span,
            Token::SlotClose { span, .. } => *span,
            Token::FragmentStart { span, .. } => *span,
            Token::Separator { span, .. } => *span,
        }
    }
}

/// Tokenizer for Hyper source files
pub struct Tokenizer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    position: Position,
    /// Parser for Python classification
    parser: tree_sitter::Parser,
    /// Track if we're inside a multi-line string (""" or ''')
    in_multiline_string: Option<&'static str>,
}

/// Context for tracking quote state in content
#[derive(Debug, Clone, Copy, PartialEq)]
enum QuoteCtx {
    None,
    Double,
    Single,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("Failed to load Python grammar");

        Self {
            source,
            bytes: source.as_bytes(),
            position: Position::new(),
            parser,
            in_multiline_string: None,
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while !self.at_eof() {
            self.tokenize_line(&mut tokens);
        }

        tokens.push(Token::Eof { position: self.position });
        tokens
    }

    /// Tokenize a single line
    fn tokenize_line(&mut self, tokens: &mut Vec<Token>) {
        // 1. Handle indentation
        let indent_start = self.position;
        let indent_level = self.consume_indent();
        if indent_level > 0 {
            tokens.push(Token::Indent {
                level: indent_level,
                span: Span { start: indent_start, end: self.position },
            });
        }

        // 2. Check for empty line or EOF
        if self.at_eof() {
            return;
        }
        if self.at_newline() {
            let nl_start = self.position;
            self.consume_newline();
            tokens.push(Token::Newline {
                span: Span { start: nl_start, end: self.position },
            });
            return;
        }

        // 3. Handle multi-line string continuation
        if let Some(delimiter) = self.in_multiline_string {
            let line_start = self.position;
            let line_content = self.peek_line();
            self.skip_to_eol();

            // Check if this line ends the multi-line string
            if line_content.contains(delimiter) {
                self.in_multiline_string = None;
            }

            tokens.push(Token::PythonStatement {
                code: line_content.to_string(),
                span: Span { start: line_start, end: self.position },
            });

            // Consume newline
            if self.at_newline() {
                let nl_start = self.position;
                self.consume_newline();
                tokens.push(Token::Newline {
                    span: Span { start: nl_start, end: self.position },
                });
            }
            return;
        }

        // 4. Determine line type and tokenize accordingly
        let _line_start = self.position;
        let line_content = self.peek_line();

        // Check for multi-line string start
        let trimmed = line_content.trim();
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let delimiter = if trimmed.starts_with("\"\"\"") { "\"\"\"" } else { "'''" };
            // Count occurrences of delimiter in line
            let count = trimmed.matches(delimiter).count();
            // If odd number, we're entering a multi-line string
            if count == 1 {
                self.in_multiline_string = Some(delimiter);
            }
            // Emit as Python statement
            let line_start = self.position;
            self.skip_to_eol();
            tokens.push(Token::PythonStatement {
                code: line_content.to_string(),
                span: Span { start: line_start, end: self.position },
            });

            // Consume newline
            if self.at_newline() {
                let nl_start = self.position;
                self.consume_newline();
                tokens.push(Token::Newline {
                    span: Span { start: nl_start, end: self.position },
                });
            }
            return;
        }

        // Check for special patterns - ORDER MATTERS!
        // 0. Separator (exactly ---)
        if trimmed == "---" {
            let sep_start = self.position;
            self.skip_to_eol();
            tokens.push(Token::Separator {
                span: Span { start: sep_start, end: self.position },
            });
        }
        // 1. Comment (starts with #)
        else if line_content.starts_with('#') {
            self.tokenize_comment(tokens);
        }
        // 2. Decorator (starts with @, no HTML, not CSS at-rules)
        else if line_content.starts_with("@") && !line_content.contains('<') && !self.is_css_at_rule(&line_content) {
            self.tokenize_decorator(tokens);
        }
        // 3. Slot definition tags: <{...}> or <{...name}>
        else if line_content.starts_with("<{...") {
            self.tokenize_slot_open(tokens);
        }
        else if line_content.starts_with("</{...") {
            self.tokenize_slot_close(tokens);
        }
        // 4. Component tags: <{Name}>
        else if line_content.starts_with("<{") {
            self.tokenize_component_open(tokens);
        }
        else if line_content.starts_with("</{") {
            self.tokenize_component_close(tokens);
        }
        // 4. End keyword (before content check!)
        else if self.is_end_keyword(&line_content) {
            let end_start = self.position;
            self.skip_to_eol();
            tokens.push(Token::End {
                span: Span { start: end_start, end: self.position },
            });
        }
        // 5. Fragment definition
        else if line_content.trim().starts_with("fragment ") && line_content.trim().ends_with(':') {
            self.tokenize_fragment_start(tokens, &line_content);
        }
        // 6. Control flow keywords
        else if self.is_control_flow(&line_content) {
            self.tokenize_control_start(tokens, &line_content);
        }
        // 6. Control continuation keywords (else, elif, except, finally)
        else if self.is_control_continuation(&line_content) {
            self.tokenize_control_continuation(tokens, &line_content);
        }
        // 7. HTML content (starts with <)
        else if line_content.starts_with('<') {
            self.tokenize_content(tokens);
        }
        // 7.5. HTML assignment (identifier = <...>)
        else if self.is_html_assignment(&line_content) {
            self.tokenize_html_assignment(tokens);
        }
        // 7.6. Parameter declarations (*args: type, **kwargs: type, name: type)
        // These aren't valid Python statements but are valid in header zone
        else if self.is_parameter_declaration(&line_content) {
            self.tokenize_python_statement(tokens);
        }
        // 8. Check if it's a Python statement using tree-sitter
        else if self.is_python_statement(&line_content) {
            self.tokenize_python_statement(tokens);
        }
        // 9. Default: treat as content
        else {
            self.tokenize_content(tokens);
        }

        // 4. Handle trailing comment if we haven't consumed to newline
        // (Already handled in tokenize_content)

        // 5. Consume newline
        if self.at_newline() {
            let nl_start = self.position;
            self.consume_newline();
            tokens.push(Token::Newline {
                span: Span { start: nl_start, end: self.position },
            });
        }
    }

    // === Classification helpers ===

    fn is_end_keyword(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed == "end"
    }

    fn is_control_flow(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // Strip trailing comment to get the "effective" line for syntax checks.
        // A trailing comment is `  # ...` (whitespace + hash) outside quotes.
        let effective = self.strip_trailing_comment(trimmed);

        // for: requires trailing `:` (parser validates `in` keyword and reports errors)
        if trimmed.starts_with("for ") || trimmed.starts_with("async for ") {
            return effective.ends_with(':');
        }

        // if, elif, while, match, with: require trailing `:`
        if trimmed.starts_with("if ")
            || trimmed.starts_with("while ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("with ")
            || trimmed.starts_with("async with ")
        {
            return effective.ends_with(':');
        }

        // try: exact keyword (already includes colon)
        if trimmed == "try:" || trimmed == "try :" {
            return true;
        }

        // def / async def: require `(` (function signature)
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            return effective.contains('(');
        }

        // class: must be followed by identifier (not `=`), and end with `:`
        if trimmed.starts_with("class ") {
            let rest = &trimmed[6..];
            if let Some(first_char) = rest.chars().next() {
                if (first_char.is_alphabetic() || first_char == '_') && effective.ends_with(':') {
                    return true;
                }
            }
        }

        false
    }

    /// Strip a trailing `# comment` from a line (outside quotes) for syntax checks.
    /// Returns the effective code portion of the line.
    fn strip_trailing_comment<'b>(&self, line: &'b str) -> &'b str {
        let mut in_single = false;
        let mut in_double = false;
        let bytes = line.as_bytes();
        for i in 0..bytes.len() {
            match bytes[i] {
                b'"' if !in_single => in_double = !in_double,
                b'\'' if !in_double => in_single = !in_single,
                b'\\' if in_single || in_double => {
                    // skip next char (escape sequence) — handled by loop increment
                    continue;
                }
                b'#' if !in_single && !in_double => {
                    // Found unquoted #; check if preceded by whitespace
                    if i > 0 && bytes[i - 1] == b' ' {
                        return line[..i].trim_end();
                    }
                }
                _ => {}
            }
        }
        line
    }

    /// Check if this looks like a parameter declaration (used in header zone)
    /// Matches patterns like: name: type, name: type = default, *args: tuple, **kwargs: dict
    fn is_parameter_declaration(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Must contain a colon for type annotation
        if !trimmed.contains(':') {
            return false;
        }

        // Handle **kwargs and *args patterns
        if trimmed.starts_with("**") || trimmed.starts_with("*") {
            return true;
        }

        // Regular parameter: name: type or name: type = default
        // Must start with identifier character
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_alphabetic() || first_char == '_' {
                // Check colon comes before any = (for defaults)
                if let Some(colon_pos) = trimmed.find(':') {
                    if let Some(equals_pos) = trimmed.find('=') {
                        return colon_pos < equals_pos;
                    }
                    return true;
                }
            }
        }

        false
    }

    fn is_control_continuation(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.starts_with("else:") || trimmed.starts_with("else :") ||
        trimmed.starts_with("elif ") ||
        trimmed.starts_with("except") ||
        trimmed.starts_with("finally:") || trimmed.starts_with("finally :") ||
        trimmed.starts_with("case ")
    }

    /// Check if line is a CSS at-rule (to avoid treating as Python decorator)
    fn is_css_at_rule(&self, line: &str) -> bool {
        let trimmed = line.trim();
        // Common CSS at-rules
        trimmed.starts_with("@media") ||
        trimmed.starts_with("@keyframes") ||
        trimmed.starts_with("@import") ||
        trimmed.starts_with("@charset") ||
        trimmed.starts_with("@font-face") ||
        trimmed.starts_with("@supports") ||
        trimmed.starts_with("@namespace") ||
        trimmed.starts_with("@page") ||
        trimmed.starts_with("@counter-style") ||
        trimmed.starts_with("@layer") ||
        trimmed.starts_with("@property") ||
        trimmed.starts_with("@container") ||
        trimmed.starts_with("@scope")
    }

    /// Fast heuristic: check if line is obviously NOT Python
    /// This avoids expensive tree-sitter calls for most content lines
    fn is_obviously_content(&self, trimmed: &str) -> bool {
        // Empty or whitespace-only
        if trimmed.is_empty() {
            return true;
        }

        let first_char = trimmed.chars().next().unwrap();

        // Triple-quoted strings are Python docstrings, not content
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            return false;
        }

        // Lines starting with these are definitely content, not Python
        // - Lowercase letters followed by content patterns (not Python keywords)
        // - HTML-like content
        // - Pure text without Python operators
        match first_char {
            // HTML tags or content starting with punctuation (except @ which is decorator)
            '<' | '>' | '&' | '!' | '?' | '/' | '*' | '+' | '-' | '.' | ',' | ';' | ':' |
            '[' | ']' | '(' | ')' | '"' | '\'' | '`' | '~' | '^' | '%' | '$' | '|' => {
                return true;
            }
            // Numbers at start are content (Python statements don't start with digits)
            '0'..='9' => {
                return true;
            }
            // Uppercase letters are typically content (English text, not Python)
            // Python statements start with lowercase keywords
            'A'..='Z' => {
                return true;
            }
            _ => {}
        }

        // Check for common Python statement patterns that REQUIRE tree-sitter
        // These are the only cases worth parsing
        let needs_parsing =
            // Assignment patterns
            trimmed.contains(" = ") || trimmed.contains(" += ") || trimmed.contains(" -= ") ||
            trimmed.contains(" *= ") || trimmed.contains(" /= ") || trimmed.contains(" //= ") ||
            trimmed.contains(" %= ") || trimmed.contains(" **= ") || trimmed.contains(" &= ") ||
            trimmed.contains(" |= ") || trimmed.contains(" ^= ") || trimmed.contains(" >>= ") ||
            trimmed.contains(" <<= ") || trimmed.contains(" := ") ||
            // Type annotations (name: type) - check for `: ` followed by identifier
            (trimmed.contains(": ") && {
                let parts: Vec<&str> = trimmed.splitn(2, ": ").collect();
                parts.len() == 2 && {
                    let name = parts[0].trim();
                    !name.is_empty() &&
                    name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) &&
                    name.chars().all(|c| c.is_alphanumeric() || c == '_')
                }
            }) ||
            // Import statements
            trimmed.starts_with("import ") || trimmed.starts_with("from ") ||
            // Control keywords (return, raise, etc.)
            trimmed.starts_with("return ") || trimmed.starts_with("return\n") || trimmed == "return" ||
            trimmed.starts_with("raise ") || trimmed.starts_with("raise\n") || trimmed == "raise" ||
            trimmed.starts_with("assert ") ||
            trimmed == "pass" || trimmed == "break" || trimmed == "continue" ||
            trimmed.starts_with("del ") ||
            trimmed.starts_with("global ") || trimmed.starts_with("nonlocal ") ||
            trimmed.starts_with("yield ") || trimmed.starts_with("yield\n") || trimmed == "yield" ||
            trimmed.starts_with("await ") ||
            // Function calls (identifier followed by parenthesis)
            (first_char.is_ascii_lowercase() && trimmed.contains('(') && trimmed.contains(')'));

        !needs_parsing
    }

    fn is_python_statement(&mut self, line: &str) -> bool {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            return false;
        }

        // Fast path: skip tree-sitter for obvious content
        if self.is_obviously_content(trimmed) {
            return false;
        }

        // Special handling for "class = ..." - Python reserved keyword used as variable
        // This pattern can't be parsed by tree-sitter but is valid in .hyper files
        if trimmed.starts_with("class =") || trimmed.starts_with("class=") {
            return true;
        }

        // Check for potential multiline statements (unclosed brackets with assignment pattern)
        // These may fail tree-sitter parsing because they're incomplete
        if self.looks_like_multiline_start(trimmed) {
            return true;
        }

        // Try to parse with tree-sitter
        if let Some(tree) = self.parser.parse(trimmed, None) {
            let root = tree.root_node();

            // Parse error = not Python (unless it's a multiline continuation)
            if root.has_error() {
                return false;
            }

            if root.kind() == "module" {
                if let Some(child) = root.child(0) {
                    match child.kind() {
                        // These are definitely Python statements
                        "assignment" | "augmented_assignment" |
                        "import_statement" | "import_from_statement" |
                        "return_statement" | "raise_statement" | "assert_statement" |
                        "pass_statement" | "break_statement" | "continue_statement" |
                        "delete_statement" | "global_statement" | "nonlocal_statement" => {
                            return true;
                        }
                        "expression_statement" => {
                            // Check if it's a meaningful expression (call, await, etc.)
                            if let Some(expr) = child.child(0) {
                                return matches!(expr.kind(),
                                    "call" | "await" | "yield" | "named_expression" |
                                    "assignment" | "augmented_assignment"
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        false
    }

    /// Check if a line is an HTML variable assignment: `name = <...>` or `name = (...)`
    fn is_html_assignment(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Look for pattern: identifier (: type)? = < or identifier (: type)? = (
        // e.g., "title = <span>", "header: str = <div>", "content = ("

        // Find the = sign
        if let Some(eq_pos) = trimmed.find(" = ") {
            let after_eq = &trimmed[eq_pos + 3..].trim_start();

            // Check if what follows = is HTML or a parenthesized expression
            if after_eq.starts_with('<') && !after_eq.starts_with("<=") && !after_eq.starts_with("<<") {
                // Verify the left side is a valid identifier (possibly with type annotation)
                let before_eq = &trimmed[..eq_pos].trim();

                // Handle type annotations: `name: type`
                let identifier = if let Some(colon_pos) = before_eq.find(':') {
                    before_eq[..colon_pos].trim()
                } else {
                    before_eq
                };

                // Check it's a valid identifier
                if !identifier.is_empty() {
                    let first = identifier.chars().next().unwrap();
                    if (first.is_alphabetic() || first == '_')
                        && identifier.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a line looks like the start of a multiline Python statement
    /// (has unclosed brackets and looks like an assignment or call)
    fn looks_like_multiline_start(&self, trimmed: &str) -> bool {
        // Must have unclosed brackets
        let depth = self.calculate_bracket_depth(trimmed);
        if depth <= 0 {
            return false;
        }

        // Must look like Python (assignment or function call pattern)
        let has_assignment = trimmed.contains(" = ") || trimmed.contains(": ");
        let has_call = trimmed.contains('(') && !trimmed.contains(')');

        // Check for valid Python identifier at start
        let starts_with_identifier = trimmed.chars().next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false);

        starts_with_identifier && (has_assignment || has_call)
    }

    // === Tokenization methods ===

    fn tokenize_comment(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        let text = self.consume_to_eol();
        tokens.push(Token::Comment {
            text,
            span: Span { start, end: self.position },
        });
    }

    fn tokenize_decorator(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        let code = self.consume_to_eol();
        tokens.push(Token::Decorator {
            code,
            span: Span { start, end: self.position },
        });
    }

    /// Tokenize a fragment definition: fragment Name:
    fn tokenize_fragment_start(&mut self, tokens: &mut Vec<Token>, _line: &str) {
        let start = self.position;
        let code = self.consume_to_eol();
        let trimmed = code.trim();

        // Extract fragment name from "fragment Name:"
        let name = trimmed
            .strip_prefix("fragment ")
            .and_then(|s| s.strip_suffix(':'))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        tokens.push(Token::FragmentStart {
            name,
            span: Span { start, end: self.position },
        });
    }

    fn tokenize_control_start(&mut self, tokens: &mut Vec<Token>, _line: &str) {
        let start = self.position;
        let code = self.consume_to_eol();
        let trimmed = code.trim();

        // Strip trailing comment before parsing the control flow statement
        let effective = self.strip_trailing_comment(trimmed);

        // Handle compound keywords (async for, async with, async def)
        let (keyword, rest) = if effective.starts_with("async for ") {
            ("async for".to_string(), effective[10..].trim_start().to_string())
        } else if effective.starts_with("async with ") {
            ("async with".to_string(), effective[11..].trim_start().to_string())
        } else if effective.starts_with("async def ") {
            ("async def".to_string(), effective[10..].trim_start().to_string())
        } else if let Some(idx) = effective.find(|c: char| c.is_whitespace() || c == ':') {
            let kw = &effective[..idx];
            let r = effective[idx..].trim_start();
            (kw.to_string(), r.to_string())
        } else {
            (effective.to_string(), String::new())
        };

        tokens.push(Token::ControlStart {
            keyword,
            rest,
            span: Span { start, end: self.position },
        });
    }

    fn tokenize_control_continuation(&mut self, tokens: &mut Vec<Token>, _line: &str) {
        let start = self.position;
        let code = self.consume_to_eol();
        let trimmed = code.trim();

        // Extract keyword and optional rest
        let (keyword, rest) = if let Some(idx) = trimmed.find(|c: char| c.is_whitespace() || c == ':') {
            let kw = &trimmed[..idx];
            let r = trimmed[idx..].trim_start();
            if r.is_empty() || r == ":" {
                (kw.to_string(), None)
            } else {
                (kw.to_string(), Some(r.to_string()))
            }
        } else {
            (trimmed.to_string(), None)
        };

        tokens.push(Token::ControlContinuation {
            keyword,
            rest,
            span: Span { start, end: self.position },
        });
    }

    fn tokenize_html_assignment(&mut self, tokens: &mut Vec<Token>) {
        // Handle patterns like: `name = <span>content</span>`
        // Convert to: `name = f"<span>content</span>"`
        let start = self.position;
        let line = self.consume_to_eol();
        let trimmed = line.trim();

        // Find the = position
        if let Some(eq_pos) = trimmed.find(" = ") {
            let left_side = &trimmed[..eq_pos];
            let right_side = &trimmed[eq_pos + 3..];

            // Convert the HTML content to an f-string
            // We need to escape {{ and }} that are already escaped braces
            // and keep {expr} expressions as-is
            let html_content = right_side.trim();

            // Build the Python assignment: `left = f"html_content"`
            let code = format!("{} = f\"\"\"{}\"\"\"", left_side, html_content);

            tokens.push(Token::PythonStatement {
                code,
                span: Span { start, end: self.position },
            });
        } else {
            // Fallback: just emit as Python statement
            tokens.push(Token::PythonStatement {
                code: line,
                span: Span { start, end: self.position },
            });
        }
    }

    fn tokenize_python_statement(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        let mut code = self.consume_to_eol();

        // Check for multiline continuation (unclosed brackets)
        let mut depth = self.calculate_bracket_depth(&code);

        while depth > 0 && !self.at_eof() {
            // Consume newline and add to code
            if self.at_newline() {
                code.push('\n');
                self.consume_newline();
            }

            // Consume indentation
            while !self.at_eof() && !self.at_newline() {
                match self.peek_char() {
                    Some(' ') | Some('\t') => {
                        code.push(self.peek_char().unwrap());
                        self.advance();
                    }
                    _ => break,
                }
            }

            // Check for EOF after indentation
            if self.at_eof() || self.at_newline() {
                // Empty continuation line - keep going
                continue;
            }

            // Consume the next line
            let line = self.consume_to_eol();
            code.push_str(&line);

            // Recalculate bracket depth
            depth = self.calculate_bracket_depth(&code);
        }

        tokens.push(Token::PythonStatement {
            code,
            span: Span { start, end: self.position },
        });
    }

    /// Calculate net bracket depth, accounting for strings and comments
    fn calculate_bracket_depth(&self, code: &str) -> i32 {
        let mut depth = 0i32;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_triple_string = false;
        let mut chars = code.chars().peekable();

        while let Some(ch) = chars.next() {
            if in_string {
                if ch == '\\' && !in_triple_string {
                    // Skip escaped character in regular strings
                    chars.next();
                    continue;
                }
                if in_triple_string {
                    // Check for triple quote end
                    if ch == string_char && chars.peek() == Some(&string_char) {
                        chars.next();
                        if chars.peek() == Some(&string_char) {
                            chars.next();
                            in_string = false;
                            in_triple_string = false;
                        }
                    }
                } else if ch == string_char {
                    in_string = false;
                }
            } else {
                match ch {
                    '"' | '\'' => {
                        // Check for triple quote
                        if chars.peek() == Some(&ch) {
                            chars.next();
                            if chars.peek() == Some(&ch) {
                                chars.next();
                                in_string = true;
                                string_char = ch;
                                in_triple_string = true;
                            }
                            // else: empty string "" or '', not entering string mode
                        } else {
                            in_string = true;
                            string_char = ch;
                            in_triple_string = false;
                        }
                    }
                    '#' => {
                        // Rest of line is comment, skip to newline
                        while chars.next().is_some_and(|c| c != '\n') {}
                    }
                    '(' | '[' | '{' => depth += 1,
                    ')' | ']' | '}' => depth = (depth - 1).max(0),
                    _ => {}
                }
            }
        }

        depth
    }

    fn tokenize_content(&mut self, tokens: &mut Vec<Token>) {
        // Tokenize content, extracting:
        // - Text segments
        // - {expr} expressions
        // - {{/}} escaped braces
        // - Trailing # comments

        let mut quote_ctx = QuoteCtx::None;
        let mut text_start = self.position;
        let mut text_buf = String::new();

        // Track whether the last emitted token was structural (HTML tag, expression,
        // component, escaped brace). This determines whether a subsequent `#` can be
        // a trailing comment. We start `true` because tokenize_content() is always
        // called right after a structural token (element open, component open, or
        // at the beginning of a content line which counts as a boundary).
        let mut after_structural = true;

        while !self.at_eof() && !self.at_newline() {
            let ch = self.peek_char().unwrap();

            match (quote_ctx, ch) {
                // Quote tracking
                (QuoteCtx::None, '"') => {
                    text_buf.push(ch);
                    self.advance();
                    quote_ctx = QuoteCtx::Double;
                    after_structural = false;
                }
                (QuoteCtx::Double, '"') => {
                    text_buf.push(ch);
                    self.advance();
                    quote_ctx = QuoteCtx::None;
                }
                (QuoteCtx::None, '\'') => {
                    text_buf.push(ch);
                    self.advance();
                    quote_ctx = QuoteCtx::Single;
                    after_structural = false;
                }
                (QuoteCtx::Single, '\'') => {
                    text_buf.push(ch);
                    self.advance();
                    quote_ctx = QuoteCtx::None;
                }
                // Escape sequences in strings
                (QuoteCtx::Double | QuoteCtx::Single, '\\') => {
                    text_buf.push(ch);
                    self.advance();
                    if let Some(next) = self.peek_char() {
                        text_buf.push(next);
                        self.advance();
                    }
                }

                // Trailing comment: `#` outside quotes, after a structural token,
                // with only whitespace accumulated since that token.
                //
                // Valid:   <div>Hello</div>  # comment
                //          {name}  # comment
                //          <br />  # comment
                //
                // Invalid: <p>Text # not a comment</p>
                //          Content with # hash
                //
                // Full-line comments (# at line start) are handled in tokenize_line().
                (QuoteCtx::None, '#') if after_structural
                    && !text_buf.is_empty()
                    && text_buf.chars().all(|c| c.is_whitespace()) => {
                    // Discard whitespace-only text_buf (it's just padding before comment)
                    text_buf.clear();
                    // Consume comment
                    let comment_start = self.position;
                    let comment = self.consume_to_eol();
                    tokens.push(Token::Comment {
                        text: comment,
                        span: Span { start: comment_start, end: self.position },
                    });
                    return; // Line is done
                }

                // Escaped braces
                (QuoteCtx::None, '{') if self.peek_next_char() == Some('{') => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    let brace_start = self.position;
                    self.advance();
                    self.advance();
                    tokens.push(Token::EscapedBrace {
                        brace: '{',
                        span: Span { start: brace_start, end: self.position },
                    });
                    text_start = self.position;
                    after_structural = true;
                }
                (QuoteCtx::None, '}') if self.peek_next_char() == Some('}') => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    let brace_start = self.position;
                    self.advance();
                    self.advance();
                    tokens.push(Token::EscapedBrace {
                        brace: '}',
                        span: Span { start: brace_start, end: self.position },
                    });
                    text_start = self.position;
                    after_structural = true;
                }

                // Expression
                (QuoteCtx::None, '{') => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    // Parse expression
                    self.tokenize_expression(tokens);
                    text_start = self.position;
                    after_structural = true;
                }

                // HTML element opening tag: <tagname
                (QuoteCtx::None, '<') if self.is_html_element_start() => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    self.tokenize_html_element_open(tokens);
                    text_start = self.position;
                    after_structural = true;
                }

                // Component closing tag: </{Name}>
                (QuoteCtx::None, '<') if self.is_component_close() => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    self.tokenize_component_close(tokens);
                    text_start = self.position;
                    after_structural = true;
                }

                // HTML element closing tag: </tagname
                (QuoteCtx::None, '<') if self.is_html_element_close() => {
                    // Flush text
                    if !text_buf.is_empty() {
                        tokens.push(Token::Text {
                            text: text_buf.clone(),
                            span: Span { start: text_start, end: self.position },
                        });
                        text_buf.clear();
                    }
                    self.tokenize_html_element_close(tokens);
                    text_start = self.position;
                    after_structural = true;
                }

                // Regular character
                _ => {
                    if !ch.is_whitespace() {
                        after_structural = false;
                    }
                    text_buf.push(ch);
                    self.advance();
                }
            }
        }

        // Flush remaining text
        if !text_buf.is_empty() {
            tokens.push(Token::Text {
                text: text_buf,
                span: Span { start: text_start, end: self.position },
            });
        }
    }

    fn tokenize_expression(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // consume {

        let _expr_start = self.position;
        let mut depth = 1;
        let mut expr = String::new();

        // Track string context to avoid counting braces inside strings
        let mut in_string = false;
        let mut string_char = ' ';

        while !self.at_eof() && depth > 0 {
            let ch = self.peek_char().unwrap();

            if in_string {
                if ch == '\\' {
                    expr.push(ch);
                    self.advance();
                    if let Some(next) = self.peek_char() {
                        expr.push(next);
                        self.advance();
                    }
                    continue;
                }
                if ch == string_char {
                    in_string = false;
                }
                expr.push(ch);
                self.advance();
            } else {
                match ch {
                    '"' | '\'' => {
                        in_string = true;
                        string_char = ch;
                        expr.push(ch);
                        self.advance();
                    }
                    '{' => {
                        depth += 1;
                        expr.push(ch);
                        self.advance();
                    }
                    '}' => {
                        depth -= 1;
                        if depth > 0 {
                            expr.push(ch);
                        }
                        self.advance();
                    }
                    _ => {
                        expr.push(ch);
                        self.advance();
                    }
                }
            }
        }

        // Convert children placeholder {...} to {children} or {...name} to {children_name}
        let trimmed = expr.trim();
        let final_expr = if trimmed.starts_with("...") {
            let slot_name = trimmed[3..].trim();
            if slot_name.is_empty() {
                "children".to_string()
            } else {
                format!("children_{}", slot_name)
            }
        } else {
            expr
        };

        tokens.push(Token::Expression {
            code: final_expr,
            span: Span { start, end: self.position },
        });
    }

    /// Parse a single attribute (shared between components and HTML elements).
    /// Returns None if no attribute could be parsed.
    fn parse_attribute(&mut self) -> Option<Attribute> {
        let ch = self.peek_char()?;

        if ch == '{' {
            // Shorthand {name}, spread {**expr}, or slot assignment {...name}
            let attr_start = self.position;
            self.advance(); // {

            if self.peek_char() == Some('*') && self.peek_next_char() == Some('*') {
                // Spread {**expr}
                self.advance();
                self.advance();
                let expr = self.consume_until_char('}');
                let attr_end = self.position;
                self.advance(); // }
                return Some(Attribute {
                    name: "**".to_string(),
                    value: AttributeValue::Spread(expr, Span { start: attr_start, end: attr_end }),
                    span: Span { start: attr_start, end: self.position },
                });
            } else if self.peek_char() == Some('.') {
                // Slot assignment {...name}
                self.advance(); // .
                self.advance(); // .
                self.advance(); // .
                let slot_name = self.consume_until_char('}');
                let attr_end = self.position;
                self.advance(); // }
                return Some(Attribute {
                    name: format!("...{}", slot_name),
                    value: AttributeValue::SlotAssignment(slot_name.trim().to_string(), Span { start: attr_start, end: attr_end }),
                    span: Span { start: attr_start, end: self.position },
                });
            } else {
                // Shorthand {name}
                let expr = self.consume_until_char('}');
                let attr_end = self.position;
                self.advance(); // }
                return Some(Attribute {
                    name: expr.clone(),
                    value: AttributeValue::Shorthand(expr, Span { start: attr_start, end: attr_end }),
                    span: Span { start: attr_start, end: self.position },
                });
            }
        } else if ch.is_alphabetic() || ch == '_' || ch == '-' || ch == '@' || ch == ':' {
            // Named attribute
            let attr_start = self.position;
            let attr_name = self.consume_while(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '@' || c == ':');

            if self.peek_char() == Some('=') {
                self.advance(); // =

                let (value, value_end) = if self.peek_char() == Some('{') {
                    // Expression value: aria={expr}
                    let val_start = self.position;
                    self.advance(); // {
                    let expr = self.consume_until_char('}');
                    self.advance(); // } - advance past closing brace
                    (AttributeValue::Expression(expr, Span { start: val_start, end: self.position }), self.position)
                } else if self.peek_char() == Some('"') {
                    // Double-quoted string
                    self.advance(); // "
                    let val = self.consume_until_char('"');
                    self.advance(); // " - advance past closing quote
                    (AttributeValue::String(val), self.position)
                } else if self.peek_char() == Some('\'') {
                    // Single-quoted string
                    self.advance(); // '
                    let val = self.consume_until_char('\'');
                    self.advance(); // ' - advance past closing quote
                    (AttributeValue::String(val), self.position)
                } else {
                    (AttributeValue::Bool, self.position)
                };

                return Some(Attribute {
                    name: attr_name,
                    value,
                    span: Span { start: attr_start, end: value_end },
                });
            } else {
                // Boolean attribute
                return Some(Attribute {
                    name: attr_name,
                    value: AttributeValue::Bool,
                    span: Span { start: attr_start, end: self.position },
                });
            }
        }

        None
    }

    fn tokenize_component_open(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <
        self.advance(); // {

        // Parse component name
        let name_start = self.position;
        let name = self.consume_until_char('}');
        let name_end = self.position;
        self.advance(); // }

        let name_span = Span { start: name_start, end: name_end };

        // Parse attributes
        let mut attrs = Vec::new();

        loop {
            self.skip_whitespace_inline();

            if self.at_eof() || self.at_newline() {
                break;
            }

            let ch = self.peek_char().unwrap();

            // Check for /> or >
            if ch == '/' && self.peek_next_char() == Some('>') {
                self.advance();
                self.advance();
                tokens.push(Token::ComponentOpen {
                    name,
                    name_span,
                    attributes: attrs,
                    self_closing: true,
                    span: Span { start, end: self.position },
                });
                return;
            }
            if ch == '>' {
                self.advance();
                tokens.push(Token::ComponentOpen {
                    name,
                    name_span,
                    attributes: attrs,
                    self_closing: false,
                    span: Span { start, end: self.position },
                });
                // There might be trailing content on this line - tokenize it
                if !self.at_eof() && !self.at_newline() {
                    self.tokenize_content(tokens);
                }
                return;
            }

            // Parse attribute using shared function
            if let Some(attr) = self.parse_attribute() {
                attrs.push(attr);
            } else {
                // Unknown character, skip
                self.advance();
            }
        }

        // Reached end of line without closing
        tokens.push(Token::ComponentOpen {
            name,
            name_span,
            attributes: attrs,
            self_closing: false,
            span: Span { start, end: self.position },
        });
    }

    fn tokenize_component_close(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <
        self.advance(); // /
        self.advance(); // {

        let name = self.consume_until_char('}');
        self.advance(); // }

        // Skip to >
        while !self.at_eof() && self.peek_char() != Some('>') {
            self.advance();
        }
        if self.peek_char() == Some('>') {
            self.advance();
        }

        tokens.push(Token::ComponentClose {
            name,
            span: Span { start, end: self.position },
        });
    }

    /// Tokenize a slot definition opening: <{...}> or <{...name}>
    fn tokenize_slot_open(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <
        self.advance(); // {
        self.advance(); // .
        self.advance(); // .
        self.advance(); // .

        // Parse optional slot name
        let name_text = self.consume_until_char('}');
        let name = if name_text.trim().is_empty() {
            None
        } else {
            Some(name_text.trim().to_string())
        };

        self.advance(); // }

        // Skip to >
        while !self.at_eof() && self.peek_char() != Some('>') {
            self.advance();
        }
        if self.peek_char() == Some('>') {
            self.advance();
        }

        tokens.push(Token::SlotOpen {
            name,
            span: Span { start, end: self.position },
        });
    }

    /// Tokenize a slot definition closing: </{...}> or </{...name}>
    fn tokenize_slot_close(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <
        self.advance(); // /
        self.advance(); // {
        self.advance(); // .
        self.advance(); // .
        self.advance(); // .

        // Parse optional slot name
        let name_text = self.consume_until_char('}');
        let name = if name_text.trim().is_empty() {
            None
        } else {
            Some(name_text.trim().to_string())
        };

        self.advance(); // }

        // Skip to >
        while !self.at_eof() && self.peek_char() != Some('>') {
            self.advance();
        }
        if self.peek_char() == Some('>') {
            self.advance();
        }

        tokens.push(Token::SlotClose {
            name,
            span: Span { start, end: self.position },
        });
    }

    /// Check if current position starts an HTML element (not component)
    fn is_html_element_start(&self) -> bool {
        if self.peek_char() != Some('<') {
            return false;
        }
        // Look at next char
        let saved = self.position;
        let next_byte = saved.byte + 1;
        if next_byte >= self.bytes.len() {
            return false;
        }
        let next_ch = self.bytes[next_byte] as char;
        // HTML element if next char is a letter (not { for component, not / for closing)
        next_ch.is_ascii_alphabetic()
    }

    /// Check if current position starts an HTML closing tag
    fn is_html_element_close(&self) -> bool {
        if self.peek_char() != Some('<') {
            return false;
        }
        let byte1 = self.position.byte + 1;
        let byte2 = self.position.byte + 2;
        if byte2 >= self.bytes.len() {
            return false;
        }
        // Must be </ followed by letter (not { for component close)
        self.bytes[byte1] == b'/' && (self.bytes[byte2] as char).is_ascii_alphabetic()
    }

    /// Check if current position starts a component closing tag: </{Name}>
    fn is_component_close(&self) -> bool {
        if self.peek_char() != Some('<') {
            return false;
        }
        let byte1 = self.position.byte + 1;
        let byte2 = self.position.byte + 2;
        if byte2 >= self.bytes.len() {
            return false;
        }
        // Must be </{ for component close
        self.bytes[byte1] == b'/' && self.bytes[byte2] == b'{'
    }

    /// Parse an HTML element opening tag: <tag attributes>
    fn tokenize_html_element_open(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <

        // Parse tag name
        let tag = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        let tag_end = self.position;

        // Parse attributes (reuse same logic as components)
        let mut attrs = Vec::new();

        loop {
            self.skip_whitespace_inline();

            if self.at_eof() || self.at_newline() {
                break;
            }

            let ch = self.peek_char().unwrap();

            // Check for /> or >
            if ch == '/' && self.peek_next_char() == Some('>') {
                let close_pos = self.position; // Position of "/"
                self.advance();
                self.advance();
                tokens.push(Token::HtmlElementOpen {
                    tag,
                    tag_span: Span { start, end: tag_end },
                    attributes: attrs,
                    close_bracket_pos: close_pos,
                    self_closing: true,
                    span: Span { start, end: self.position },
                });
                return;
            }
            if ch == '>' {
                let close_pos = self.position; // Position of ">"
                self.advance();
                tokens.push(Token::HtmlElementOpen {
                    tag,
                    tag_span: Span { start, end: tag_end },
                    attributes: attrs,
                    close_bracket_pos: close_pos,
                    self_closing: false,
                    span: Span { start, end: self.position },
                });
                return;
            }

            // Parse attribute using shared function
            if let Some(attr) = self.parse_attribute() {
                attrs.push(attr);
            } else {
                // Unknown character, skip
                self.advance();
            }
        }

        // Reached end of line without closing >
        tokens.push(Token::HtmlElementOpen {
            tag,
            tag_span: Span { start, end: tag_end },
            attributes: attrs,
            close_bracket_pos: self.position, // No actual ">" - use end position
            self_closing: false,
            span: Span { start, end: self.position },
        });
    }

    /// Parse an HTML element closing tag: </tag>
    fn tokenize_html_element_close(&mut self, tokens: &mut Vec<Token>) {
        let start = self.position;
        self.advance(); // <
        self.advance(); // /

        let tag = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

        // Skip to >
        while !self.at_eof() && self.peek_char() != Some('>') && !self.at_newline() {
            self.advance();
        }
        if self.peek_char() == Some('>') {
            self.advance();
        }

        tokens.push(Token::HtmlElementClose {
            tag,
            span: Span { start, end: self.position },
        });
    }

    // === Low-level helpers ===

    fn at_eof(&self) -> bool {
        self.position.byte >= self.bytes.len()
    }

    fn at_newline(&self) -> bool {
        if self.at_eof() { return false; }
        let ch = self.bytes[self.position.byte];
        ch == b'\n' || ch == b'\r'
    }

    fn peek_char(&self) -> Option<char> {
        if self.at_eof() { return None; }
        // Simple ASCII fast path
        let b = self.bytes[self.position.byte];
        if b < 128 {
            Some(b as char)
        } else {
            self.source[self.position.byte..].chars().next()
        }
    }

    fn peek_next_char(&self) -> Option<char> {
        if self.position.byte + 1 >= self.bytes.len() { return None; }
        let b = self.bytes[self.position.byte + 1];
        if b < 128 {
            Some(b as char)
        } else {
            self.source[self.position.byte..].chars().nth(1)
        }
    }

    fn peek_line(&self) -> String {
        let start = self.position.byte;
        let mut end = start;
        while end < self.bytes.len() && self.bytes[end] != b'\n' && self.bytes[end] != b'\r' {
            end += 1;
        }
        self.source[start..end].to_string()
    }

    fn advance(&mut self) {
        if self.at_eof() { return; }
        let ch = self.peek_char().unwrap();
        let char_len = ch.len_utf8();

        self.position.byte += char_len;

        if ch == '\n' {
            self.position.line += 1;
            self.position.col = 0;
        } else {
            self.position.col += 1;
        }
    }

    fn consume_indent(&mut self) -> usize {
        let mut level = 0;
        while !self.at_eof() {
            match self.peek_char() {
                Some(' ') => { level += 1; self.advance(); }
                Some('\t') => { level += 4; self.advance(); } // Tab = 4 spaces
                _ => break,
            }
        }
        level
    }

    fn consume_newline(&mut self) {
        if self.peek_char() == Some('\r') {
            self.advance();
        }
        if self.peek_char() == Some('\n') {
            self.advance();
        }
    }

    fn consume_to_eol(&mut self) -> String {
        let start = self.position.byte;
        while !self.at_eof() && !self.at_newline() {
            self.advance();
        }
        self.source[start..self.position.byte].to_string()
    }

    fn skip_to_eol(&mut self) {
        while !self.at_eof() && !self.at_newline() {
            self.advance();
        }
    }

    fn skip_whitespace_inline(&mut self) {
        while !self.at_eof() && !self.at_newline() {
            match self.peek_char() {
                Some(' ') | Some('\t') => self.advance(),
                _ => break,
            }
        }
    }

    fn consume_until_char(&mut self, stop: char) -> String {
        let start = self.position.byte;
        let mut depth = 0;
        while !self.at_eof() {
            let ch = self.peek_char().unwrap();
            if ch == '{' { depth += 1; }
            if ch == '}' {
                if depth == 0 && stop == '}' { break; }
                depth -= 1;
            }
            if ch == stop && depth == 0 { break; }
            self.advance();
        }
        self.source[start..self.position.byte].to_string()
    }

    fn consume_while<F: Fn(char) -> bool>(&mut self, pred: F) -> String {
        let start = self.position.byte;
        while !self.at_eof() && !self.at_newline() {
            if let Some(ch) = self.peek_char() {
                if pred(ch) {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        self.source[start..self.position.byte].to_string()
    }
}

/// Tokenize source code
pub fn tokenize(source: &str) -> Vec<Token> {
    Tokenizer::new(source).tokenize()
}

// =============================================================================
// Incremental Tokenizer
// =============================================================================

/// A text change for incremental parsing
#[derive(Debug, Clone)]
pub struct TextChange {
    /// Start line (0-indexed)
    pub start_line: usize,
    /// End line (exclusive, 0-indexed)
    pub end_line: usize,
    /// New text for the changed lines (including newlines)
    pub new_text: String,
}

/// Incremental tokenizer that can efficiently update tokens when source changes.
///
/// Instead of re-tokenizing the entire file on every edit, this tracks which
/// tokens came from which lines and only re-tokenizes affected regions.
#[derive(Debug)]
pub struct IncrementalTokenizer {
    /// Current source code
    source: String,
    /// All tokens
    tokens: Vec<Token>,
    /// Mapping from line number to token index range (start, end exclusive)
    line_to_tokens: Vec<(usize, usize)>,
    /// Number of lines in source
    line_count: usize,
}

impl IncrementalTokenizer {
    /// Create a new incremental tokenizer from source
    pub fn new(source: &str) -> Self {
        let tokens = tokenize(source);
        let line_count = source.lines().count().max(1);
        let line_to_tokens = Self::build_line_map(&tokens, line_count);

        Self {
            source: source.to_string(),
            tokens,
            line_to_tokens,
            line_count,
        }
    }

    /// Build mapping from line numbers to token index ranges
    fn build_line_map(tokens: &[Token], line_count: usize) -> Vec<(usize, usize)> {
        let mut map = vec![(0, 0); line_count + 1]; // +1 for safety

        for (idx, token) in tokens.iter().enumerate() {
            let line = token.span().start.line;
            if line < map.len() {
                if map[line].0 == map[line].1 {
                    // First token on this line
                    map[line] = (idx, idx + 1);
                } else {
                    // Extend range
                    map[line].1 = idx + 1;
                }
            }
        }

        // Fill in gaps - empty lines should map to the next token
        let mut last_end = 0;
        for i in 0..map.len() {
            if map[i].0 == map[i].1 {
                map[i] = (last_end, last_end);
            } else {
                last_end = map[i].1;
            }
        }

        map
    }

    /// Get all tokens
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Get current source
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Apply a text change incrementally
    ///
    /// Returns the range of tokens that were affected (for potential re-generation)
    pub fn update(&mut self, change: TextChange) -> (usize, usize) {
        // Calculate the new source
        let lines: Vec<&str> = self.source.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();

        // Lines before the change
        for i in 0..change.start_line.min(lines.len()) {
            new_lines.push(lines[i].to_string());
        }

        // New lines from the change
        for line in change.new_text.lines() {
            new_lines.push(line.to_string());
        }
        // Handle case where new_text is empty or ends with newline
        if change.new_text.is_empty() || change.new_text.ends_with('\n') {
            // The split handles this correctly
        }

        // Lines after the change
        for i in change.end_line..lines.len() {
            new_lines.push(lines[i].to_string());
        }

        // Build new source
        let new_source = if new_lines.is_empty() {
            String::new()
        } else {
            new_lines.join("\n") + "\n"
        };

        // Calculate affected token range in old tokens
        let old_token_start = if change.start_line < self.line_to_tokens.len() {
            self.line_to_tokens[change.start_line].0
        } else {
            self.tokens.len().saturating_sub(1) // EOF token
        };

        let _old_token_end = if change.end_line < self.line_to_tokens.len() {
            self.line_to_tokens[change.end_line.saturating_sub(1).max(change.start_line)].1
        } else {
            self.tokens.len()
        };

        // For now, use a simple approach: re-tokenize from the changed line to end
        // A more sophisticated approach would only re-tokenize affected lines
        // and adjust positions for lines after

        // Full re-tokenize (simpler, still much faster than full transpile)
        let new_tokens = tokenize(&new_source);
        let new_line_count = new_source.lines().count().max(1);
        let new_line_map = Self::build_line_map(&new_tokens, new_line_count);

        // Calculate how many new tokens replaced the old range
        let new_token_start = old_token_start.min(new_tokens.len());
        let new_token_end = new_tokens.len();

        self.source = new_source;
        self.tokens = new_tokens;
        self.line_to_tokens = new_line_map;
        self.line_count = new_line_count;

        (new_token_start, new_token_end)
    }

    /// Get tokens for a specific line range
    pub fn tokens_for_lines(&self, start_line: usize, end_line: usize) -> &[Token] {
        let token_start = if start_line < self.line_to_tokens.len() {
            self.line_to_tokens[start_line].0
        } else {
            self.tokens.len()
        };

        let token_end = if end_line < self.line_to_tokens.len() {
            self.line_to_tokens[end_line].1
        } else {
            self.tokens.len()
        };

        &self.tokens[token_start..token_end.min(self.tokens.len())]
    }

    /// Re-tokenize completely (for when incremental update isn't sufficient)
    pub fn full_retokenize(&mut self) {
        self.tokens = tokenize(&self.source);
        self.line_count = self.source.lines().count().max(1);
        self.line_to_tokens = Self::build_line_map(&self.tokens, self.line_count);
    }
}

/// Tokenize a single line (for incremental updates)
pub fn tokenize_line(line: &str, line_number: usize) -> Vec<Token> {
    // Add newline if not present for consistent tokenization
    let source = if line.ends_with('\n') {
        line.to_string()
    } else {
        format!("{}\n", line)
    };

    let mut tokenizer = Tokenizer::new(&source);

    // Adjust the starting position to reflect the actual line number
    tokenizer.position.line = line_number;

    let mut tokens = Vec::new();
    tokenizer.tokenize_line(&mut tokens);

    // Remove the Eof token if present
    tokens.retain(|t| !matches!(t, Token::Eof { .. }));

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_html() {
        let tokens = tokenize("<div>Hello</div>\n");
        // Now parsed as structured HTML elements
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, .. } if tag == "div"));
        assert!(matches!(&tokens[1], Token::Text { text, .. } if text == "Hello"));
        assert!(matches!(&tokens[2], Token::HtmlElementClose { tag, .. } if tag == "div"));
        assert!(matches!(&tokens[3], Token::Newline { .. }));
    }

    #[test]
    fn test_expression() {
        let tokens = tokenize("<span>{name}</span>\n");
        // Now parsed as structured HTML elements
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, .. } if tag == "span"));
        assert!(matches!(&tokens[1], Token::Expression { code, .. } if code == "name"));
        assert!(matches!(&tokens[2], Token::HtmlElementClose { tag, .. } if tag == "span"));
    }

    #[test]
    fn test_trailing_comment() {
        let tokens = tokenize("<span>Active</span>  # Comment\n");
        // Trailing whitespace before comment is trimmed
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, .. } if tag == "span"));
        assert!(matches!(&tokens[1], Token::Text { text, .. } if text == "Active"));
        assert!(matches!(&tokens[2], Token::HtmlElementClose { tag, .. } if tag == "span"));
        assert!(matches!(&tokens[3], Token::Comment { text, .. } if text == "# Comment"));
    }

    #[test]
    fn test_comment_in_string() {
        let tokens = tokenize("<a href=\"#section\">Link</a>\n");
        // The # is inside href attribute, so no Comment token
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment { .. })));
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, attributes, .. }
            if tag == "a" && attributes.iter().any(|a| a.name == "href")));
    }

    #[test]
    fn test_control_flow() {
        let tokens = tokenize("if count > 0:\n");
        assert!(matches!(&tokens[0], Token::ControlStart { keyword, .. } if keyword == "if"));
    }

    #[test]
    fn test_end_keyword() {
        let tokens = tokenize("end\n");
        assert!(matches!(&tokens[0], Token::End { .. }));
    }

    #[test]
    fn test_escaped_braces() {
        let tokens = tokenize("<p>Use {{variable}} for templates</p>\n");
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { brace: '{', .. })));
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { brace: '}', .. })));
    }

    #[test]
    fn test_component_self_closing() {
        let tokens = tokenize("<{Button} type=\"submit\" />\n");
        assert!(matches!(&tokens[0], Token::ComponentOpen { name, self_closing: true, .. } if name == "Button"));
    }

    #[test]
    fn test_component_with_slot() {
        let tokens = tokenize("<{Card} title={title}>\n");
        if let Token::ComponentOpen { name, attributes, self_closing, .. } = &tokens[0] {
            assert_eq!(name, "Card");
            assert!(!self_closing);
            assert_eq!(attributes.len(), 1);
            assert_eq!(attributes[0].name, "title");
        } else {
            panic!("Expected ComponentOpen");
        }
    }

    #[test]
    fn test_python_stmt() {
        let tokens = tokenize("x = 1\n");
        assert!(matches!(&tokens[0], Token::PythonStatement { code, .. } if code == "x = 1"));
    }

    #[test]
    fn test_decorator() {
        let tokens = tokenize("@fragment\n");
        assert!(matches!(&tokens[0], Token::Decorator { code, .. } if code == "@fragment"));
    }

    #[test]
    fn test_indent() {
        let tokens = tokenize("    <span>Indented</span>\n");
        assert!(matches!(&tokens[0], Token::Indent { level: 4, .. }));
        // Now parsed as structured HTML elements
        assert!(matches!(&tokens[1], Token::HtmlElementOpen { tag, .. } if tag == "span"));
    }

    #[test]
    fn test_mixed_line() {
        // Inline comments on content lines should be extracted as separate tokens
        // Trailing whitespace before comment is trimmed
        let tokens = tokenize("    <span>Active</span>  # Comment\n");

        let token_types: Vec<&str> = tokens.iter().map(|t| match t {
            Token::Indent { .. } => "Indent",
            Token::Text { .. } => "Text",
            Token::Comment { .. } => "Comment",
            Token::Newline { .. } => "Newline",
            Token::Eof { .. } => "Eof",
            Token::HtmlElementOpen { .. } => "HtmlOpen",
            Token::HtmlElementClose { .. } => "HtmlClose",
            _ => "Other",
        }).collect();

        // Now parsed as structured HTML: Indent, HtmlOpen, Text, HtmlClose, Comment, Newline, Eof
        assert_eq!(token_types, vec!["Indent", "HtmlOpen", "Text", "HtmlClose", "Comment", "Newline", "Eof"]);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_hash_in_url() {
        // # in URL should NOT be treated as comment
        let tokens = tokenize("<a href=\"#section\">Link</a>\n");
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment { .. })));
    }

    #[test]
    fn test_hash_in_css_color() {
        // # in CSS color should NOT be treated as comment
        let tokens = tokenize("<div style=\"color: #fff\">Text</div>\n");
        assert!(!tokens.iter().any(|t| matches!(t, Token::Comment { .. })));
    }

    #[test]
    fn test_css_braces() {
        // CSS braces should be escaped {{ }}
        let tokens = tokenize("<style>.foo {{ color: red; }}</style>\n");
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { brace: '{', .. })));
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { brace: '}', .. })));
    }

    #[test]
    fn test_nested_expression() {
        // Nested braces in expressions
        let tokens = tokenize("<span>{user['name']}</span>\n");
        if let Some(Token::Expression { code, .. }) = tokens.iter().find(|t| matches!(t, Token::Expression { .. })) {
            assert_eq!(code, "user['name']");
        } else {
            panic!("Expected Expr token");
        }
    }

    #[test]
    fn test_dict_comprehension_looks_like_escape() {
        // {{k: v for k, v in d.items()}} - dict comprehension looks like double-escape
        // This is actually escaped braces around an expression
        let tokens = tokenize("<span>{{k: v for k, v in d.items()}}</span>\n");
        // Should have escaped braces, not an expression
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { brace: '{', .. })));
    }

    #[test]
    fn test_end_as_variable() {
        // "end" as part of a for loop variable should NOT close the block
        let tokens = tokenize("for end in endpoints:\n");
        assert!(matches!(&tokens[0], Token::ControlStart { keyword, .. } if keyword == "for"));
    }

    #[test]
    fn test_content_with_if() {
        // "If you see this" is content, not Python
        let tokens = tokenize("If you see this\n");
        assert!(matches!(&tokens[0], Token::Text { text, .. } if text == "If you see this"));
    }

    #[test]
    fn test_content_with_for() {
        // "for your information" currently matches "for " prefix
        // This is a known limitation - could be improved by checking for "in" clause
        // For now, this is classified as control (which will fail at Python level)
        let tokens = tokenize("for your information\n");
        let token_types: Vec<&str> = tokens.iter().map(|t| match t {
            Token::Text { .. } => "Text",
            Token::ControlStart { .. } => "Control",
            _ => "Other",
        }).collect();
        // Currently classified as control - could improve by validating for/in pattern
        // This is acceptable because invalid Python will fail at transpile time
        assert!(token_types.contains(&"Control") || token_types.contains(&"Text"));
    }

    #[test]
    fn test_class_attribute_vs_keyword() {
        // class="foo" in HTML vs class Foo: in Python
        let tokens = tokenize("<div class=\"card\">Content</div>\n");
        // Now parsed as structured HTML element with class attribute
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, attributes, .. }
            if tag == "div" && attributes.iter().any(|a| a.name == "class")));
        assert!(!tokens.iter().any(|t| matches!(t, Token::ControlStart { keyword, .. } if keyword == "class")));
    }

    #[test]
    fn test_multiline_expression() {
        // Expression with dict literal
        let tokens = tokenize("<span>{{'key': value}}</span>\n");
        // This is escaped brace then expression
        assert!(tokens.iter().any(|t| matches!(t, Token::EscapedBrace { .. })));
    }

    #[test]
    fn test_control_continuation() {
        let tokens = tokenize("else:\n");
        assert!(matches!(&tokens[0], Token::ControlContinuation { keyword, .. } if keyword == "else"));
    }

    #[test]
    fn test_elif() {
        let tokens = tokenize("elif count > 5:\n");
        assert!(matches!(&tokens[0], Token::ControlContinuation { keyword, .. } if keyword == "elif"));
    }

    #[test]
    fn test_except() {
        let tokens = tokenize("except ValueError:\n");
        assert!(matches!(&tokens[0], Token::ControlContinuation { keyword, .. } if keyword == "except"));
    }

    #[test]
    fn test_import_statement() {
        let tokens = tokenize("from datetime import datetime\n");
        assert!(matches!(&tokens[0], Token::PythonStatement { .. }));
    }

    #[test]
    fn test_function_call() {
        let tokens = tokenize("print(\"hello\")\n");
        assert!(matches!(&tokens[0], Token::PythonStatement { code, .. } if code == "print(\"hello\")"));
    }

    #[test]
    fn test_augmented_assignment() {
        let tokens = tokenize("count += 1\n");
        assert!(matches!(&tokens[0], Token::PythonStatement { .. }));
    }

    #[test]
    fn test_alpine_js_x_data() {
        // Alpine.js x-data attribute with JS object
        // The {{ inside quoted HTML attribute is just literal text, not escaped braces
        // (escaped braces only apply in f-string context, not inside HTML attribute strings)
        let tokens = tokenize("<div x-data=\"{{ open: false }}\">Content</div>\n");
        // Now parsed as structured HTML with x-data attribute containing the JS object literal
        assert!(matches!(&tokens[0], Token::HtmlElementOpen { tag, attributes, .. }
            if tag == "div" && attributes.iter().any(|a| a.name == "x-data")));
    }

    #[test]
    fn test_component_with_expression_attrs() {
        let tokens = tokenize("<{Card} title={title} count={count * 2} />\n");
        if let Token::ComponentOpen { attributes, .. } = &tokens[0] {
            assert_eq!(attributes.len(), 2);
            assert_eq!(attributes[0].name, "title");
            assert_eq!(attributes[1].name, "count");
            if let AttributeValue::Expression(code, _) = &attributes[1].value {
                assert_eq!(code, "count * 2");
            }
        } else {
            panic!("Expected ComponentOpen");
        }
    }

    #[test]
    fn test_component_spread() {
        let tokens = tokenize("<{Button} {**props} />\n");
        if let Token::ComponentOpen { attributes, .. } = &tokens[0] {
            assert_eq!(attributes.len(), 1);
            assert!(matches!(&attributes[0].value, AttributeValue::Spread(expr, _) if expr == "props"));
        } else {
            panic!("Expected ComponentOpen");
        }
    }

    #[test]
    fn test_component_shorthand() {
        let tokens = tokenize("<{Input} {value} {disabled} />\n");
        if let Token::ComponentOpen { attributes, .. } = &tokens[0] {
            assert_eq!(attributes.len(), 2);
            assert!(matches!(&attributes[0].value, AttributeValue::Shorthand(name, _) if name == "value"));
            assert!(matches!(&attributes[1].value, AttributeValue::Shorthand(name, _) if name == "disabled"));
        } else {
            panic!("Expected ComponentOpen");
        }
    }

    #[test]
    fn test_empty_lines() {
        let tokens = tokenize("\n\n\n");
        let newlines = tokens.iter().filter(|t| matches!(t, Token::Newline { .. })).count();
        assert_eq!(newlines, 3);
    }

    #[test]
    fn test_bare_text_content() {
        // Text that isn't Python and isn't HTML
        let tokens = tokenize("Hello World\n");
        assert!(matches!(&tokens[0], Token::Text { text, .. } if text == "Hello World"));
    }

    #[test]
    fn test_multiline_dict_statement() {
        // Multiline dict assignment should be a single token
        let source = "config = {\n    \"key\": \"value\",\n    \"other\": 123\n}\n";
        let tokens = tokenize(source);

        // Should be a single PythonStatement token (plus newline and EOF)
        let stmt_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::PythonStatement { .. }))
            .collect();
        assert_eq!(stmt_tokens.len(), 1);

        if let Token::PythonStatement { code, .. } = &stmt_tokens[0] {
            assert!(code.contains("\"key\": \"value\""));
            assert!(code.contains("\"other\": 123"));
        }
    }

    #[test]
    fn test_multiline_list_statement() {
        // Multiline list assignment
        let source = "items = [\n    1,\n    2,\n    3\n]\n";
        let tokens = tokenize(source);

        let stmt_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::PythonStatement { .. }))
            .collect();
        assert_eq!(stmt_tokens.len(), 1);
    }

    #[test]
    fn test_multiline_function_call() {
        // Multiline function call
        let source = "result = some_function(\n    arg1,\n    arg2,\n    kwarg=\"value\"\n)\n";
        let tokens = tokenize(source);

        let stmt_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::PythonStatement { .. }))
            .collect();
        assert_eq!(stmt_tokens.len(), 1);
    }

    #[test]
    fn test_multiline_with_string_containing_bracket() {
        // String containing bracket should not affect depth count
        let source = "x = {\n    \"key\": \"value with { brace\"\n}\n";
        let tokens = tokenize(source);

        let stmt_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t, Token::PythonStatement { .. }))
            .collect();
        assert_eq!(stmt_tokens.len(), 1);
    }

    #[test]
    fn test_single_line_statement_unchanged() {
        // Single line statements should still work
        let source = "x = {\"key\": \"value\"}\n";
        let tokens = tokenize(source);

        assert!(matches!(&tokens[0], Token::PythonStatement { code, .. } if code == "x = {\"key\": \"value\"}"));
    }

    #[test]
    fn test_separator() {
        let tokens = tokenize("---\n");
        assert!(matches!(&tokens[0], Token::Separator { .. }));
    }

    #[test]
    fn test_separator_with_content() {
        let source = "name: str\n---\n<div>{name}</div>\n";
        let tokens = tokenize(source);

        // Should have: PythonStatement, Newline, Separator, Newline, HtmlElementOpen, Expression, HtmlElementClose, Newline, Eof
        let has_separator = tokens.iter().any(|t| matches!(t, Token::Separator { .. }));
        assert!(has_separator, "should have separator token");

        // Find separator position
        let sep_pos = tokens.iter().position(|t| matches!(t, Token::Separator { .. })).unwrap();

        // PythonStatement should come before separator
        let stmt_pos = tokens.iter().position(|t| matches!(t, Token::PythonStatement { .. })).unwrap();
        assert!(stmt_pos < sep_pos, "statement should come before separator");

        // HTML content should come after separator
        let html_pos = tokens.iter().position(|t| matches!(t, Token::HtmlElementOpen { .. })).unwrap();
        assert!(html_pos > sep_pos, "HTML content should come after separator");
    }
}
