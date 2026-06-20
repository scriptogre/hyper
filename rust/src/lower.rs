//! Lower the flat node stream into the template's [`Function`], splitting
//! frontmatter (params, imports, orphaned decorators, header comments) from the
//! body. Runs once, between parse and the plugins, so later stages read a
//! structured function instead of re-deriving it.

use std::sync::Arc;

use crate::ast::python::{Arguments, Code, Expr, ExprCall, ExprName, Identifier};
use crate::ast::{Ast, ExpressionNode, Function, Node, Position, TextRange};

pub fn lower(nodes: Vec<Node>, source: &str) -> Ast {
    let n = nodes.len();

    // A decorator "leads to a definition" when only decorators, comments, or
    // blank lines sit between it and a def. Such decorators stay in the body
    // with their def; orphaned ones apply to the outer function.
    let mut decorator_leads_to_def = vec![false; n];
    let mut whitespace_in_decorator_chain = vec![false; n];

    for i in 0..n {
        if !matches!(nodes[i], Node::Decorator(_)) {
            continue;
        }

        let mut found_def = false;
        for node in &nodes[i + 1..] {
            match node {
                Node::Decorator(_) | Node::Comment(_) => continue,
                Node::Text(t) if t.content.trim().is_empty() => continue,
                Node::Definition(_) => {
                    found_def = true;
                    break;
                }
                _ => break,
            }
        }
        decorator_leads_to_def[i] = found_def;

        if found_def {
            for (offset, node) in nodes[i + 1..].iter().enumerate() {
                match node {
                    Node::Text(t) if t.content.trim().is_empty() => {
                        whitespace_in_decorator_chain[i + 1 + offset] = true;
                    }
                    Node::Decorator(_) | Node::Comment(_) => continue,
                    _ => break,
                }
            }
        }
    }

    let mut params = Vec::new();
    let mut imports = Vec::new();
    let mut decorators = Vec::new();
    let mut header_comments = Vec::new();
    let mut body = Vec::new();
    let mut in_header = true;

    for (i, node) in nodes.into_iter().enumerate() {
        match node {
            Node::Parameter(p) => params.push(Node::Parameter(p)),
            Node::Import(import) => imports.push(import),
            Node::Comment(c) if in_header && params.is_empty() && imports.is_empty() => {
                header_comments.push(c)
            }
            Node::Decorator(dec) => {
                in_header = false;
                if decorator_leads_to_def[i] {
                    body.push(Node::Decorator(dec));
                } else {
                    decorators.push(dec);
                }
            }
            Node::Text(t) if whitespace_in_decorator_chain[i] && t.content.trim().is_empty() => {}
            other => {
                in_header = false;
                body.push(other);
            }
        }
    }

    Ast::new(
        Function {
            is_async: false,
            params,
            imports,
            decorators,
            header_comments,
            body,
        },
        Arc::from(source),
    )
}

/// Lower a template interpolation to an output expression for the cases already
/// migrated to the output AST. Returns None for cases the old generator path
/// still emits (raw `{x}`, `str(...)`, format-spec/conversion/debug).
pub fn lower_interpolation(expr: &ExpressionNode) -> Option<Expr> {
    let has_format_extras = expr.format_spec.is_some() || expr.conversion.is_some() || expr.debug;
    if has_format_extras || !expr.escape {
        return None;
    }
    Some(escape_call(interp_code(&expr.expr, expr.range)))
}

/// `Code` for an interpolation: source is the printed expr text; range is the
/// `{expr}` span minus its braces. Synthetic stays synthetic. Source length is
/// independent of the span (they differ after keyword renames), so this is not
/// `code_from`.
fn interp_code(source: &str, brace_range: TextRange) -> Code {
    let range = if brace_range.is_synthetic() {
        brace_range
    } else {
        TextRange {
            start: Position {
                byte: brace_range.start.byte + 1,
                ..brace_range.start
            },
            end: Position {
                byte: brace_range.end.byte - 1,
                ..brace_range.end
            },
        }
    };
    Code {
        source: source.to_string(),
        range,
    }
}

fn escape_call(arg: Code) -> Expr {
    Expr::Call(ExprCall {
        func: Box::new(Expr::Name(ExprName {
            id: Identifier::new("escape"),
        })),
        arguments: Arguments {
            args: vec![Expr::Code(arg)],
        },
    })
}
