//! Lowering pass: hyper AST → Ruff Python AST (`ModModule`).
//!
//! This is where the intelligence that used to live in the string-based code
//! generator moves. Instead of concatenating Python text, we build a real
//! Python AST that downstream plugins transform and a source-map-aware printer
//! renders.
//!
//! The lowering is intentionally "dumb" about program-level concerns that the
//! plugins own (async-ness, slot parameters, helper imports, mutable-default
//! guards, `**kwargs` spreads). It produces the structural skeleton — user
//! imports, the `@html` decorator, the function signature with its declared
//! parameters, and the lowered body — and lets the plugin passes refine it.
//!
//! ## Status
//!
//! Phase 1 (this module): outer structure + a subset of body nodes. Body node
//! kinds that are not yet lowered to real Python AST are emitted as transitional
//! string-constant `yield`s (clearly marked) so the pipeline stays whole while
//! Phase 2 fills them in. The existing string-based generator remains the
//! default; this path is exercised by unit tests via [`crate::compile_via_ast`].

pub mod builders;

use ruff_python_ast::{self as ast, Stmt};

use crate::ast::{Ast, Node, ParameterNode};
use crate::error::CompileError;
use builders as b;

/// Convert a snake_case file stem into the PascalCase component name.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Partitioned top-level nodes of a hyper template.
struct Partition<'a> {
    imports: Vec<&'a crate::ast::ImportNode>,
    params: Vec<&'a ParameterNode>,
    /// Orphan decorators applied to the outer template function.
    decorators: Vec<&'a crate::ast::DecoratorNode>,
    body: Vec<&'a Node>,
}

