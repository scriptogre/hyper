use crate::helpers::{byte_to_utf16, compile};
use hyper::generate::Language;
use hyper::parse::tokenizer::{Token, tokenize};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

/// Validate that every HTML element open/close tag has corresponding HTML ranges.
pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;
    let tokens = tokenize(&source);

    let html_segments: Vec<_> = result
        .segments
        .iter()
        .filter(|s| s.language == Language::Html)
        .collect();

    // Find separator position to distinguish preamble vs body
    let separator_byte = tokens.iter().find_map(|t| {
        if let Token::Separator { range } = t {
            Some(range.start.byte)
        } else {
            None
        }
    });

    let in_body = |byte: usize| -> bool { separator_byte.is_none_or(|sep| byte > sep) };

    /// Check if any HTML segment overlaps a UTF-16 source range.
    /// "overlaps" rather than "covers" because HTML segments may split around
    /// expression attributes (e.g. `<div class={x}>` has separate segments
    /// for `<div class=` and `>`).
    fn has_html_coverage(
        html_segments: &[&hyper::generate::Segment],
        start_u16: usize,
        end_u16: usize,
    ) -> bool {
        html_segments
            .iter()
            .any(|s| s.source_start < end_u16 && s.source_end > start_u16)
    }

    for token in &tokens {
        match token {
            Token::HtmlElementOpen { range, .. } if in_body(range.start.byte) => {
                let start_u16 = byte_to_utf16(&source, range.start.byte);
                let end_u16 = byte_to_utf16(&source, range.end.byte);
                if !has_html_coverage(&html_segments, start_u16, end_u16) {
                    return Err(format!(
                        "HTML element open tag at [{},{}] has no HTML range: {:?}",
                        range.start.byte,
                        range.end.byte,
                        &source[range.start.byte..range.end.byte]
                    )
                    .into());
                }
            }
            Token::HtmlElementClose { range, .. } if in_body(range.start.byte) => {
                let start_u16 = byte_to_utf16(&source, range.start.byte);
                let end_u16 = byte_to_utf16(&source, range.end.byte);
                if !has_html_coverage(&html_segments, start_u16, end_u16) {
                    return Err(format!(
                        "HTML element close tag at [{},{}] has no HTML range: {:?}",
                        range.start.byte,
                        range.end.byte,
                        &source[range.start.byte..range.end.byte]
                    )
                    .into());
                }
            }
            // Component and slot tags don't require full HTML coverage:
            // - The component name gets a Python injection range
            // - Attributes (if any) get HTML ranges with a synthetic "<x" prefix
            // - Closing tags and slot names are handled by the TextMate grammar
            // So we intentionally skip the coverage check for these token types.
            _ => {}
        }
    }
    Ok(())
}
