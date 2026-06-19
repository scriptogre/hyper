//! Template AST (`hyper`) is the parser's input IR; output AST (`python`) is
//! what the generator prints. Callers say `crate::ast::*` for template types.

mod hyper;
pub mod python;

pub use hyper::*;
