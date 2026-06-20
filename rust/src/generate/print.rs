//! Printer for output AST nodes. Renders to source text byte-identical to the
//! prior `format!` paths; caller owns trailing newlines.

use crate::ast::python::{Alias, Code, Expr, StmtImportFrom};
use crate::generate::{Language, Output, Segment};

pub fn print_import_from(stmt: &StmtImportFrom) -> String {
    let module = stmt.module.as_ref().map(|m| m.id.as_str()).unwrap_or("");
    let dots = ".".repeat(stmt.level as usize);
    let names = stmt
        .names
        .iter()
        .map(print_alias)
        .collect::<Vec<_>>()
        .join(", ");
    format!("from {dots}{module} import {names}")
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

/// Print an output expression. `Code` records its own source segment via
/// `print_code`; synthetic scaffolding (`escape`, parens) carries no range.
pub fn print_expr(output: &mut Output, expr: &Expr) {
    match expr {
        Expr::Name(name) => output.push(&name.id.id),
        Expr::StringLiteral(s) => {
            output.push("\"");
            output.push(&s.value);
            output.push("\"");
        }
        Expr::Code(code) => print_code(output, code),
        Expr::Call(call) => {
            // Every Call we build wraps a helper; user calls stay inside Code.
            if let Expr::Name(func) = call.func.as_ref() {
                output.use_helper(&func.id.id);
            }
            print_expr(output, &call.func);
            output.push("(");
            for (i, arg) in call.arguments.args.iter().enumerate() {
                if i > 0 {
                    output.push(", ");
                }
                print_expr(output, arg);
            }
            output.push(")");
        }
    }
}

fn print_alias(alias: &Alias) -> String {
    match &alias.asname {
        Some(asname) => format!("{} as {}", alias.name.id, asname.id),
        None => alias.name.id.clone(),
    }
}
