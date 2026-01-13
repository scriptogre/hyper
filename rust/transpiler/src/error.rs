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
        // ANSI codes — empty strings when color is off
        let red = if color { "\x1b[1;31m" } else { "" };
        let cyan = if color { "\x1b[1;36m" } else { "" };
        let blue = if color { "\x1b[1;34m" } else { "" };
        let yellow = if color { "\x1b[33m" } else { "" };
        let bold = if color { "\x1b[1m" } else { "" };
        let dim = if color { "\x1b[2m" } else { "" };
        let reset = if color { "\x1b[0m" } else { "" };

        let mut output = String::new();

        // Error header
        output.push_str(&format!("{}error{}: {}{}{}\n", red, reset, bold, self.message, reset));

        // Location
        let line = self.span.start.line + 1;
        let col = self.span.start.col + 1;
        output.push_str(&format!("  {}-->{} {}:{}:{}\n", blue, reset, filename, line, col));

        // Source context
        if let Some(source_line) = source.lines().nth(self.span.start.line) {
            let line_num_width = format!("{}", line).len().max(2);
            output.push_str(&format!("{}{:>width$} |{}\n", blue, "", reset, width = line_num_width));
            output.push_str(&format!("{}{:>width$} |{} {}\n", blue, line, reset, source_line, width = line_num_width));

            // Underline the error position
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
                blue, "", reset,
                spaces, red, carets, reset,
                width = line_num_width
            ));
        }

        // Related span
        if let Some(ref related) = self.related_span {
            let related_line = related.start.line + 1;
            if let Some(related_source_line) = source.lines().nth(related.start.line) {
                let line_num_width = format!("{}", related_line).len().max(2);
                output.push_str(&format!("{}{:>width$} |{}\n", blue, "", reset, width = line_num_width));
                output.push_str(&format!(
                    "{}{:>width$} |{} {}\n",
                    blue, related_line, reset,
                    related_source_line,
                    width = line_num_width
                ));

                let underline_start = related.start.col;
                let underline_len = if related.end.line == related.start.line {
                    (related.end.col.saturating_sub(related.start.col)).max(1)
                } else {
                    related_source_line.len().saturating_sub(underline_start).max(1)
                };

                let spaces = " ".repeat(underline_start);
                let dashes = "-".repeat(underline_len);
                let label = self.related_label.as_deref().unwrap_or("opened here");
                output.push_str(&format!(
                    "{}{:>width$} |{} {}{}{} {}{}{}\n",
                    blue, "", reset,
                    spaces, cyan, dashes, label, reset, "",
                    width = line_num_width
                ));
            }
        }

        // Help text
        if let Some(ref help) = self.help {
            for (i, help_line) in help.lines().enumerate() {
                if i == 0 {
                    output.push_str(&format!("   {}= help:{} {}\n", yellow, reset, help_line));
                } else {
                    output.push_str(&format!("           {}\n", help_line));
                }
            }
        }

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
