//! Expression brace position collector and tag highlight collector.
//!
//! Walks the AST to find all `{` / `}` brace pairs that delimit expressions,
//! component names, slot references, and attribute interpolations. These positions
//! are used by the IDE to provide brace matching and highlighting.
//!
//! Also collects tag highlight positions for component/slot tags — the `<`, `{`, `}`,
//! `>`, `/>` punctuation and the component/slot names within them.

use super::output::TagHighlightKind;
use crate::ast::*;

/// Collect all expression brace positions (byte offsets) from the AST.
/// Returns `(open_byte, close_byte)` pairs for each expression brace pair.
pub fn collect_expression_braces(ast: &Ast) -> Vec<(usize, usize)> {
    let mut braces = Vec::new();
    for node in &ast.nodes {
        collect_braces_node(node, &mut braces);
    }
    braces
}

fn collect_braces_node(node: &Node, braces: &mut Vec<(usize, usize)>) {
    match node {
        Node::Expression(expr) => {
            // span covers {expr} with exclusive end
            braces.push((expr.span.start.byte, expr.span.end.byte - 1));
        }
        Node::Element(el) => {
            for attr in &el.attributes {
                collect_braces_attr(attr, braces);
            }
            for child in &el.children {
                collect_braces_node(child, braces);
            }
        }
        Node::Component(c) => {
            // Opening tag <{Name}>: { is before name_span, } is at name_span.end
            braces.push((c.name_span.start.byte - 1, c.name_span.end.byte));
            // Closing tag </{Name}>: { at start+2, } at end-2
            if let Some(ref cs) = c.close_span {
                braces.push((cs.start.byte + 2, cs.end.byte - 2));
            }
            for attr in &c.attributes {
                collect_braces_attr(attr, braces);
            }
            for child in &c.children {
                collect_braces_node(child, braces);
            }
        }
        Node::Fragment(f) => {
            for child in &f.children {
                collect_braces_node(child, braces);
            }
        }
        Node::Slot(s) => {
            if s.close_span.is_some() {
                // Tag-form slot <{...name}>: { at start+1, } at end-2
                braces.push((s.span.start.byte + 1, s.span.end.byte - 2));
                // Closing tag </{...name}>: { at start+2, } at end-2
                if let Some(ref cs) = s.close_span {
                    braces.push((cs.start.byte + 2, cs.end.byte - 2));
                }
            } else {
                // Inline slot {...}: span covers {..} with exclusive end
                braces.push((s.span.start.byte, s.span.end.byte - 1));
            }
            for child in &s.fallback {
                collect_braces_node(child, braces);
            }
        }
        Node::If(if_node) => {
            for child in &if_node.then_branch {
                collect_braces_node(child, braces);
            }
            for (_, _, body) in &if_node.elif_branches {
                for child in body {
                    collect_braces_node(child, braces);
                }
            }
            if let Some(else_branch) = &if_node.else_branch {
                for child in else_branch {
                    collect_braces_node(child, braces);
                }
            }
        }
        Node::For(for_node) => {
            for child in &for_node.body {
                collect_braces_node(child, braces);
            }
        }
        Node::Match(match_node) => {
            for case in &match_node.cases {
                for child in &case.body {
                    collect_braces_node(child, braces);
                }
            }
        }
        Node::While(while_node) => {
            for child in &while_node.body {
                collect_braces_node(child, braces);
            }
        }
        Node::With(with_node) => {
            for child in &with_node.body {
                collect_braces_node(child, braces);
            }
        }
        Node::Try(try_node) => {
            for child in &try_node.body {
                collect_braces_node(child, braces);
            }
            for except in &try_node.except_clauses {
                for child in &except.body {
                    collect_braces_node(child, braces);
                }
            }
            if let Some(else_clause) = &try_node.else_clause {
                for child in else_clause {
                    collect_braces_node(child, braces);
                }
            }
            if let Some(finally_clause) = &try_node.finally_clause {
                for child in finally_clause {
                    collect_braces_node(child, braces);
                }
            }
        }
        Node::Definition(def) => {
            for child in &def.body {
                collect_braces_node(child, braces);
            }
        }
        _ => {} // Text, Comment, Statement, Import, Parameter, Decorator
    }
}

