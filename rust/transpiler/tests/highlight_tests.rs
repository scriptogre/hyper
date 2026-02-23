/// Tests for ANSI syntax highlighting in error messages
///
/// These tests verify that error messages have correct highlighting.
/// The visible_ansi function converts ANSI codes to visible markers for comparison.
/// The assert_highlighted helper verifies a word is wrapped in color codes.

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

/// Assert that `text` appears wrapped in ANSI color codes in the rendered output.
/// Checks for the pattern `›text‹` which means the text is preceded by a color
/// code's closing `›` and followed by another code's opening `‹` (typically reset).
fn assert_highlighted(rendered: &str, text: &str) {
    let visible = visible_ansi(rendered);
    let pattern = format!("›{}‹", text);
    assert!(visible.contains(&pattern),
        "Expected '{}' to be highlighted (wrapped in ANSI codes).\nGot: {}", text, visible);
}

/// Find the source line from a rendered error — the line with a line number, pipe
/// gutter, and ANSI color codes for highlighted source content.
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
    // The source line has syntax-highlighting color codes (38;5;NNN) after the pipe,
    // unlike the blank gutter line which only has dim/reset codes.
    rendered.lines()
        .find(|l| l.contains('|') && {
            if let Some(pipe_pos) = l.find('|') {
                l[pipe_pos..].contains("\x1b[38;5;")
            } else {
                false
            }
        })
        .unwrap_or("")
        .to_string()
}

/// Find the error message line — the line containing the "error:" label.
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
    rendered.lines()
        .find(|l| l.contains("error"))
        .unwrap_or("")
        .to_string()
}

/// Find the help line — the line containing the "help:" label.
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
    rendered.lines()
        .find(|l| l.contains("help:"))
        .unwrap_or("")
        .to_string()
}

mod highlight_syntax {
    use super::*;

    #[test]
    fn python_keywords_anywhere() {
        let line = get_source_line("async with conn as c:");
        assert_highlighted(&line, "async");
        assert_highlighted(&line, "with");
        assert_highlighted(&line, "as");
    }

    #[test]
    fn for_loop() {
        let line = get_source_line("for item in items:");
        assert_highlighted(&line, "for");
        assert_highlighted(&line, "in");
    }

    #[test]
    fn match_keyword() {
        let line = get_source_line("match status:");
        assert_highlighted(&line, "match");
    }

    #[test]
    fn component_basic() {
        // Component name is between {RESET and KEYWORD}: ‹0›Button‹38;5;173›
        // The braces are highlighted, not the name directly — verify the braces wrap it
        let line = get_source_line("<{Button}>");
        let visible = visible_ansi(&line);
        assert!(visible.contains("Button"), "Should contain component name");
        // Verify the braces around Button are keyword-colored
        assert_highlighted(&line, "{");
        assert_highlighted(&line, "}");
    }

    #[test]
    fn html_tag_with_attrs() {
        let line = get_source_line(r#"<div class="container">"#);
        // In source highlighting, <div is colored as TAG, and class as ATTR
        let visible = visible_ansi(&line);
        assert!(visible.contains("‹38;5;180›"), "Should have TAG color code for <div");
        assert!(visible.contains("div"), "Should contain tag name");
        assert!(visible.contains("‹38;5;250›"), "Should have ATTR color code for class");
        assert!(visible.contains("class"), "Should contain attribute");
    }
}

mod highlight_inline {
    use super::*;

    #[test]
    fn quoted_end() {
        let line = get_error_message_line("Add 'end' on its own line");
        // 'end' is rendered as `end` with keyword color
        assert_highlighted(&line, "`end`");
    }

    #[test]
    fn html_tag_in_message() {
        let line = get_error_message_line("<br> cannot have content");
        // The whole <br> tag is wrapped in TAG color
        assert_highlighted(&line, "<br>");
    }

    #[test]
    fn component_in_message() {
        let line = get_error_message_line("Add </{Button}> to close");
        // Component braces are keyword-colored, verify the structure is highlighted
        let visible = visible_ansi(&line);
        assert!(visible.contains("Button"), "Should contain component name");
        assert_highlighted(&line, "{");
        assert_highlighted(&line, "}");
    }
}

mod highlight_help {
    use super::*;

    #[test]
    fn void_element_tags() {
        let line = get_help_line("<br> is a void element (like <img>, <input>, <hr>). Write it as <br /> instead.");
        assert_highlighted(&line, "<br>");
        assert_highlighted(&line, "<img>");
        assert_highlighted(&line, "<input>");
        assert_highlighted(&line, "<hr>");
    }

    #[test]
    fn close_tag_help() {
        let line = get_help_line("Add </div> to close this element, or use <div /> if it has no children.");
        assert_highlighted(&line, "</div>");
        assert_highlighted(&line, "<div />");
    }

    #[test]
    fn quoted_keyword_help() {
        let line = get_help_line("Close with 'end'");
        assert_highlighted(&line, "`end`");
    }
}
