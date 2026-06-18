//! Render a combinable run of content nodes (text / expression / element) into
//! Python f-string *source*, which the lowering pass then parses into a real
//! Ruff `ExprFString` (or `ExprStringLiteral`) node.
//!
//! This mirrors the rendering rules of the string generator
//! (`generate/python.rs`): the same HTML literal text, the same `{escape(...)}`
//! interpolations, the same attribute-helper selection
//! (`render_class`/`render_style`/`render_attr`/`render_data`/`render_aria`/
//! `spread_attrs`). Producing source and re-parsing it keeps the IR a real AST
//! while reusing Ruff's parser for the embedded Python — the same technique the
//! control-flow lowering uses for headers.
//!
//! Whitespace/indentation *formatting* of the f-string (the `f"""\` dedented
//! style) is deliberately NOT reproduced here; that is the job of the
//! source-map-aware printer (Phase 4). This module emits structurally-correct
//! f-strings whose rendered HTML is equivalent.

use crate::ast::{Attribute, AttributeKind, ElementNode, Node};

/// A rendered run: the inner f-string content plus whether it contains any
/// interpolations (which decides the `f` prefix).
pub struct Rendered {
    pub content: String,
    pub has_expr: bool,
}

/// Does this node (or a descendant) contain a Python interpolation?
pub fn node_has_expressions(node: &Node) -> bool {
    match node {
        Node::Expression(_) => true,
        Node::Element(el) => {
            el.attributes.iter().any(|attr| {
                !matches!(
                    attr.kind,
                    AttributeKind::Static { .. } | AttributeKind::Boolean { .. }
                )
            }) || el.children.iter().any(node_has_expressions)
        }
        _ => false,
    }
}

/// Can this node be combined into a single string literal? Components, slots,
/// control flow, statements, etc. break the run.
pub fn is_combinable(node: &Node) -> bool {
    match node {
        Node::Text(_) | Node::Expression(_) => true,
        Node::Element(el) => el.children.iter().all(is_combinable),
        _ => false,
    }
}

/// Render a run of combinable nodes into f-string content.
pub fn render_run(nodes: &[&Node]) -> Rendered {
    let has_expr = nodes.iter().any(|n| node_has_expressions(n));
    let mut content = String::new();
    for node in nodes {
        render_node(node, &mut content, has_expr);
    }
    Rendered { content, has_expr }
}

fn render_node(node: &Node, out: &mut String, in_fstring: bool) {
    match node {
        Node::Text(text) => {
            if in_fstring {
                // Escape braces so they stay literal inside the f-string.
                out.push_str(&text.content.replace('{', "{{").replace('}', "}}"));
            } else {
                out.push_str(&text.content);
            }
        }
        Node::Expression(expr) if in_fstring => {
            let has_format_extras =
                expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
            if has_format_extras {
                out.push('{');
                out.push_str(&expr.expr);
                if expr.debug {
                    out.push('=');
                }
                if let Some(conv) = expr.conversion {
                    out.push('!');
                    out.push(conv);
                }
                if let Some(spec) = &expr.format_spec {
                    out.push(':');
                    out.push_str(spec);
                }
                out.push('}');
            } else if expr.escape {
                out.push_str("{escape(");
                out.push_str(expr.expr.trim());
                out.push_str(")}");
            } else {
                out.push('{');
                out.push_str(expr.expr.trim());
                out.push('}');
            }
        }
        Node::Element(el) => render_element(el, out, in_fstring),
        _ => {}
    }
}

/// Render just an element's open tag (`<tag attrs>` or `<tag attrs />`), without
/// its children. Used when the element is not combinable (it has a component /
/// slot / control-flow child) and must yield its tags around lowered children.
pub fn render_open_tag(el: &ElementNode) -> Rendered {
    let has_expr = el.attributes.iter().any(|attr| {
        matches!(
            attr.kind,
            AttributeKind::Expression { .. }
                | AttributeKind::Template { .. }
                | AttributeKind::Shorthand { .. }
                | AttributeKind::Spread { .. }
        )
    });
    let mut content = String::from("<");
    content.push_str(&el.tag);
    for attr in &el.attributes {
        render_attribute(attr, &mut content, has_expr);
    }
    if el.self_closing {
        content.push_str(" />");
    } else {
        content.push('>');
    }
    Rendered { content, has_expr }
}

fn render_element(el: &ElementNode, out: &mut String, in_fstring: bool) {
    out.push('<');
    out.push_str(&el.tag);
    for attr in &el.attributes {
        render_attribute(attr, out, in_fstring);
    }
    if el.self_closing {
        out.push_str(" />");
    } else {
        out.push('>');
        for child in &el.children {
            render_node(child, out, in_fstring);
        }
        out.push_str("</");
        out.push_str(&el.tag);
        out.push('>');
    }
}