#[allow(clippy::while_let_on_iterator)]
fn collect_braces_attr(attr: &Attribute, braces: &mut Vec<(usize, usize)>) {
    match &attr.kind {
        AttributeKind::Expression { expr_span, .. } => {
            // expr_span covers {expr} with exclusive end
            braces.push((expr_span.start.byte, expr_span.end.byte - 1));
        }
        AttributeKind::Shorthand { expr_span, .. } | AttributeKind::Spread { expr_span, .. } => {
            // expr_span.end points TO closing brace (not past it)
            braces.push((expr_span.start.byte, expr_span.end.byte));
        }
        AttributeKind::SlotAssignment {
            expr_span: Some(span),
            ..
        } => {
            // expr_span.end points TO closing brace
            braces.push((span.start.byte, span.end.byte));
        }
        AttributeKind::Template { name, value } => {
            // Walk value to find {expr} brace positions
            let value_start_byte = attr.span.start.byte + name.len() + 2; // skip `name="`
            let mut byte_offset = 0;
            let mut chars = value.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '{' {
                    let open_byte = value_start_byte + byte_offset;
                    byte_offset += ch.len_utf8();
                    let mut depth = 1;
                    while let Some(inner) = chars.next() {
                        byte_offset += inner.len_utf8();
                        if inner == '{' {
                            depth += 1;
                        } else if inner == '}' {
                            depth -= 1;
                            if depth == 0 {
                                let close_byte = value_start_byte + byte_offset - 1;
                                braces.push((open_byte, close_byte));
                                break;
                            }
                        }
                    }
                } else {
                    byte_offset += ch.len_utf8();
                }
            }
        }
        _ => {}
    }
}

// ─── Tag highlight collector ──────────────────────────────────────────

/// Collect component/slot tag highlight positions (byte offsets) from the AST.
/// Returns `(start, end, kind)` triples for punctuation (`<`, `{`, `}`, `>`, `/>`)
/// and names (component name, slot `...`, slot name).
pub fn collect_tag_highlights(ast: &Ast) -> Vec<(usize, usize, TagHighlightKind)> {
    let mut highlights = Vec::new();
    for node in &ast.nodes {
        collect_tag_highlights_node(node, &mut highlights);
    }
    highlights
}

