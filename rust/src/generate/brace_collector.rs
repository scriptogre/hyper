//! Expression brace position collector.
//!
//! Walks the AST to find all `{` / `}` brace pairs that delimit expressions,
//! component names, slot references, and attribute interpolations. These positions
//! are used by the IDE to provide brace matching and highlighting.

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
