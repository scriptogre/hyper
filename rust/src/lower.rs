//! Lower the flat node stream into the template's [`Function`], splitting
//! frontmatter (params, imports, orphaned decorators, header comments) from the
//! body. Runs once, between parse and the plugins, so later stages read a
//! structured function instead of re-deriving it.

use std::sync::Arc;

use crate::ast::python::{Arguments, Code, Expr, ExprCall, ExprName, Identifier, StringLiteral};
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
    Some(helper_call("escape", interp_code(&expr.expr, expr.range)))
}

/// `Code` spanning explicit source bytes. Source text is independent of the span
/// (they differ after keyword renames), so callers pass both.
pub fn code_span(source: impl Into<String>, start_byte: usize, end_byte: usize) -> Code {
    Code {
        source: source.into(),
        range: TextRange {
            start: Position {
                byte: start_byte,
                line: 0,
                col: 0,
            },
            end: Position {
                byte: end_byte,
                line: 0,
                col: 0,
            },
        },
    }
}

/// `Code` for an interpolation: source is the printed expr text; range is the
/// `{expr}` span minus its braces. Synthetic stays synthetic.
fn interp_code(source: &str, brace_range: TextRange) -> Code {
    if brace_range.is_synthetic() {
        return Code {
            source: source.to_string(),
            range: brace_range,
        };
    }
    code_span(source, brace_range.start.byte + 1, brace_range.end.byte - 1)
}

/// `helper(arg)` where `arg` is verbatim user `Code`. Used for single-argument
/// helpers (`escape`, `render_class`, `render_style`, `render_data`,
/// `render_aria`, `spread_attrs`).
pub fn helper_call(name: &str, arg: Code) -> Expr {
    Expr::Call(ExprCall {
        func: Box::new(Expr::Name(ExprName {
            id: Identifier::new(name),
        })),
        arguments: Arguments {
            args: vec![Expr::Code(arg)],
        },
    })
}

/// `render_attr("name", arg)`: the static attribute name plus the user `Code`.
pub fn render_attr_call(attr_name: &str, arg: Code) -> Expr {
    Expr::Call(ExprCall {
        func: Box::new(Expr::Name(ExprName {
            id: Identifier::new("render_attr"),
        })),
        arguments: Arguments {
            args: vec![
                Expr::StringLiteral(StringLiteral {
                    value: attr_name.to_string(),
                }),
                Expr::Code(arg),
            ],
        },
    })
}
