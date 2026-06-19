//! Printer for output AST nodes. Renders to source text byte-identical to the
//! prior `format!` paths; caller owns trailing newlines.

use crate::ast::python::{Alias, StmtImportFrom};

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

fn print_alias(alias: &Alias) -> String {
    match &alias.asname {
        Some(asname) => format!("{} as {}", alias.name.id, asname.id),
        None => alias.name.id.clone(),
    }
}
