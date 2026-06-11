use super::Visitor;
use crate::ast::{AttributeKind, Node};

/// Python keywords that cannot appear as bare identifiers in generated code, plus
/// builtins worth shadowing consistently so references resolve to the param.
const RESERVED: &[&str] = &["class", "type"];

/// Renames reserved keywords used as identifiers (`class` to `class_`) on params,
/// component-call kwargs, and expressions. Skips statements, so `class Foo:` stays valid.
pub struct ReservedKeywordPlugin;

/// Rename a whole identifier if reserved: `class` → `class_`.
fn safe_ident(name: &str) -> Option<String> {
    RESERVED.contains(&name).then(|| format!("{name}_"))
}

/// Rename a reserved keyword only when it leads the expression as a whole
/// identifier (`class`, `class.x`), leaving `classroom` and `f(class)` alone.
fn rename_leading(expr: &str) -> String {
    let lead = expr.len() - expr.trim_start().len();
    let (ws, body) = expr.split_at(lead);

    for kw in RESERVED {
        if let Some(rest) = body.strip_prefix(kw) {
            let at_boundary = rest
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric() && c != '_');
            if at_boundary {
                return format!("{ws}{kw}_{rest}");
            }
        }
    }
    expr.to_string()
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
    if let Some(renamed) = safe_ident(name) {
        *name = renamed;
    }
}

/// An attribute value expression references a surrounding param.
fn rename_value_expr(kind: &mut AttributeKind) {
    if let AttributeKind::Expression { expr, .. } = kind {
        *expr = rename_leading(expr);
    }
}

impl Visitor for ReservedKeywordPlugin {
    fn enter(&mut self, node: &mut Node, _metadata: &mut super::TransformMetadata) -> bool {
        match node {
            Node::Parameter(param) => {
                if let Some(renamed) = safe_ident(&param.name) {
                    param.name = renamed;
                }
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
                expr.expr = rename_leading(&expr.expr);
            }
            _ => {}
        }
        true
    }
}
