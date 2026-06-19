//! Printer for output AST nodes. Renders to source text byte-identical to the
//! prior `format!` paths; caller owns trailing newlines.

use crate::ast::python::{Alias, Code, StmtImportFrom};
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

fn print_alias(alias: &Alias) -> String {
    match &alias.asname {
        Some(asname) => format!("{} as {}", alias.name.id, asname.id),
        None => alias.name.id.clone(),
    }
}
