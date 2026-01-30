use crate::parser::tokenizer::Span;
use std::fmt;

/// Kind of parse error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    UnclosedElement,
    UnclosedComponent,
    UnclosedSlot,
    UnclosedBlock,
    MismatchedCloseTag,
    UnexpectedToken,
    InvalidSyntax,
    VoidElementWithContent,
    DuplicateAttribute,
    InvalidNesting,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::UnclosedElement => "Unclosed element",
            ErrorKind::UnclosedComponent => "Unclosed component",
            ErrorKind::UnclosedSlot => "Unclosed slot",
            ErrorKind::UnclosedBlock => "Unclosed block",
            ErrorKind::MismatchedCloseTag => "Mismatched close tag",
            ErrorKind::UnexpectedToken => "Unexpected token",
            ErrorKind::InvalidSyntax => "Invalid syntax",
            ErrorKind::VoidElementWithContent => "Void element with content",
            ErrorKind::DuplicateAttribute => "Duplicate attribute",
            ErrorKind::InvalidNesting => "Invalid nesting",
        }
    }
}

/// Error during parsing
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
    pub related_span: Option<Span>,
    pub related_label: Option<String>,
    pub help: Option<String>,
}

impl ParseError {
    /// Create a new parse error
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            related_span: None,
            related_label: None,
            help: None,
        }
    }

    /// Add a related span with a label (e.g., "opened here")
    pub fn with_related(mut self, span: Span) -> Self {
        self.related_span = Some(span);
        self
    }

    /// Set the label for the related span
    pub fn with_related_label(mut self, label: impl Into<String>) -> Self {
        self.related_label = Some(label.into());
        self
    }

    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render the error with source context (with ANSI colors when `color` is true)
    pub fn render(&self, source: &str, filename: &str) -> String {
        self.render_inner(source, filename, false)
    }

    /// Render the error with ANSI color codes
    pub fn render_color(&self, source: &str, filename: &str) -> String {
        self.render_inner(source, filename, true)
    }

    fn render_inner(&self, source: &str, filename: &str, color: bool) -> String {
        // Visual hierarchy: red for errors only, dim for structural chrome, bold for emphasis
        let red = if color { "\x1b[1;31m" } else { "" };
        let _bold = if color { "\x1b[1m" } else { "" };
        let dim = if color { "\x1b[2m" } else { "" };
        let underline = if color { "\x1b[4m" } else { "" };
        let cyan = if color { "\x1b[1;38;5;73m" } else { "" }; // bold #2cabb8 for help label
        let reset = if color { "\x1b[0m" } else { "" };

        let mut output = String::new();

        // Leading blank line for visual separation
        output.push('\n');

        // File location at the top: use related_span if available (points to where fix is needed)
        let loc_span = self.related_span.as_ref().unwrap_or(&self.span);
        let line = loc_span.start.line + 1;
        let col = loc_span.start.col + 1;
        let location = format!("{}:{}:{}", filename, line, col);
        if color {
            // OSC 8 hyperlink: \x1b]8;;URL\x07TEXT\x1b]8;;\x07
            let abs_path = std::path::Path::new(filename)
                .canonicalize()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| filename.to_string());
            output.push_str(&format!(
                " {}file:{} \x1b]8;;file://{}\x07{}{}{}\x1b]8;;\x07\n",
                dim, reset, abs_path, underline, location, reset
            ));
        } else {
            output.push_str(&format!(" file: {}\n", location));
        }

        // Error header: bold red label, message with highlighted tags
        let message = if color { highlight_inline_tags(&self.message) } else { self.message.clone() };
        output.push_str(&format!("{}error:{} {}\n", red, reset, message));

        // Source context
        let err_line = self.span.start.line + 1;
        if let Some(source_line) = source.lines().nth(self.span.start.line) {
            let line_num_width = format!("{}", err_line).len().max(2);
            let highlighted = if color { highlight_syntax(source_line) } else { source_line.to_string() };
            output.push_str(&format!("{}{:>width$} |{}\n", dim, "", reset, width = line_num_width));
            output.push_str(&format!("{}{:>width$} |{} {}\n", dim, err_line, reset, highlighted, width = line_num_width));

            // Underline: red carets — the primary visual anchor in the code
            let underline_start = self.span.start.col;
            let underline_len = if self.span.end.line == self.span.start.line {
                (self.span.end.col.saturating_sub(self.span.start.col)).max(1)
            } else {
                source_line.len().saturating_sub(underline_start).max(1)
            };

            let spaces = " ".repeat(underline_start);
            let carets = "^".repeat(underline_len);
            output.push_str(&format!(
                "{}{:>width$} |{} {}{}{}{}\n",
                dim, "", reset,
                spaces, red, carets, reset,
                width = line_num_width
            ));
        }

        // Related span: dim chrome, normal text — secondary context
        if let Some(ref related) = self.related_span {
            let related_line = related.start.line + 1;
            if let Some(related_source_line) = source.lines().nth(related.start.line) {
                let line_num_width = format!("{}", related_line).len().max(2);
                let highlighted = if color { highlight_syntax(related_source_line) } else { related_source_line.to_string() };
                output.push_str(&format!(
                    "{}{:>width$} |{} {}\n",
                    dim, related_line, reset,
                    highlighted,
                    width = line_num_width
                ));

                let underline_start = related.start.col;
                let underline_len = if related.end.line == related.start.line {
                    (related.end.col.saturating_sub(related.start.col)).max(1)
                } else {
                    related_source_line.len().saturating_sub(underline_start).max(1)
                };

                let spaces = " ".repeat(underline_start);
                let carets = "^".repeat(underline_len);
                let label = self.related_label.as_deref().unwrap_or("opened here");
                output.push_str(&format!(
                    "{}{:>width$} |{} {}{}{} {}{}\n",
                    dim, "", reset,
                    spaces, dim, carets, label, reset,
                    width = line_num_width
                ));
            }
        }

        // Help text: bold cyan label (aligned with error:), content with highlighted tags
        if let Some(ref help) = self.help {
            output.push('\n'); // blank line before help for spacing
            for (i, help_line) in help.lines().enumerate() {
                let content = if color { highlight_inline_tags(help_line) } else { help_line.to_string() };
                if i == 0 {
                    output.push_str(&format!(" {}help:{} {}\n", cyan, reset, content));
                } else {
                    output.push_str(&format!("       {}\n", content));
                }
            }
        }

        // Trailing blank line for visual separation
        output.push('\n');

        output
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Error during compilation (parsing or generation)
#[derive(Debug)]
pub enum CompileError {
    Parse(ParseError),
    Generate(String),
}