/// Split the flat top-level node list into header (imports/params/decorators)
/// and body, mirroring the partitioning the string generator performs.
fn partition(nodes: &[Node]) -> Partition<'_> {
    let mut imports = Vec::new();
    let mut params = Vec::new();
    let mut decorators = Vec::new();
    let mut body = Vec::new();

    // A decorator that is immediately followed (modulo comments/blank text) by a
    // definition belongs to that definition and stays in the body; otherwise it
    // is an orphan decorator applied to the outer `@html` function.
    let mut decorator_leads_to_def = vec![false; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        if matches!(node, Node::Decorator(_)) {
            for next in &nodes[i + 1..] {
                match next {
                    Node::Decorator(_) | Node::Comment(_) => continue,
                    Node::Text(t) if t.content.trim().is_empty() => continue,
                    Node::Definition(_) => {
                        decorator_leads_to_def[i] = true;
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    for (i, node) in nodes.iter().enumerate() {
        match node {
            Node::Parameter(p) => params.push(p),
            Node::Import(im) => imports.push(im),
            Node::Decorator(d) if !decorator_leads_to_def[i] => decorators.push(d),
            _ => body.push(node),
        }
    }

    Partition {
        imports,
        params,
        decorators,
        body,
    }
}

/// Lower a parameter declaration into a keyword-only function parameter.
fn lower_parameter(param: &ParameterNode) -> Result<ast::ParameterWithDefault, CompileError> {
    let range = b::span_range(param.span);
    let annotation = match &param.type_hint {
        Some(hint) => Some(b::parse_expr(hint)?),
        None => None,
    };
    let default = match &param.default {
        Some(def) => Some(b::parse_expr(def)?),
        None => None,
    };
    Ok(b::kwonly_param(&param.name, range, annotation, default))
}

/// Lower a single hyper body node into zero or more Python statements.
///
/// Phase 1 lowers the structurally simple, self-contained node kinds. The
/// remaining kinds (elements, components, control flow, slots, definitions) are
/// emitted as transitional string-constant `yield`s pending Phase 2.
fn lower_node(node: &Node) -> Result<Vec<Stmt>, CompileError> {
    match node {
        // Blank/structural-only text between header items produces nothing.
        Node::Text(t) if t.content.is_empty() => Ok(vec![]),

        // A Python statement parses straight through into real statements.
        Node::Statement(s) => b::parse_stmts(&s.stmt),

        // A bare expression yields its (optionally escaped) value.
        Node::Expression(e) => {
            let expr = b::parse_expr(&e.expr)?;
            let value = if e.escape {
                let escape_fn = b::name_expr("escape", b::SENTINEL);
                b::call(escape_fn, vec![expr], vec![], b::span_range(e.span))
            } else {
                expr
            };
            Ok(vec![b::expr_stmt(b::yield_expr(value), b::span_range(e.span))])
        }

        // Comments carry no runtime effect in the lowered Python.
        Node::Comment(_) => Ok(vec![]),

        // Transitional: kinds not yet lowered to real Python AST are emitted as
        // string-constant yields so the module stays well-formed (Phase 2).
        other => Ok(vec![transitional_yield(other)]),
    }
}

/// A placeholder `yield "<…>"` for a body node kind Phase 2 will lower properly.
fn transitional_yield(node: &Node) -> Stmt {
    let label = match node {
        Node::Text(_) => "text",
        Node::Element(_) => "element",
        Node::Component(_) => "component",
        Node::Fragment(_) => "fragment",
        Node::Slot(_) => "slot",
        Node::If(_) => "if",
        Node::For(_) => "for",
        Node::Match(_) => "match",
        Node::While(_) => "while",
        Node::With(_) => "with",
        Node::Try(_) => "try",
        Node::Definition(_) => "definition",
        _ => "node",
    };
    let placeholder = b::string_literal(&format!("__hyper_todo__:{label}"), b::SENTINEL);
    b::expr_stmt(b::yield_expr(placeholder), b::SENTINEL)
}

/// Lower a hyper [`Ast`] into a Ruff [`ast::ModModule`].
///
/// `function_name` is the (snake_case) file stem; it is PascalCased to form the
/// component function name.
pub fn lower(ast: &Ast, function_name: Option<&str>) -> Result<ast::ModModule, CompileError> {
    let part = partition(&ast.nodes);

    let mut module_body: Vec<Stmt> = Vec::new();

    // 1. User imports, parsed straight through.
    for import in &part.imports {
        module_body.extend(b::parse_stmts(&import.stmt)?);
    }

    // 2. The generated `from hyper import html` (helper plugin will extend this).
    module_body.push(b::import_from("hyper", &[("html", None)], b::SENTINEL));

    // 3. Function parameters (keyword-only, after the bare `*`).
    let mut parameters = b::empty_parameters();
    for param in &part.params {
        parameters.kwonlyargs.push(lower_parameter(param)?);
    }

    // 4. Function body.
    let mut func_body: Vec<Stmt> = Vec::new();
    for node in &part.body {
        func_body.extend(lower_node(node)?);
    }
    if func_body.is_empty() {
        func_body.push(b::pass_stmt());
    }

    // 5. Decorators: user orphan decorators first, then `@html`.
    let mut decorators: Vec<ast::Decorator> = Vec::new();
    for dec in &part.decorators {
        let text = dec.decorator.trim_start_matches('@');
        let expr = b::parse_expr(text)?;
        decorators.push(b::decorator(expr, b::span_range(dec.span)));
    }
    decorators.push(b::decorator(b::name_expr("html", b::SENTINEL), b::SENTINEL));

    let name = function_name.map(to_pascal_case).unwrap_or_else(|| "Render".to_string());

    module_body.push(b::function_def(
        &name,
        /* is_async */ false,
        decorators,
        parameters,
        func_body,
    ));

    Ok(b::module(module_body))
}

#[cfg(test)]
mod tests {
    use crate::compile_via_ast;

    #[test]
    fn lowers_only_params() {
        let src = "name: str\ncount: int = 0\nitems: list\n\n---\n";
        let out = compile_via_ast(src, Some("only_params")).unwrap();
        println!("=== only_params ===\n{out}");
        assert!(out.contains("def OnlyParams"));
        assert!(out.contains("from hyper import html"));
    }

    #[test]
    fn lowers_imports_and_expressions() {
        let src = "from datetime import datetime\nimport json\n\nname: str\n\n---\n\n<p>{datetime.now().isoformat()}</p>\n";
        let out = compile_via_ast(src, Some("imports")).unwrap();
        println!("=== imports ===\n{out}");
        assert!(out.contains("from datetime import datetime"));
        assert!(out.contains("import json"));
    }
}
