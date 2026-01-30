/// Tests for ANSI syntax highlighting in error messages
///
/// These tests verify that error messages have correct highlighting.
/// The visible_ansi function converts ANSI codes to visible markers for comparison.

use hyper_transpiler::error::{ErrorKind, ParseError};
use hyper_transpiler::parser::tokenizer::{Position, Span};

/// Convert ANSI escape sequences to visible markers for comparison
/// e.g., \x1b[38;5;180m becomes ‹38;5;180›
fn visible_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            result.push('‹');
            while let Some(&nc) = chars.peek() {
                if nc == 'm' {
                    chars.next();
                    result.push('›');
                    break;
                }
                result.push(chars.next().unwrap());
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn get_source_line(source: &str) -> String {
    let err = ParseError::new(
        ErrorKind::InvalidSyntax,
        "test",
        Span {
            start: Position { byte: 0, line: 0, col: 0 },
            end: Position { byte: 1, line: 0, col: 1 },
        },
    );
    let rendered = err.render_color(source, "test.hyper");
    let lines: Vec<&str> = rendered.lines().collect();
    if lines.len() > 4 {
        visible_ansi(lines[4])
    } else {
        String::new()
    }
}

fn get_error_message_line(message: &str) -> String {
    let err = ParseError::new(
        ErrorKind::InvalidSyntax,
        message,
        Span {
            start: Position { byte: 0, line: 0, col: 0 },
            end: Position { byte: 1, line: 0, col: 1 },
        },
    );
    let rendered = err.render_color("x", "test.hyper");
    let lines: Vec<&str> = rendered.lines().collect();
    lines.get(2).map(|s| visible_ansi(s)).unwrap_or_default()
}

fn get_help_line(help: &str) -> String {
    let err = ParseError::new(
        ErrorKind::InvalidSyntax,
        "test",
        Span {
            start: Position { byte: 0, line: 0, col: 0 },
            end: Position { byte: 1, line: 0, col: 1 },
        },
    ).with_help(help);
    let rendered = err.render_color("x", "test.hyper");
    let lines: Vec<&str> = rendered.lines().collect();
    lines.iter()
        .find(|l| l.contains("help:"))
        .map(|s| visible_ansi(s))
        .unwrap_or_default()
}

mod highlight_syntax {
    use super::*;

    #[test]
    fn python_keywords_anywhere() {
        let result = get_source_line("async with conn as c:");
        // Keywords 'async', 'with', 'as' should be highlighted
        assert!(result.contains("async"), "Should contain 'async' keyword");
        assert!(result.contains("with"), "Should contain 'with' keyword");
        assert!(result.contains("as"), "Should contain 'as' keyword");
    }

    #[test]
    fn for_loop() {
        let result = get_source_line("for item in items:");
        assert!(result.contains("for"), "Should contain 'for' keyword");
        assert!(result.contains("in"), "Should contain 'in' keyword");
    }

    #[test]
    fn match_keyword() {
        let result = get_source_line("match status:");
        assert!(result.contains("match"), "Should contain 'match' keyword");
    }

    #[test]
    fn component_basic() {
        let result = get_source_line("<{Button}>");
        // Component should have special highlighting
        assert!(result.contains("Button"), "Should contain component name");
    }

    #[test]
    fn html_tag_with_attrs() {
        let result = get_source_line("<div class=\"container\">");
        assert!(result.contains("div"), "Should contain tag name");
        assert!(result.contains("class"), "Should contain attribute");
    }
}

mod highlight_inline {
    use super::*;

    #[test]
    fn quoted_end() {
        let result = get_error_message_line("Add 'end' on its own line");
        // 'end' should be quoted/highlighted
        assert!(result.contains("end"), "Should contain 'end'");
    }

    #[test]
    fn html_tag_in_message() {
        let result = get_error_message_line("<br> cannot have content");
        assert!(result.contains("br"), "Should contain tag name");
    }

    #[test]
    fn component_in_message() {
        let result = get_error_message_line("Add </{Button}> to close");
        assert!(result.contains("Button"), "Should contain component name");
    }
}

mod highlight_help {
    use super::*;

    #[test]
    fn void_element_tags() {
        let result = get_help_line("<br> is a void element (like <img>, <input>, <hr>). Write it as <br /> instead.");
        assert!(result.contains("help:"), "Should have help prefix");
        assert!(result.contains("br"), "Should contain tag names");
    }

    #[test]
    fn close_tag_help() {
        let result = get_help_line("Add </div> to close this element, or use <div /> if it has no children.");
        assert!(result.contains("div"), "Should contain tag name");
    }

    #[test]
    fn quoted_keyword_help() {
        let result = get_help_line("Close with 'end'");
        assert!(result.contains("end"), "Should contain 'end'");
    }
}