impl CompileError {
    /// Render the error with source context (no color)
    pub fn render(&self, source: &str, filename: &str) -> String {
        match self {
            CompileError::Parse(err) => err.render(source, filename),
            CompileError::Generate(msg) => format!("error: Generation error: {}\n", msg),
        }
    }

    /// Render the error with ANSI color codes
    pub fn render_color(&self, source: &str, filename: &str) -> String {
        match self {
            CompileError::Parse(err) => err.render_color(source, filename),
            CompileError::Generate(msg) => format!("\x1b[1;31merror\x1b[0m: \x1b[1m{}\x1b[0m\n", msg),
        }
    }
}

impl From<ParseError> for CompileError {
    fn from(err: ParseError) -> Self {
        CompileError::Parse(err)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Parse(err) => write!(f, "{}", err),
            CompileError::Generate(msg) => write!(f, "Generation error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

/// Highlight tags and keywords in prose text (error messages, help text)
fn highlight_inline_tags(text: &str) -> String {
    const TAG: &str = "\x1b[38;5;180m"; // #d5b778 - HTML tags
    const KEYWORD: &str = "\x1b[38;5;173m"; // #ce8e6d - Python keywords
    const RESET: &str = "\x1b[0m";

    // Known HTML tags (avoid highlighting placeholders like <variable>)
    const HTML_TAGS: &[&str] = &[
        "a", "abbr", "address", "area", "article", "aside", "audio",
        "b", "base", "bdi", "bdo", "blockquote", "body", "br", "button",
        "canvas", "caption", "cite", "code", "col", "colgroup",
        "data", "datalist", "dd", "del", "details", "dfn", "dialog", "div", "dl", "dt",
        "em", "embed", "fieldset", "figcaption", "figure", "footer", "form",
        "h1", "h2", "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", "html",
        "i", "iframe", "img", "input", "ins", "kbd", "label", "legend", "li", "link",
        "main", "map", "mark", "menu", "meta", "meter", "nav", "noscript",
        "object", "ol", "optgroup", "option", "output", "p", "picture", "pre", "progress",
        "q", "rp", "rt", "ruby", "s", "samp", "script", "section", "select", "slot", "small",
        "source", "span", "strong", "style", "sub", "summary", "sup", "svg",
        "table", "tbody", "td", "template", "textarea", "tfoot", "th", "thead", "time", "title", "tr", "track",
        "u", "ul", "var", "video", "wbr",
    ];

    // Python keywords to highlight in code-like contexts
    const CODE_KEYWORDS: &[&str] = &[
        "for", "in", "if", "else", "elif", "while", "with", "as", "try", "except",
        "finally", "def", "class", "return", "yield", "import", "from", "end",
    ];

    // Check if text contains placeholder patterns like <variable> (indicates code example)
    let has_placeholders = {
        let chars: Vec<char> = text.chars().collect();
        let mut found = false;
        let mut j = 0;
        while j < chars.len() {
            if chars[j] == '<' {
                j += 1;
                let start = j;
                while j < chars.len() && chars[j].is_alphabetic() {
                    j += 1;
                }
                if j > start && j < chars.len() && chars[j] == '>' {
                    let word: String = chars[start..j].iter().collect();
                    if !HTML_TAGS.contains(&word.to_lowercase().as_str()) {
                        found = true;
                        break;
                    }
                }
            }
            j += 1;
        }
        found
    };

    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check for quoted code like 'end' or 'async with' - render as `end` with highlighting
        if chars[i] == '\'' {
            let start = i;
            i += 1;
            let content_start = i;
            // Collect until closing quote
            while i < chars.len() && chars[i] != '\'' {
                i += 1;
            }
            if i < chars.len() && i > content_start {
                let content: String = chars[content_start..i].iter().collect();
                i += 1; // skip closing quote

                // Render as `content` with keyword color
                result.push_str(KEYWORD);
                result.push('`');
                result.push_str(&content);
                result.push('`');
                result.push_str(RESET);
                continue;
            }
            // Not valid, reset
            i = start;
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // Component syntax: <{Name}> or </{Name}>
        if chars[i] == '<' {
            let start = i;
            i += 1;
            let is_close = i < chars.len() && chars[i] == '/';
            if is_close {
                i += 1;
            }

            // Check for component: <{Name}>
            if i < chars.len() && chars[i] == '{' {
                i += 1;
                let name_start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name_end = i;
                if i < chars.len() && chars[i] == '}' {
                    i += 1;
                    // Skip optional space and />
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '/' {
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '>' {
                        // Valid component
                        result.push_str(TAG);
                        result.push('<');
                        if is_close {
                            result.push('/');
                        }
                        result.push_str(RESET);
                        result.push_str(KEYWORD);
                        result.push('{');
                        result.push_str(RESET);
                        for c in &chars[name_start..name_end] {
                            result.push(*c);
                        }
                        result.push_str(KEYWORD);
                        result.push('}');
                        result.push_str(RESET);
                        result.push_str(TAG);
                        result.push('>');
                        result.push_str(RESET);
                        i += 1;
                        continue;
                    }
                }
                // Not valid, reset
                i = start;
                result.push(chars[i]);
                i += 1;
                continue;
            }

            // Regular HTML tag
            let name_start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-') {
                i += 1;
            }
            let name_end = i;

            if name_end > name_start {
                let tag_name: String = chars[name_start..name_end].iter().collect();
                let tag_lower = tag_name.to_lowercase();

                // Only highlight known HTML tags
                if HTML_TAGS.contains(&tag_lower.as_str()) {
                    // Skip whitespace and check for self-closing /
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    let is_self_closing = i < chars.len() && chars[i] == '/';
                    if is_self_closing {
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '>' {
                        result.push_str(TAG);
                        result.push('<');
                        if is_close {
                            result.push('/');
                        }
                        result.push_str(&tag_name);
                        if is_self_closing {
                            result.push_str(" /");
                        }
                        result.push('>');
                        result.push_str(RESET);
                        i += 1;
                        continue;
                    }
                }
            }
            // Not a valid tag, reset
            i = start;
            result.push(chars[i]);
            i += 1;
            continue;
        }

        // Keywords: highlight if text contains placeholder patterns (code example context)
        if has_placeholders && chars[i].is_alphabetic() {
            let word_start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[word_start..i].iter().collect();

            if CODE_KEYWORDS.contains(&word.as_str()) {
                result.push_str(KEYWORD);
                result.push_str(&word);
                result.push_str(RESET);
            } else {
                result.push_str(&word);
            }
            continue;
        }

        // Default: pass through
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Syntax highlighting for error context lines (JetBrains New UI dark theme)
fn highlight_syntax(line: &str) -> String {
    // 256-color ANSI codes approximating JetBrains New UI dark theme
    const TAG: &str = "\x1b[38;5;180m"; // #d5b778 - HTML tags
    const ATTR: &str = "\x1b[38;5;250m"; // #bababa - attribute names
    const STRING: &str = "\x1b[38;5;72m"; // #6aab73 - strings, =, expression braces
    const KEYWORD: &str = "\x1b[38;5;173m"; // #ce8e6d - Python keywords
    const BUILTIN: &str = "\x1b[38;5;103m"; // #8888c5 - builtin functions
    const NUMBER: &str = "\x1b[38;5;73m"; // #2cabb8 - numbers
    const RESET: &str = "\x1b[0m";

    // Keywords that are unambiguous (rarely appear as English words in HTML content)
    const KEYWORDS: &[&str] = &[
        "if", "elif", "else", "for", "while", "with", "match", "case", "try", "except", "finally",
        "def", "class", "return", "yield", "import", "from", "pass", "break",
        "continue", "raise", "assert", "async", "await", "lambda", "None", "True",
        "False", "end",
    ];

    // Keywords that are also common English words - only highlight in code context
    const AMBIGUOUS_KEYWORDS: &[&str] = &["is", "in", "as", "or", "and", "not"];

    const BUILTINS: &[&str] = &[
        "print", "len", "range", "open", "str", "int", "float", "bool", "list", "dict",
        "set", "tuple", "type", "isinstance", "hasattr", "getattr", "setattr", "delattr",
        "sum", "min", "max", "abs", "round", "sorted", "reversed", "enumerate", "zip",
        "map", "filter", "any", "all", "input", "format", "repr", "id", "hex", "bin", "oct",
    ];

    // Check if line starts with Python code (keyword at start)
    let trimmed = line.trim_start();
    let first_word: String = trimmed.chars().take_while(|c| c.is_alphabetic() || *c == '_').collect();
    let line_is_python = KEYWORDS.contains(&first_word.as_str());

    let mut result = String::with_capacity(line.len() * 2);
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // HTML/Component tags: <tag>, </tag>, <tag />, <{Component}>, </{Component}>
        if chars[i] == '<' {
            let start = i;
            i += 1;
            let is_close = i < chars.len() && chars[i] == '/';
            if is_close {
                i += 1;
            }

            // Component syntax: <{Name}>
            if i < chars.len() && chars[i] == '{' {
                let brace_pos = i;
                i += 1;
                let name_start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name_end = i;
                if name_end > name_start && i < chars.len() && chars[i] == '}' {
                    i += 1;
                    // Handle attributes or self-closing
                    let mut attrs = String::new();
                    let mut found_close = false;
                    while i < chars.len() && chars[i] != '>' {
                        if chars[i].is_whitespace() {
                            attrs.push(chars[i]);
                            i += 1;
                        } else if chars[i] == '/' {
                            attrs.push_str(TAG);
                            attrs.push('/');
                            attrs.push_str(RESET);
                            i += 1;
                        } else if chars[i] == '=' {
                            attrs.push_str(STRING);
                            attrs.push('=');
                            attrs.push_str(RESET);
                            i += 1;
                        } else if chars[i] == '"' || chars[i] == '\'' {
                            let quote = chars[i];
                            attrs.push_str(STRING);
                            attrs.push(quote);
                            i += 1;
                            while i < chars.len() && chars[i] != quote {
                                attrs.push(chars[i]);
                                i += 1;
                            }
                            if i < chars.len() {
                                attrs.push(quote);
                                i += 1;
                            }
                            attrs.push_str(RESET);
                        } else if chars[i] == '{' {
                            attrs.push_str(STRING);
                            attrs.push('{');
                            attrs.push_str(RESET);
                            i += 1;
                            let mut depth = 1;
                            while i < chars.len() && depth > 0 {
                                if chars[i] == '{' {
                                    depth += 1;
                                    attrs.push_str(STRING);
                                    attrs.push('{');
                                    attrs.push_str(RESET);
                                } else if chars[i] == '}' {
                                    depth -= 1;
                                    attrs.push_str(STRING);
                                    attrs.push('}');
                                    attrs.push_str(RESET);
                                } else {
                                    attrs.push(chars[i]);
                                }
                                i += 1;
                            }
                        } else if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == ':' || chars[i] == '@' {
                            attrs.push_str(ATTR);
                            while i < chars.len()
                                && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == ':' || chars[i] == '@')
                            {
                                attrs.push(chars[i]);
                                i += 1;
                            }
                            attrs.push_str(RESET);
                        } else {
                            attrs.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i < chars.len() && chars[i] == '>' {
                        found_close = true;
                        i += 1;
                    }
                    if found_close {
                        // Output: <{Name}> with proper colors
                        result.push_str(TAG);
                        result.push('<');
                        if is_close {
                            result.push('/');
                        }
                        result.push_str(RESET);
                        result.push_str(KEYWORD);
                        result.push('{');
                        result.push_str(RESET);
                        for c in &chars[name_start..name_end] {
                            result.push(*c);
                        }
                        result.push_str(KEYWORD);
                        result.push('}');
                        result.push_str(RESET);
                        result.push_str(&attrs);
                        result.push_str(TAG);
                        result.push('>');
                        result.push_str(RESET);
                        continue;
                    }
                }
                // Not valid component, reset
                i = brace_pos;
            }

            // Regular HTML tag
            let name_start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
                i += 1;
            }
            let name_end = i;

            if name_end > name_start {
                // Output tag name
                result.push_str(TAG);
                result.push('<');
                if is_close {
                    result.push('/');
                }
                for c in &chars[name_start..name_end] {
                    result.push(*c);
                }
                result.push_str(RESET);

                // Handle attributes until >
                while i < chars.len() && chars[i] != '>' {
                    if chars[i].is_whitespace() {
                        result.push(chars[i]);
                        i += 1;
                    } else if chars[i] == '/' {
                        result.push_str(TAG);
                        result.push('/');
                        result.push_str(RESET);
                        i += 1;
                    } else if chars[i] == '=' {
                        result.push_str(STRING);
                        result.push('=');
                        result.push_str(RESET);
                        i += 1;
                    } else if chars[i] == '"' || chars[i] == '\'' {
                        let quote = chars[i];
                        result.push_str(STRING);
                        result.push(quote);
                        i += 1;
                        while i < chars.len() && chars[i] != quote {
                            result.push(chars[i]);
                            i += 1;
                        }
                        if i < chars.len() {
                            result.push(quote);
                            i += 1;
                        }
                        result.push_str(RESET);
                    } else if chars[i] == '{' {
                        result.push_str(STRING);
                        result.push('{');
                        result.push_str(RESET);
                        i += 1;
                        let mut depth = 1;
                        while i < chars.len() && depth > 0 {
                            if chars[i] == '{' {
                                depth += 1;
                                result.push_str(STRING);
                                result.push('{');
                                result.push_str(RESET);
                            } else if chars[i] == '}' {
                                depth -= 1;
                                result.push_str(STRING);
                                result.push('}');
                                result.push_str(RESET);
                            } else {
                                result.push(chars[i]);
                            }
                            i += 1;
                        }
                    } else if chars[i].is_alphabetic() || chars[i] == '_' || chars[i] == ':' || chars[i] == '@' {
                        result.push_str(ATTR);
                        while i < chars.len()
                            && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == ':' || chars[i] == '@')
                        {
                            result.push(chars[i]);
                            i += 1;
                        }
                        result.push_str(RESET);
                    } else {
                        result.push(chars[i]);
                        i += 1;
                    }
                }

                if i < chars.len() && chars[i] == '>' {
                    result.push_str(TAG);
                    result.push('>');
                    result.push_str(RESET);
                    i += 1;
                }
                continue;
            } else {
                // Not a valid tag
                i = start;
                result.push(chars[i]);
                i += 1;
                continue;
            }
        }

        // Standalone expressions: {expr}
        if chars[i] == '{' {
            result.push_str(STRING);
            result.push('{');
            result.push_str(RESET);
            i += 1;
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '{' {
                    depth += 1;
                    result.push_str(STRING);
                    result.push('{');
                    result.push_str(RESET);
                } else if chars[i] == '}' {
                    depth -= 1;
                    result.push_str(STRING);
                    result.push('}');
                    result.push_str(RESET);
                } else {
                    result.push(chars[i]);
                }
                i += 1;
            }
            continue;
        }

        // Python strings
        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            result.push_str(STRING);
            result.push(quote);
            i += 1;
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    result.push(chars[i]);
                    i += 1;
                    result.push(chars[i]);
                    i += 1;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
            if i < chars.len() {
                result.push(quote);
                i += 1;
            }
            result.push_str(RESET);
            continue;
        }

        // Identifiers: keywords, builtins, or regular names
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let word_start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[word_start..i].iter().collect();

            // Check if followed by ( for builtin detection
            let is_call = i < chars.len() && chars[i] == '(';

            // Highlight keywords, but be careful with ambiguous ones (is, in, as, etc.)
            // Only highlight ambiguous keywords if the line starts with Python code
            let is_keyword = KEYWORDS.contains(&word.as_str());
            let is_ambiguous = AMBIGUOUS_KEYWORDS.contains(&word.as_str());

            if is_keyword || (is_ambiguous && line_is_python) {
                result.push_str(KEYWORD);
                result.push_str(&word);
                result.push_str(RESET);
            } else if is_call && BUILTINS.contains(&word.as_str()) {
                result.push_str(BUILTIN);
                result.push_str(&word);
                result.push_str(RESET);
            } else {
                result.push_str(&word);
            }
            continue;
        }

        // Numbers
        if chars[i].is_ascii_digit() {
            result.push_str(NUMBER);
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.' || chars[i] == '_' || chars[i] == 'e' || chars[i] == 'E') {
                result.push(chars[i]);
                i += 1;
            }
            result.push_str(RESET);
            continue;
        }

        // Default: pass through
        result.push(chars[i]);
        i += 1;
    }

    result
}
