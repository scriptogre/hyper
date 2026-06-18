//! Terse constructors for Ruff Python AST nodes.
//!
//! Every Ruff AST node carries a `node_index` and a `range`. For synthetic
//! nodes (generated imports, the `@html` decorator, slot parameters, guard
//! statements) we have no `.hyper` source position, so they get the
//! [`SENTINEL`] range. Nodes that map back to user-written Python carry a real
//! [`TextRange`] derived from the originating hyper [`Span`].
//!
//! The source-map-aware printer (Phase 4) distinguishes the two: a non-sentinel
//! range means "this output maps back to `.hyper` source at these bytes".

use ruff_python_ast::{self as ast, AtomicNodeIndex, Expr, Stmt};
use ruff_python_parser::{parse_expression, parse_module};
use ruff_text_size::{TextRange, TextSize};

use crate::ast::Span;
use crate::error::CompileError;

/// Range used for synthetic nodes that have no `.hyper` source origin.
///
/// `TextRange::default()` is `0..0`. Real source spans always have
/// `start < end` for non-empty content, and byte 0 of a `.hyper` file is never
/// the start of an interesting Python node (it is the header), so an empty
/// range at offset 0 is a safe sentinel.
pub const SENTINEL: TextRange = TextRange::new(TextSize::new(0), TextSize::new(0));

/// Is this range the synthetic sentinel (no source mapping)?
pub fn is_sentinel(range: TextRange) -> bool {
    range == SENTINEL
}

/// Convert a hyper [`Span`] (byte offsets) into a Ruff [`TextRange`].
pub fn span_range(span: Span) -> TextRange {
    let start = TextSize::try_from(span.start.byte).unwrap_or(TextSize::new(0));
    let end = TextSize::try_from(span.end.byte).unwrap_or(start);
    if end < start {
        TextRange::empty(start)
    } else {
        TextRange::new(start, end)
    }
}

/// Parse a Python expression fragment into a Ruff [`Expr`].
///
/// The fragment originated from the `.hyper` source and was already validated
/// by the tokenizer, so a parse failure here is an internal error.
pub fn parse_expr(source: &str) -> Result<Expr, CompileError> {
    parse_expression(source)
        .map(|parsed| parsed.into_syntax().body.as_ref().clone())
        .map_err(|e| {
            CompileError::Generate(format!("failed to parse expression `{source}`: {e}"))
        })
}

/// Parse Python statements into a list of Ruff [`Stmt`]s (a suite).
pub fn parse_stmts(source: &str) -> Result<Vec<Stmt>, CompileError> {
    parse_module(source)
        .map(|parsed| parsed.into_syntax().body.into_iter().collect())
        .map_err(|e| CompileError::Generate(format!("failed to parse statement `{source}`: {e}")))
}

/// `name` as a `Load`-context [`Expr::Name`].
pub fn name_expr(name: &str, range: TextRange) -> Expr {
    Expr::Name(ast::ExprName {
        node_index: AtomicNodeIndex::NONE,
        range,
        id: ast::name::Name::new(name),
        ctx: ast::ExprContext::Load,
    })
}

/// An [`Identifier`] with the given range.
pub fn ident(name: &str, range: TextRange) -> ast::Identifier {
    ast::Identifier::new(name, range)
}

/// A keyword-only [`ParameterWithDefault`]: `name: annotation = default`.
pub fn kwonly_param(
    name: &str,
    name_range: TextRange,
    annotation: Option<Expr>,
    default: Option<Expr>,
) -> ast::ParameterWithDefault {
    ast::ParameterWithDefault {
        range: name_range,
        node_index: AtomicNodeIndex::NONE,
        parameter: ast::Parameter {
            range: name_range,
            node_index: AtomicNodeIndex::NONE,
            name: ident(name, name_range),
            annotation: annotation.map(Box::new),
        },
        default: default.map(Box::new),
    }
}

/// An empty [`Parameters`] list.
pub fn empty_parameters() -> ast::Parameters {
    ast::Parameters {
        range: SENTINEL,
        node_index: AtomicNodeIndex::NONE,
        posonlyargs: Default::default(),
        args: Default::default(),
        vararg: None,
        kwonlyargs: Default::default(),
        kwarg: None,
    }
}