fn collect_tag_highlights_node(
    node: &Node,
    highlights: &mut Vec<(usize, usize, TagHighlightKind)>,
) {
    match node {
        Node::Component(c) => {
            highlight_component_tag(c, highlights);
            for child in &c.children {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::Slot(s) => {
            highlight_slot_tag(s, highlights);
            for child in &s.fallback {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::Element(el) => {
            for child in &el.children {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::Fragment(f) => {
            for child in &f.children {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::If(if_node) => {
            for child in &if_node.then_branch {
                collect_tag_highlights_node(child, highlights);
            }
            for (_, _, body) in &if_node.elif_branches {
                for child in body {
                    collect_tag_highlights_node(child, highlights);
                }
            }
            if let Some(else_branch) = &if_node.else_branch {
                for child in else_branch {
                    collect_tag_highlights_node(child, highlights);
                }
            }
        }
        Node::For(for_node) => {
            for child in &for_node.body {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::While(while_node) => {
            for child in &while_node.body {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::Match(match_node) => {
            for case in &match_node.cases {
                for child in &case.body {
                    collect_tag_highlights_node(child, highlights);
                }
            }
        }
        Node::With(with_node) => {
            for child in &with_node.body {
                collect_tag_highlights_node(child, highlights);
            }
        }
        Node::Try(try_node) => {
            for child in &try_node.body {
                collect_tag_highlights_node(child, highlights);
            }
            for except in &try_node.except_clauses {
                for child in &except.body {
                    collect_tag_highlights_node(child, highlights);
                }
            }
            if let Some(else_clause) = &try_node.else_clause {
                for child in else_clause {
                    collect_tag_highlights_node(child, highlights);
                }
            }
            if let Some(finally_clause) = &try_node.finally_clause {
                for child in finally_clause {
                    collect_tag_highlights_node(child, highlights);
                }
            }
        }
        Node::Definition(def) => {
            for child in &def.body {
                collect_tag_highlights_node(child, highlights);
            }
        }
        _ => {}
    }
}

/// Emit highlights for a component tag: `<{Name} ... />` or `<{Name}>...</{Name}>`
fn highlight_component_tag(
    c: &ComponentNode,
    highlights: &mut Vec<(usize, usize, TagHighlightKind)>,
) {
    let tag_start = c.span.start.byte;
    let brace_open = c.name_span.start.byte - 1; // `{`
    let brace_close = c.name_span.end.byte; // `}`
    let tag_end = c.span.end.byte;

    // `<` before `{`
    highlights.push((tag_start, brace_open, TagHighlightKind::TagPunctuation));
    // `{`
    highlights.push((brace_open, brace_open + 1, TagHighlightKind::TagPunctuation));
    // Component name
    highlights.push((
        c.name_span.start.byte,
        c.name_span.end.byte,
        TagHighlightKind::ComponentName,
    ));
    // `}`
    highlights.push((
        brace_close,
        brace_close + 1,
        TagHighlightKind::TagPunctuation,
    ));
    // `>` or `/>` at end of opening tag.
    // Self-closing components (no children, no close_span) always end in `/>`,
    // so we highlight the last 2 bytes. Components with bodies end in `>`.
    if c.children.is_empty() && c.close_span.is_none() {
        highlights.push((tag_end - 2, tag_end, TagHighlightKind::TagPunctuation));
    } else {
        highlights.push((tag_end - 1, tag_end, TagHighlightKind::TagPunctuation));
    }

    // Closing tag: </{Name}>
    if let Some(ref cs) = c.close_span {
        let cs_start = cs.start.byte;
        let cs_end = cs.end.byte;
        // `</`
        highlights.push((cs_start, cs_start + 2, TagHighlightKind::TagPunctuation));
        // `{`
        highlights.push((cs_start + 2, cs_start + 3, TagHighlightKind::TagPunctuation));
        // Name
        let close_name_start = cs_start + 3;
        let close_name_end = cs_end - 2;
        if close_name_end > close_name_start {
            highlights.push((
                close_name_start,
                close_name_end,
                TagHighlightKind::ComponentName,
            ));
        }
        // `}`
        highlights.push((cs_end - 2, cs_end - 1, TagHighlightKind::TagPunctuation));
        // `>`
        highlights.push((cs_end - 1, cs_end, TagHighlightKind::TagPunctuation));
    }
}

/// Emit highlights for a slot tag: `<{...name}>` or `<{...name}>...</{...name}>`
fn highlight_slot_tag(s: &SlotNode, highlights: &mut Vec<(usize, usize, TagHighlightKind)>) {
    // Only tag-form slots (with close_span) get highlights
    if s.close_span.is_none() {
        return;
    }

    let tag_start = s.span.start.byte;
    let tag_end = s.span.end.byte;
    let brace_open = tag_start + 1; // `{` after `<`
    let brace_close = tag_end - 2; // `}` before `>`

    // Opening tag: `<{...name}>`
    // `<`
    highlights.push((tag_start, tag_start + 1, TagHighlightKind::TagPunctuation));
    // `{`
    highlights.push((brace_open, brace_open + 1, TagHighlightKind::TagPunctuation));
    // `...` keyword
    let dots_start = brace_open + 1;
    if let Some(ref name) = s.name {
        highlights.push((dots_start, dots_start + 3, TagHighlightKind::SlotKeyword));
        // Slot name
        let name_start = dots_start + 3;
        let name_end = name_start + name.len();
        highlights.push((name_start, name_end, TagHighlightKind::SlotName));
    } else {
        // Default slot: `{...}`
        highlights.push((dots_start, dots_start + 3, TagHighlightKind::SlotKeyword));
    }
    // `}`
    highlights.push((
        brace_close,
        brace_close + 1,
        TagHighlightKind::TagPunctuation,
    ));
    // `>`
    highlights.push((tag_end - 1, tag_end, TagHighlightKind::TagPunctuation));

    // Closing tag
    if let Some(ref cs) = s.close_span {
        let cs_start = cs.start.byte;
        let cs_end = cs.end.byte;
        // `</`
        highlights.push((cs_start, cs_start + 2, TagHighlightKind::TagPunctuation));
        // `{`
        highlights.push((cs_start + 2, cs_start + 3, TagHighlightKind::TagPunctuation));
        // `...` keyword
        let dots_start = cs_start + 3;
        if let Some(ref name) = s.name {
            highlights.push((dots_start, dots_start + 3, TagHighlightKind::SlotKeyword));
            let name_start = dots_start + 3;
            let name_end = name_start + name.len();
            highlights.push((name_start, name_end, TagHighlightKind::SlotName));
        } else {
            highlights.push((dots_start, dots_start + 3, TagHighlightKind::SlotKeyword));
        }
        // `}`
        highlights.push((cs_end - 2, cs_end - 1, TagHighlightKind::TagPunctuation));
        // `>`
        highlights.push((cs_end - 1, cs_end, TagHighlightKind::TagPunctuation));
    }
}
