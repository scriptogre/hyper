//! Printer for output nodes. Ruff codegen prints compiler-generated Python;
//! user `Code` is pushed verbatim. Caller owns trailing newlines.

use ruff_python_ast::str::Quote;
use ruff_python_ast::{
    Alias, Arguments, AtomicNodeIndex, Expr, ExprCall, ExprContext, ExprName, ExprStringLiteral,
    Identifier, Stmt, StmtImportFrom, StringLiteral, StringLiteralFlags, StringLiteralValue,
};
use ruff_python_codegen::{Generator, Indentation};
use ruff_text_size::TextRange;

use crate::ast::python::{Code, HelperCall};
use crate::generate::{Language, Output, Segment};

/// Stands in for user `Code` while Ruff prints the generated call around it.
const CODE_PLACEHOLDER: &str = "__hyper_code__";

/// `from <module> import <names>`, built as a Ruff node and printed by Ruff.
pub fn print_import_from(module: &str, names: &[&str]) -> String {
    let stmt = Stmt::ImportFrom(StmtImportFrom {
        node_index: AtomicNodeIndex::default(),
        range: TextRange::default(),
        module: Some(Identifier::new(module, TextRange::default())),
        names: names
            .iter()
            .map(|name| Alias {
                range: TextRange::default(),
                node_index: AtomicNodeIndex::default(),
                name: Identifier::new(*name, TextRange::default()),
                asname: None,
            })
            .collect(),
        level: 0,
        is_lazy: false,
    });
    Generator::new(&Indentation::default(), Default::default()).stmt(&stmt)
}

/// Write verbatim source. Record a Python-injection segment when the range
/// is real, so source maps fall out of printing for any node carrying `Code`.
pub fn print_code(output: &mut Output, code: &Code) {
    let start = output.position();
    output.push(&code.source);
    let end = output.position();
    if !code.range.is_synthetic() {
        output.add_segment(Segment {
            language: Language::Python,
            source_start: code.range.start.byte,
            source_end: code.range.end.byte,
            compiled_start: start,
            compiled_end: end,
            needs_injection: true,
            html_prefix: None,
        });
    }
}

/// Print a helper call. Ruff prints the call around a placeholder; the user
/// `Code` replaces it verbatim and records the only source segment.
pub fn print_helper_call(output: &mut Output, call: &HelperCall) {
    let attr_name = call.attr_name.as_deref().map(|name| {
        Expr::StringLiteral(ExprStringLiteral {
            node_index: AtomicNodeIndex::default(),
            range: TextRange::default(),
            value: StringLiteralValue::single(StringLiteral {
                range: TextRange::default(),
                node_index: AtomicNodeIndex::default(),
                value: name.into(),
                flags: StringLiteralFlags::empty().with_quote_style(Quote::Double),
            }),
        })
    });
    let expression = Expr::Call(ExprCall {
        node_index: AtomicNodeIndex::default(),
        range: TextRange::default(),
        func: Box::new(build_name(call.helper)),
        arguments: Arguments {
            range: TextRange::default(),
            node_index: AtomicNodeIndex::default(),
            args: attr_name
                .into_iter()
                .chain([build_name(CODE_PLACEHOLDER)])
                .collect(),
            keywords: Box::default(),
        },
    });
    let generated = Generator::new(&Indentation::default(), Default::default()).expr(&expression);
    // The placeholder is the last argument, so the rightmost match is it.
    let (prefix, suffix) = generated
        .rsplit_once(CODE_PLACEHOLDER)
        .expect("Ruff prints the placeholder argument");
    output.use_helper(call.helper);
    output.push(prefix);
    print_code(output, &call.arg);
    output.push(suffix);
}

fn build_name(id: &str) -> Expr {
    Expr::Name(ExprName {
        node_index: AtomicNodeIndex::default(),
        range: TextRange::default(),
        id: id.into(),
        ctx: ExprContext::Load,
    })
}