/// A bare [`Parameter`] (no default), used for `**kwargs` / `*args`.
pub fn bare_param(name: &str, annotation: Option<Expr>) -> ast::Parameter {
    ast::Parameter {
        range: SENTINEL,
        node_index: AtomicNodeIndex::NONE,
        name: ident(name, SENTINEL),
        annotation: annotation.map(Box::new),
    }
}

/// A [`Decorator`] wrapping an expression.
pub fn decorator(expression: Expr, range: TextRange) -> ast::Decorator {
    ast::Decorator {
        range,
        node_index: AtomicNodeIndex::NONE,
        expression,
    }
}

/// `from <module> import <names>` where each name is `(name, asname)`.
pub fn import_from(module: &str, names: &[(&str, Option<&str>)], range: TextRange) -> Stmt {
    Stmt::ImportFrom(ast::StmtImportFrom {
        node_index: AtomicNodeIndex::NONE,
        range,
        module: Some(ident(module, range)),
        names: names
            .iter()
            .map(|(name, asname)| ast::Alias {
                range,
                node_index: AtomicNodeIndex::NONE,
                name: ident(name, range),
                asname: asname.map(|a| ident(a, range)),
            })
            .collect(),
        level: 0,
        is_lazy: false,
    })
}

/// The `None` literal expression.
pub fn none_literal() -> Expr {
    Expr::NoneLiteral(ast::ExprNoneLiteral {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
    })
}

/// A simple assignment `target = value`.
pub fn assign(target: &str, value: Expr) -> Stmt {
    Stmt::Assign(ast::StmtAssign {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
        targets: vec![Expr::Name(ast::ExprName {
            node_index: AtomicNodeIndex::NONE,
            range: SENTINEL,
            id: ast::name::Name::new(target),
            ctx: ast::ExprContext::Store,
        })],
        value: Box::new(value),
    })
}

/// A `pass` statement.
pub fn pass_stmt() -> Stmt {
    Stmt::Pass(ast::StmtPass {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
    })
}

/// Wrap an expression as an expression statement.
pub fn expr_stmt(value: Expr, range: TextRange) -> Stmt {
    Stmt::Expr(ast::StmtExpr {
        node_index: AtomicNodeIndex::NONE,
        range,
        value: Box::new(value),
    })
}

/// A `yield <value>` expression.
pub fn yield_expr(value: Expr) -> Expr {
    Expr::Yield(ast::ExprYield {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
        value: Some(Box::new(value)),
    })
}

/// A `yield from <value>` expression.
pub fn yield_from_expr(value: Expr) -> Expr {
    Expr::YieldFrom(ast::ExprYieldFrom {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
        value: Box::new(value),
    })
}

/// A call `func(args..., keywords...)`.
pub fn call(func: Expr, args: Vec<Expr>, keywords: Vec<ast::Keyword>, range: TextRange) -> Expr {
    Expr::Call(ast::ExprCall {
        node_index: AtomicNodeIndex::NONE,
        range,
        func: Box::new(func),
        arguments: ast::Arguments {
            range: SENTINEL,
            node_index: AtomicNodeIndex::NONE,
            args: args.into_boxed_slice(),
            keywords: keywords.into_boxed_slice(),
        },
    })
}

/// A string literal expression.
pub fn string_literal(value: &str, range: TextRange) -> Expr {
    Expr::StringLiteral(ast::ExprStringLiteral {
        node_index: AtomicNodeIndex::NONE,
        range,
        value: ast::StringLiteralValue::single(ast::StringLiteral {
            node_index: AtomicNodeIndex::NONE,
            range,
            value: value.into(),
            flags: ast::StringLiteralFlags::empty(),
        }),
    })
}

/// The outer `@html def Name(...): body` function definition.
pub fn function_def(
    name: &str,
    is_async: bool,
    decorators: Vec<ast::Decorator>,
    parameters: ast::Parameters,
    body: Vec<Stmt>,
) -> Stmt {
    Stmt::FunctionDef(ast::StmtFunctionDef {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
        is_async,
        decorator_list: decorators.into_iter().collect(),
        name: ident(name, SENTINEL),
        type_params: None,
        parameters: Box::new(parameters),
        returns: None,
        body: body.into_iter().collect(),
    })
}

/// Build a [`ast::ModModule`] from a list of top-level statements.
pub fn module(body: Vec<Stmt>) -> ast::ModModule {
    ast::ModModule {
        node_index: AtomicNodeIndex::NONE,
        range: SENTINEL,
        body: body.into_iter().collect(),
    }
}
