use super::{Flow, Plugin};
use crate::ast::{AttributeKind, Node};
use crate::error::CompileError;

/// Keywords that are syntax errors as bare identifiers and never valid inside an
/// expression, so renaming is always safe. Builtins like `type` are left alone.
const RESERVED: &[&str] = &["class"];

/// Renames reserved keywords used as identifiers (`class` to `class_`) on params,
/// component-call kwargs, and expressions. Skips statements, so `class Foo:` stays valid.
pub struct RenameReservedKeywords;

/// Rename every whole-word reserved keyword in a Python expression to its safe
/// form (`class` to `class_`). Skips string literals and attribute access.
pub fn rename_reserved_keywords(expr: &str) -> String {
    let chars: Vec<char> = expr.chars().collect();
    let mut out = String::with_capacity(expr.len() + 2);
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '"' || c == '\'' {
            i = copy_string_literal(&chars, i, &mut out);
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let is_attribute = start > 0 && chars[start - 1] == '.';
            out.push_str(&word);
            if !is_attribute && RESERVED.contains(&word.as_str()) {
                out.push('_');
            }
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

/// Copy a string literal (single/double/triple quoted, honoring `\` escapes)
/// verbatim. Returns the index just past the closing quote.
fn copy_string_literal(chars: &[char], start: usize, out: &mut String) -> usize {
    let quote = chars[start];
    let triple = start + 2 < chars.len() && chars[start + 1] == quote && chars[start + 2] == quote;
    let open = if triple { 3 } else { 1 };

    let mut i = start;
    for _ in 0..open {
        out.push(quote);
        i += 1;
    }

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            out.push(chars[i]);
            out.push(chars[i + 1]);
            i += 2;
            continue;
        }
        if triple {
            if i + 2 < chars.len() && (i..i + 3).all(|j| chars[j] == quote) {
                (0..3).for_each(|_| out.push(quote));
                return i + 3;
            }
        } else if chars[i] == quote {
            out.push(quote);
            return i + 1;
        }
        out.push(chars[i]);
        i += 1;
    }
    i
}

/// On a component call, an attribute name becomes a kwarg key.
fn rename_kwarg_key(kind: &mut AttributeKind) {
    let name = match kind {
        AttributeKind::Static { name, .. }
        | AttributeKind::Expression { name, .. }
        | AttributeKind::Template { name, .. }
        | AttributeKind::Boolean { name }
        | AttributeKind::Shorthand { name, .. } => name,
        AttributeKind::Spread { .. } | AttributeKind::SlotAssignment { .. } => return,
    };
    *name = rename_reserved_keywords(name);
}

/// An attribute value expression references surrounding params.
fn rename_value_expr(kind: &mut AttributeKind) {
    match kind {
        AttributeKind::Expression { expr, .. } | AttributeKind::Spread { expr, .. } => {
            *expr = rename_reserved_keywords(expr);
        }
        _ => {}
    }
}

impl Plugin for RenameReservedKeywords {
    fn enter(&mut self, node: &mut Node) -> Result<Flow, CompileError> {
        match node {
            Node::Parameter(param) => {
                param.name = rename_reserved_keywords(&param.name);
            }
            Node::Component(component) => {
                for attr in &mut component.attributes {
                    rename_kwarg_key(&mut attr.kind);
                    rename_value_expr(&mut attr.kind);
                }
            }
            Node::Element(element) => {
                for attr in &mut element.attributes {
                    rename_value_expr(&mut attr.kind);
                }
            }
            Node::Expression(expr) => {
                expr.expr = rename_reserved_keywords(&expr.expr);
            }
            Node::If(if_node) => {
                if_node.condition = rename_reserved_keywords(&if_node.condition);
                for (condition, _, _) in &mut if_node.elif_branches {
                    *condition = rename_reserved_keywords(condition);
                }
            }
            Node::For(for_node) => {
                for_node.binding = rename_reserved_keywords(&for_node.binding);
                for_node.iterable = rename_reserved_keywords(&for_node.iterable);
            }
            Node::While(while_node) => {
                while_node.condition = rename_reserved_keywords(&while_node.condition);
            }
            Node::With(with_node) => {
                with_node.items = rename_reserved_keywords(&with_node.items);
            }
            Node::Match(match_node) => {
                match_node.expr = rename_reserved_keywords(&match_node.expr);
            }
            _ => {}
        }
        Ok(Flow::Continue)
    }
}
