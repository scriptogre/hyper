//! Injection analyzer — computes IDE injection ranges from AST and generated code.
//!
//! This module handles:
//! - Computing prefix/suffix injections from ranges (for JetBrains virtual files)
//! - Building HTML injection ranges for element and component tags

use super::output::{Injection, Range, RangeType, compute_injections};
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

    /// Post-process ranges to compute injection prefix/suffix.
    pub fn analyze(
        &self,
        _ast: &Ast,
        code: &str,
        source: &str,
        ranges: Vec<Range>,
    ) -> (Vec<Range>, Vec<Injection>) {
        let injections = compute_injections(code, source, &ranges);
        (ranges, injections)
    }
}

// ─── HTML range builders ───────────────────────────────────────────────
//
// These produce RangeType::Html ranges for the static parts of element
// and component/slot tags, skipping over embedded expression spans.
// HTML ranges don't need compiled positions (compiled_start/end = 0)
// because the virtual HTML file is built from source text directly.

/// Build HTML injection ranges for an element's opening and closing tags.
///
/// The opening tag span covers `<tag attrs>` or `<tag attrs />`.
/// The closing tag span covers `</tag>`.
/// Returns ranges for the static HTML parts, with gaps for expression attributes.
pub fn html_ranges_for_element(el: &ElementNode) -> Vec<Range> {
    let mut ranges = Vec::new();

    // Collect expression spans (exclusive end) within the opening tag.
    // Dynamic spans already use exclusive end (past '}').
    // Shorthand/SlotAssignment spans end AT '}', so we +1 for exclusive end.
    let mut expr_spans = Vec::new();
    for attr in &el.attributes {
        match &attr.kind {
            AttributeKind::Expression { expr_span, .. } => {
                // Include the = sign before { so virtual HTML sees a boolean attr
                let gap_start = expr_span.start.byte.saturating_sub(1);
                expr_spans.push((gap_start, expr_span.end.byte));
            }
            AttributeKind::Shorthand { expr_span, .. }
            | AttributeKind::Spread { expr_span, .. } => {
                expr_spans.push((expr_span.start.byte, expr_span.end.byte + 1));
            }
            AttributeKind::SlotAssignment {
                expr_span: Some(span),
                ..
            } => {
                // Include the = sign before { so virtual HTML sees a boolean attr
                let gap_start = span.start.byte.saturating_sub(1);
                expr_spans.push((gap_start, span.end.byte + 1));
            }
            AttributeKind::Template { name, value } => {
                // Walk value to find {expr} positions, exclude them from HTML ranges
                let value_start_byte = attr.span.start.byte + name.len() + 2;
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
    let tag_start = el.span.start.byte;
    let tag_end = el.span.end.byte;
    let mut pos = tag_start;

    for (expr_start, expr_end) in &expr_spans {
        if *expr_start > pos && *expr_start <= tag_end {
            ranges.push(Range {
                range_type: RangeType::Html,
                source_start: pos,
                source_end: *expr_start,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
            });
        }
        if *expr_end > pos {
            pos = *expr_end;
        }
    }

    // Remaining static part of opening tag
    if pos < tag_end {
        ranges.push(Range {
            range_type: RangeType::Html,
            source_start: pos,
            source_end: tag_end,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
        });
    }

    // Closing tag range (e.g. </div>)
    if let Some(close_span) = &el.close_span {
        ranges.push(Range {
            range_type: RangeType::Html,
            source_start: close_span.start.byte,
            source_end: close_span.end.byte,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
        });
    }

    ranges
}

/// Build HTML injection ranges for component/slot tag angle brackets.
///
/// For `<{Card}>`, creates ranges for `<` and `>`, skipping `{Card}`.
/// For `</{Card}>`, creates ranges for `</` and `>`, skipping `{Card}`.
pub fn html_ranges_for_component(
    open_span: &Span,
    close_span: Option<&Span>,
    brace_open: usize,
    brace_close: usize,
) -> Vec<Range> {
    let mut ranges = Vec::new();

    // Opening tag: "<" before the brace
    let lt_start = open_span.start.byte;
    if brace_open > lt_start {
        ranges.push(Range {
            range_type: RangeType::Html,
            source_start: lt_start,
            source_end: brace_open,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
        });
    }

    // Opening tag: ">" after the brace
    let gt_pos = open_span.end.byte - 1;
    if gt_pos > brace_close {
        ranges.push(Range {
            range_type: RangeType::Html,
            source_start: brace_close + 1,
            source_end: open_span.end.byte,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
        });
    }

    // Closing tag
    if let Some(cs) = close_span {
        // Closing tag is like </{Card}> or </{...header}>
        // "</" is at cs.start.byte..cs.start.byte+2
        // "{" is at cs.start.byte+2
        // "}" is at cs.end.byte-2
        // ">" is at cs.end.byte-1
        let close_brace_open = cs.start.byte + 2;
        let close_brace_close = cs.end.byte - 2;

        // "</" before brace
        ranges.push(Range {
            range_type: RangeType::Html,
            source_start: cs.start.byte,
            source_end: close_brace_open,
            compiled_start: 0,
            compiled_end: 0,
            needs_injection: true,
        });

        // ">" after brace
        if cs.end.byte > close_brace_close + 1 {
            ranges.push(Range {
                range_type: RangeType::Html,
                source_start: close_brace_close + 1,
                source_end: cs.end.byte,
                compiled_start: 0,
                compiled_end: 0,
                needs_injection: true,
            });
        }
    }

    ranges
}
