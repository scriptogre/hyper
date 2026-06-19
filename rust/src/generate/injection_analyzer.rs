//! HTML segment builders for element, component, and slot tags. They emit
//! `Language::Html` segments for static tag parts; compiled positions are unused.

use super::output::{Language, Segment};
use crate::ast::*;

/// Collect expression spans from component/slot attributes that must be
/// excluded from HTML segments.  Returns `(start, exclusive_end)` byte pairs.
pub fn collect_component_attr_expr_spans(attrs: &[Attribute]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    for attr in attrs {
        match &attr.kind {
            AttributeKind::Expression { expr_range, .. } => {
                // Include `={…}` — gap starts at the `=` before `{`
                let gap_start = expr_range.start.byte.saturating_sub(1);
                spans.push((gap_start, expr_range.end.byte));
            }
            AttributeKind::Shorthand { expr_range, .. }
            | AttributeKind::Spread { expr_range, .. } => {
                spans.push((expr_range.start.byte, expr_range.end.byte + 1));
            }
            AttributeKind::SlotAssignment {
                expr_range: Some(range),
                ..
            } => {
                let gap_start = range.start.byte.saturating_sub(1);
                spans.push((gap_start, range.end.byte + 1));
            }
            _ => {}
        }
    }
    spans
}

/// Build HTML injection segments for an element's opening and closing tags.
///
/// The opening tag segment covers `<tag attrs>` or `<tag attrs />`.
/// The closing tag segment covers `</tag>`.
/// Returns segments for the static HTML parts, with gaps for expression attributes.
pub fn html_segments_for_element(el: &ElementNode) -> Vec<Segment> {
    let mut segments = Vec::new();

    // Collect expression spans (exclusive end) within the opening tag.
    // Dynamic spans already use exclusive end (past '}').
    // Shorthand/SlotAssignment spans end AT '}', so we +1 for exclusive end.
    let mut expr_spans = Vec::new();
    for attr in &el.attributes {
        match &attr.kind {
            AttributeKind::Expression { expr_range, .. } => {
                // Include the = sign before { so virtual HTML sees a boolean attr
                let gap_start = expr_range.start.byte.saturating_sub(1);
                expr_spans.push((gap_start, expr_range.end.byte));
            }
            AttributeKind::Shorthand { expr_range, .. }
            | AttributeKind::Spread { expr_range, .. } => {
                expr_spans.push((expr_range.start.byte, expr_range.end.byte + 1));
            }
            AttributeKind::SlotAssignment {
                expr_range: Some(range),
                ..
            } => {
                // Include the = sign before { so virtual HTML sees a boolean attr
                let gap_start = range.start.byte.saturating_sub(1);
                expr_spans.push((gap_start, range.end.byte + 1));
            }
            AttributeKind::Template { name, value } => {
                // Walk value to find {expr} positions, exclude them from HTML segments
                let value_start_byte = attr.range.start.byte + name.len() + 2;
                let mut byte_offset = 0;
                let mut chars = value.chars().peekable();
                #[allow(clippy::while_let_on_iterator)]
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        let gap_start = value_start_byte + byte_offset;
                        byte_offset += ch.len_utf8();
                        let mut depth = 1;
                        while let Some(inner) = chars.next() {
                            byte_offset += inner.len_utf8();
                            if inner == '{' {
                                depth += 1;
                            } else if inner == '}' {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                        }
                        let gap_end = value_start_byte + byte_offset;
                        expr_spans.push((gap_start, gap_end));
                    } else {
                        byte_offset += ch.len_utf8();
                    }
                }
            }
            _ => {}
        }
    }

    // Sort by start position
    expr_spans.sort_by_key(|s| s.0);

    // Create HTML segments for the gaps between expressions within the opening tag
    let tag_start = el.range.start.byte;
    let tag_end = el.range.end.byte;
    let mut pos = tag_start;

    for (expr_start, expr_end) in &expr_spans {
        if *expr_start > pos && *expr_start <= tag_end {
            segments.push(Segment {
                language: Language::Html,
                source_start: pos,
                source_end: *expr_start,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
                html_prefix: None,
            });
        }
        if *expr_end > pos {
            pos = *expr_end;
        }
    }

    // Remaining static part of opening tag
    if pos < tag_end {
        segments.push(Segment {
            language: Language::Html,
            source_start: pos,
            source_end: tag_end,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
            html_prefix: None,
        });
    }

    // Closing tag segment (e.g. </div>)
    if let Some(close_range) = &el.close_range {
        segments.push(Segment {
            language: Language::Html,
            source_start: close_range.start.byte,
            source_end: close_range.end.byte,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
            html_prefix: None,
        });
    }

    segments
}

/// Build HTML injection segments for component/slot tag angle brackets.
///
/// For `<{Card}>`, creates segments for `<` and `>`, skipping `{Card}`.
/// For `</{Card}>`, creates segments for `</` and `>`, skipping `{Card}`.
///
/// The `attr_expr_spans` parameter contains (start, exclusive_end) byte positions
/// for expression attributes within the opening tag that must be excluded from
/// HTML segments (to avoid overlapping with Python injection segments).
pub fn html_segments_for_component(
    open_range: &TextRange,
    _close_range: Option<&TextRange>,
    _brace_open: usize,
    brace_close: usize,
    attr_expr_spans: &[(usize, usize)],
) -> Vec<Segment> {
    let mut segments = Vec::new();

    // NOTE: We intentionally do NOT emit the lone "<" before the component name
    // brace as an HTML segment. A lone "<" is unparseable HTML and would pollute
    // the virtual HTML document. Instead, we give the attribute-region fragments
    // an html_prefix of "<x" so the virtual HTML sees a valid tag like
    // `<x text="Sale" />`, enabling attribute highlighting.

    // Opening tag: region after the component name brace to end of tag,
    // split around any attribute expression spans.
    // The first fragment gets html_prefix "<x" for tag context.
    let after_brace = brace_close + 1;
    let tag_end = open_range.end.byte;

    if tag_end > after_brace {
        // Collect and sort attribute expression spans that fall in this region
        let mut spans: Vec<(usize, usize)> = attr_expr_spans
            .iter()
            .filter(|(s, e)| *s >= after_brace && *e <= tag_end)
            .copied()
            .collect();
        spans.sort_by_key(|s| s.0);

        let mut first = true;
        let mut pos = after_brace;
        for (expr_start, expr_end) in &spans {
            if *expr_start > pos {
                segments.push(Segment {
                    language: Language::Html,
                    source_start: pos,
                    source_end: *expr_start,
                    compiled_start: 0,
                    compiled_end: 0,
                    needs_injection: true,
                    html_prefix: if first {
                        first = false;
                        Some("<x".into())
                    } else {
                        None
                    },
                });
            }
            if *expr_end > pos {
                pos = *expr_end;
            }
        }

        // Remaining static part after last expression
        if pos < tag_end {
            segments.push(Segment {
                language: Language::Html,
                source_start: pos,
                source_end: tag_end,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
                html_prefix: if first { Some("<x".into()) } else { None },
            });
        }
    }

    // Closing tag: </{Card}> or </{...header}>
    // We don't emit HTML segments for the closing tag fragments ("</" and ">")
    // because they're unparseable HTML on their own and would pollute the
    // virtual HTML document. The component/slot closing tag gets its coloring
    // from the TextMate grammar / annotator instead.

    segments
}
