/// Helper to make ANSI codes visible for snapshot comparison
fn visible_ansi(s: &str) -> String {
    // Replace ANSI escape sequences like \x1b[38;5;180m with ‹38;5;180›
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            result.push('‹');
            // consume until 'm'
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

mod highlight_syntax {
    use super::*;

    macro_rules! snap {
        ($name:ident, $input:expr) => {
            #[test]
            fn $name() {
                // We need to trigger an error to get highlighted output
                // For now, test via the error rendering
                let source = $input;
                let err = hyper_transpiler::error::ParseError::new(
                    hyper_transpiler::error::ErrorKind::InvalidSyntax,
                    "test",
                    hyper_transpiler::parser::tokenizer::Span {
                        start: hyper_transpiler::parser::tokenizer::Position { byte: 0, line: 0, col: 0 },
                        end: hyper_transpiler::parser::tokenizer::Position { byte: 1, line: 0, col: 1 },
                    },
                );
                let rendered = err.render_color(source, "test.hyper");
                // Extract just the source line (line 5 of output: blank, file, error, gutter, source)
                let lines: Vec<&str> = rendered.lines().collect();
                if lines.len() > 4 {
                    insta::assert_snapshot!(visible_ansi(lines[4]));
                }
            }
        };
    }

    snap!(python_keywords_anywhere, "async with conn as c:");
    snap!(builtin_function, "with open(\"file.txt\") as f:");
    snap!(numbers, "while count < 10:");
    snap!(component_basic, "<{Button}>");
    snap!(component_with_attrs, "<{Button} type=\"submit\">");
    snap!(self_closing_br, "<br />");
    snap!(self_closing_div, "<div />");
    snap!(html_tag_with_attrs, "<div class=\"container\" id={item.id}>");
    snap!(for_loop, "for item in items:");
    snap!(string_in_code, "name = \"hello\"");
    snap!(html_content_with_is, "<div>This is invalid HTML</div>");
    snap!(keyword_as_in_context, "with open(\"f\") as f:");
    snap!(match_keyword, "match status:");
}

mod highlight_inline {
    use super::*;

    macro_rules! snap {
        ($name:ident, $input:expr) => {
            #[test]
            fn $name() {
                let err = hyper_transpiler::error::ParseError::new(
                    hyper_transpiler::error::ErrorKind::InvalidSyntax,
                    $input,
                    hyper_transpiler::parser::tokenizer::Span {
                        start: hyper_transpiler::parser::tokenizer::Position { byte: 0, line: 0, col: 0 },
                        end: hyper_transpiler::parser::tokenizer::Position { byte: 1, line: 0, col: 1 },
                    },
                );
                let rendered = err.render_color("x", "test.hyper");
                // Extract just the error line (line 3: blank, file, error)
                let lines: Vec<&str> = rendered.lines().collect();
                let error_line = lines.get(2).unwrap_or(&"");
                insta::assert_snapshot!(visible_ansi(error_line));
            }
        };
    }

    snap!(quoted_end, "Add 'end' on its own line");
    snap!(quoted_async_with, "This 'async with' block is never closed");
    snap!(html_tag_in_message, "<br> cannot have content");
    snap!(component_in_message, "Add </{Button}> to close");
    snap!(no_highlight_if_prose, "if it has no children");
    snap!(no_highlight_as_prose, "renders as <p></p>");
    snap!(placeholder_not_highlighted, "Expected: for <variable> in <iterable>:");
}

mod highlight_help {
    use super::*;

    macro_rules! snap {
        ($name:ident, $help:expr) => {
            #[test]
            fn $name() {
                let err = hyper_transpiler::error::ParseError::new(
                    hyper_transpiler::error::ErrorKind::InvalidSyntax,
                    "test",
                    hyper_transpiler::parser::tokenizer::Span {
                        start: hyper_transpiler::parser::tokenizer::Position { byte: 0, line: 0, col: 0 },
                        end: hyper_transpiler::parser::tokenizer::Position { byte: 1, line: 0, col: 1 },
                    },
                ).with_help($help);
                let rendered = err.render_color("x", "test.hyper");
                // Extract help line (after blank, error, gutter, source, caret, blank)
                let lines: Vec<&str> = rendered.lines().collect();
                let help_line = lines.iter().find(|l| l.contains("help:")).unwrap_or(&"");
                insta::assert_snapshot!(visible_ansi(help_line));
            }
        };
    }

    snap!(void_element_tags, "<br> is a void element (like <img>, <input>, <hr>). Write it as <br /> instead.");
    snap!(close_tag_help, "Add </div> to close this element, or use <div /> if it has no children.");
    snap!(quoted_keyword_help, "Close with 'end'");
}
