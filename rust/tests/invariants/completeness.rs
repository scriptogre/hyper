use crate::helpers::compile;
use hyper_transpiler::generate::RangeType;
use hyper_transpiler::parser::tokenizer::{Token, tokenize};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;
    let tokens = tokenize(&source);

    let python_ranges: Vec<_> = result
        .ranges
        .iter()
        .filter(|r| r.range_type == RangeType::Python)
        .collect();

    // Find separator position to distinguish preamble vs body
    let separator_byte = tokens.iter().find_map(|t| {
        if let Token::Separator { span } = t {
            Some(span.start.byte)
        } else {
            None
        }
    });

    let in_body = |byte: usize| -> bool { separator_byte.is_none_or(|sep| byte > sep) };

    let is_covered = |start: usize, end: usize| -> bool {
        python_ranges
            .iter()
            .any(|r| r.source_start <= start && r.source_end >= end)
    };

    /// Trim trailing `:` and whitespace from a rest_span to match what the
    /// generator emits (it strips the colon before adding the range).
    fn trim_colon_end(source: &str, start: usize, end: usize) -> usize {
        let text = &source[start..end];
        let trimmed = text.trim_end_matches(':').trim_end();
        start + trimmed.len()
    }

    for token in &tokens {
        match token {
            Token::Decorator { span, .. }
                if in_body(span.start.byte) && !is_covered(span.start.byte, span.end.byte) =>
            {
                return Err(format!(
                    "decorator at [{},{}] has no Python range: {:?}",
                    span.start.byte,
                    span.end.byte,
                    &source[span.start.byte..span.end.byte]
                )
                .into());
            }
            Token::ControlStart {
                keyword, rest_span, ..
            } if (keyword == "def" || keyword == "class" || keyword == "async def")
                && in_body(rest_span.start.byte)
                && !is_covered(rest_span.start.byte, rest_span.end.byte) =>
            {
                return Err(format!(
                    "{} signature at [{},{}] has no Python range: {:?}",
                    keyword,
                    rest_span.start.byte,
                    rest_span.end.byte,
                    &source[rest_span.start.byte..rest_span.end.byte]
                )
                .into());
            }
            Token::Expression { span, .. } if in_body(span.start.byte) => {
                // Skip slot expressions ({...} / {...name}) — tokenizer converts
                // these to {children} / {children_name} but source still has "..."
                let inner = &source[span.start.byte + 1..span.end.byte - 1];
                if inner.trim().starts_with("...") {
                    continue;
                }
                let inner_start = span.start.byte + 1;
                let inner_end = span.end.byte - 1;
                if inner_start < inner_end && !is_covered(inner_start, inner_end) {
                    return Err(format!(
                        "expression at [{},{}] has no Python range: {:?}",
                        inner_start,
                        inner_end,
                        &source[inner_start..inner_end]
                    )
                    .into());
                }
            }
            Token::PythonStatement { code, span, .. } if in_body(span.start.byte) => {
                // Skip renamed statements (class/type assignments) — generator
                // intentionally omits their range since the compiled code differs
                if code.starts_with("class ")
                    || code.starts_with("class=")
                    || code.starts_with("type ")
                    || code.starts_with("type=")
                {
                    continue;
                }
                // Skip multiline statements — continuation lines get re-indented,
                // so source != compiled and validate_python_ranges drops the range
                if code.contains('\n') {
                    continue;
                }
                if !is_covered(span.start.byte, span.end.byte) {
                    return Err(format!(
                        "statement at [{},{}] has no Python range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            Token::ControlStart {
                keyword, rest_span, ..
            } if in_body(rest_span.start.byte)
                && matches!(keyword.as_str(), "if" | "for" | "while" | "match" | "with") =>
            {
                // Generator trims trailing `:` from conditions
                let trimmed_end = trim_colon_end(&source, rest_span.start.byte, rest_span.end.byte);
                if !is_covered(rest_span.start.byte, trimmed_end) {
                    return Err(format!(
                        "{} condition at [{},{}] has no Python range: {:?}",
                        keyword,
                        rest_span.start.byte,
                        trimmed_end,
                        &source[rest_span.start.byte..trimmed_end]
                    )
                    .into());
                }
            }
            Token::ControlContinuation {
                keyword,
                rest_span: Some(rest_span),
                ..
            } if in_body(rest_span.start.byte) => {
                // Generator trims trailing `:` from continuation clauses
                let trimmed_end = trim_colon_end(&source, rest_span.start.byte, rest_span.end.byte);
                if !is_covered(rest_span.start.byte, trimmed_end) {
                    return Err(format!(
                        "{} clause at [{},{}] has no Python range: {:?}",
                        keyword,
                        rest_span.start.byte,
                        trimmed_end,
                        &source[rest_span.start.byte..trimmed_end]
                    )
                    .into());
                }
            }
            // Attribute expressions in HTML elements and components
            Token::HtmlElementOpen {
                attributes, span, ..
            }
            | Token::ComponentOpen {
                attributes, span, ..
            } if in_body(span.start.byte) => {
                for attr in attributes {
                    use hyper_transpiler::parser::tokenizer::AttributeValue;
                    // Get (inner_start, inner_end) excluding delimiters
                    let inner = match &attr.value {
                        // class={expr}: span is {expr}, inner skips { and }
                        AttributeValue::Expression(_, s) => {
                            Some((s.start.byte + 1, s.end.byte - 1))
                        }
                        // {name}: span is {name}, inner skips {
                        // (span.end is before }, so no -1)
                        AttributeValue::Shorthand(_, s) => Some((s.start.byte + 1, s.end.byte)),
                        _ => None,
                    };
                    if let Some((start, end)) = inner
                        && start < end
                        && !is_covered(start, end)
                    {
                        return Err(format!(
                            "attribute expression at [{},{}] has no Python range: {:?}",
                            start,
                            end,
                            &source[start..end]
                        )
                        .into());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