fn render_attribute(attr: &Attribute, out: &mut String, in_fstring: bool) {
    match &attr.kind {
        AttributeKind::Static { name, value } => {
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            out.push_str(&value.replace('"', "&quot;"));
            out.push('"');
        }
        AttributeKind::Boolean { name } => {
            out.push(' ');
            out.push_str(name);
        }
        AttributeKind::Expression { name, expr, .. } if in_fstring => {
            let safe = expr.trim();
            if name == "class" {
                out.push_str(&format!(" class=\"{{render_class({safe})}}\""));
            } else if name == "style" {
                out.push_str(&format!(" style=\"{{render_style({safe})}}\""));
            } else if crate::html::is_boolean_attribute(name) {
                out.push_str(&format!("{{render_attr(\"{name}\", {safe})}}"));
            } else {
                out.push_str(&format!(" {name}=\"{{escape({safe})}}\""));
            }
        }
        AttributeKind::Shorthand { name, .. } if in_fstring => {
            let var = crate::plugins::rename_reserved_keywords(name);
            if name == "class" {
                out.push_str(&format!(" class=\"{{render_class({var})}}\""));
            } else if name == "style" {
                out.push_str(&format!(" style=\"{{render_style({var})}}\""));
            } else if name == "data" {
                out.push_str(&format!("{{render_data({var})}}"));
            } else if name == "aria" {
                out.push_str(&format!("{{render_aria({var})}}"));
            } else {
                out.push_str(&format!("{{render_attr(\"{name}\", {var})}}"));
            }
        }
        AttributeKind::Spread { expr, .. } if in_fstring => {
            out.push_str(&format!("{{spread_attrs({})}}", expr.trim()));
        }
        AttributeKind::SlotAssignment { name, expr, .. } => match expr {
            Some(e) if in_fstring => {
                out.push_str(&format!(" slot:{name}=\"{{{e}}}\""));
            }
            Some(_) => {}
            None => {
                out.push_str(&format!(" slot:{name}"));
            }
        },
        AttributeKind::Template { name, value } if in_fstring => {
            out.push(' ');
            out.push_str(name);
            out.push_str("=\"");
            render_template_value(value, out);
            out.push('"');
        }
        // Expression-bearing kinds outside an f-string cannot occur (their
        // presence forces `has_expr`); nothing to emit.
        _ => {}
    }
}

/// Escape a string for embedding in a double-quoted Python string literal.
pub fn escape_python_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Convert a template attribute value into f-string content for a component
/// keyword argument (`{expr}` → `{escape(expr)}`, `"` → `&quot;`). Unlike
/// [`render_template_value`], this does not rename reserved keywords (matching
/// the string generator's `convert_template_expressions`).
pub fn component_template_value(value: &str) -> String {
    let mut out = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                let mut expr = String::new();
                let mut depth = 1;
                for inner in chars.by_ref() {
                    if inner == '{' {
                        depth += 1;
                        expr.push(inner);
                    } else if inner == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(inner);
                    } else {
                        expr.push(inner);
                    }
                }
                out.push_str(&format!("{{escape({expr})}}"));
            }
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Render a component's attributes as Python keyword-argument source, joined by
/// `, ` (e.g. `a="1", b=x, **rest`). Slot-assignment attributes are handled by
/// the slot mechanism and skipped here.
pub fn component_kwargs(attrs: &[Attribute]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for attr in attrs {
        match &attr.kind {
            AttributeKind::Static { name, value } => {
                parts.push(format!("{name}=\"{}\"", escape_python_string(value)));
            }
            AttributeKind::Expression { name, expr, .. } => {
                parts.push(format!("{name}={}", expr.trim()));
            }
            AttributeKind::Boolean { name } => parts.push(format!("{name}=True")),
            AttributeKind::Shorthand { name, .. } => {
                let var = crate::plugins::rename_reserved_keywords(name);
                parts.push(format!("{name}={var}"));
            }
            AttributeKind::Spread { expr, .. } => parts.push(format!("**{}", expr.trim())),
            AttributeKind::Template { name, value } => {
                parts.push(format!("{name}=f\"{}\"", component_template_value(value)));
            }
            AttributeKind::SlotAssignment { .. } => {}
        }
    }
    parts.join(", ")
}

/// Render a mixed template attribute value (`"{expr} static"`), turning `{expr}`
/// markers into `{escape(expr)}` interpolations and escaping literal quotes.
fn render_template_value(value: &str, out: &mut String) {
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                let mut expr = String::new();
                let mut depth = 1;
                for inner in chars.by_ref() {
                    if inner == '{' {
                        depth += 1;
                        expr.push(inner);
                    } else if inner == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr.push(inner);
                    } else {
                        expr.push(inner);
                    }
                }
                let safe = crate::plugins::rename_reserved_keywords(expr.trim());
                out.push_str(&format!("{{escape({safe})}}"));
            }
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
}
