use crate::helpers::compile;
use hyper_transpiler::generate::RangeType;
use hyper_transpiler::parser::tokenizer::{Token, tokenize};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

/// Validate that every HTML element open/close tag has corresponding HTML ranges.
pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;
    let tokens = tokenize(&source);

    let html_ranges: Vec<_> = result
        .ranges
        .iter()
        .filter(|r| r.range_type == RangeType::Html)
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

    /// Check if any HTML range overlaps with a source byte range.
    /// We use "overlaps" rather than "covers" because HTML ranges may be split
    /// around expression attributes (e.g., `<div class={x}>` has separate ranges
    /// for `<div class=` and `>`).
    fn has_html_coverage(
        html_ranges: &[&hyper_transpiler::generate::Range],
        start: usize,
        end: usize,
    ) -> bool {
        html_ranges
            .iter()
            .any(|r| r.source_start < end && r.source_end > start)
    }

    for token in &tokens {
        match token {
            Token::HtmlElementOpen { span, .. } if in_body(span.start.byte) => {
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "HTML element open tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            Token::HtmlElementClose { span, .. } if in_body(span.start.byte) => {
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "HTML element close tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            Token::ComponentOpen {
                span, self_closing, ..
            } if in_body(span.start.byte) => {
                // Component tags like <{Card}> should have HTML ranges for the
                // angle brackets (< and >) around the braced name.
                // Self-closing components (<{Card} />) also need coverage.
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "component open tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
                let _ = self_closing; // suppress unused warning
            }
            Token::ComponentClose { span, .. } if in_body(span.start.byte) => {
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "component close tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            Token::SlotOpen { name, span } if name.is_some() && in_body(span.start.byte) => {
                // Tag-form named slots like <{...header}> should have HTML ranges.
                // Inline slots ({...} or {...name}) don't get HTML ranges — they're
                // expressions, not tags.
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "slot open tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            Token::SlotClose { span, .. } if in_body(span.start.byte) => {
                if !has_html_coverage(&html_ranges, span.start.byte, span.end.byte) {
                    return Err(format!(
                        "slot close tag at [{},{}] has no HTML range: {:?}",
                        span.start.byte,
                        span.end.byte,
                        &source[span.start.byte..span.end.byte]
                    )
                    .into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
