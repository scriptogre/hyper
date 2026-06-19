//! Injection analyzer — computes IDE injection ranges from AST and generated code.
//!
//! This module handles:
//! - Computing prefix/suffix injections from ranges (for JetBrains virtual files)
//! - Building HTML injection ranges for element and component tags

use super::output::{Injection, Language, Segment, compute_injections};
use crate::ast::*;

/// Analyzes AST and generated code to produce injection ranges and injections.
pub struct InjectionAnalyzer;

impl Default for InjectionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl InjectionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Post-process segments to compute injection prefix/suffix.
    pub fn analyze(
        &self,
        _ast: &Ast,
        code: &str,
        source: &str,
        segments: Vec<Segment>,
    ) -> (Vec<Segment>, Vec<Injection>) {
        let injections = compute_injections(code, source, &segments);
        (segments, injections)
    }
}

// ─── HTML range builders ───────────────────────────────────────────────
//
// These produce Language::Html ranges for the static parts of element
// and component/slot tags, skipping over embedded expression spans.
// HTML ranges don't need compiled positions (compiled_start/end = 0)
// because the virtual HTML file is built from source text directly.

/// Collect expression spans from component/slot attributes that must be
/// excluded from HTML ranges.  Returns `(start, exclusive_end)` byte pairs.
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

/// Build HTML injection ranges for an element's opening and closing tags.
///
/// The opening tag range covers `<tag attrs>` or `<tag attrs />`.
/// The closing tag range covers `</tag>`.
/// Returns ranges for the static HTML parts, with gaps for expression attributes.
pub fn html_ranges_for_element(el: &ElementNode) -> Vec<Segment> {
    let mut ranges = Vec::new();

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
                // Walk value to find {expr} positions, exclude them from HTML ranges
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

    // Create HTML ranges for the gaps between expressions within the opening tag
    let tag_start = el.range.start.byte;
    let tag_end = el.range.end.byte;
    let mut pos = tag_start;

    for (expr_start, expr_end) in &expr_spans {
        if *expr_start > pos && *expr_start <= tag_end {
            ranges.push(Segment {
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
        ranges.push(Segment {
            language: Language::Html,
            source_start: pos,
            source_end: tag_end,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
            html_prefix: None,
        });
    }

    // Closing tag range (e.g. </div>)
    if let Some(close_range) = &el.close_range {
        ranges.push(Segment {
            language: Language::Html,
            source_start: close_range.start.byte,
            source_end: close_range.end.byte,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
            html_prefix: None,
        });
    }

    ranges
}

/// Build HTML injection ranges for component/slot tag angle brackets.
///
/// For `<{Card}>`, creates ranges for `<` and `>`, skipping `{Card}`.
/// For `</{Card}>`, creates ranges for `</` and `>`, skipping `{Card}`.
///
/// The `attr_expr_spans` parameter contains (start, exclusive_end) byte positions
/// for expression attributes within the opening tag that must be excluded from
/// HTML ranges (to avoid overlapping with Python injection ranges).
pub fn html_ranges_for_component(
    open_range: &TextRange,
    _close_range: Option<&TextRange>,
    _brace_open: usize,
    brace_close: usize,
    attr_expr_spans: &[(usize, usize)],
) -> Vec<Segment> {
    let mut ranges = Vec::new();

    // NOTE: We intentionally do NOT emit the lone "<" before the component name
    // brace as an HTML range. A lone "<" is unparseable HTML and would pollute
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
                ranges.push(Segment {
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
            ranges.push(Segment {
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
    // We don't emit HTML ranges for the closing tag fragments ("</" and ">")
    // because they're unparseable HTML on their own and would pollute the
    // virtual HTML document. The component/slot closing tag gets its coloring
    // from the TextMate grammar / annotator instead.

    ranges
}
