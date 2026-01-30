//! Injection analyzer - computes IDE injection ranges from AST and generated code

use crate::ast::Ast;
use super::output::{Range, Injection, compute_injections};

/// Analyzes AST and generated code to produce injection ranges for IDE support
pub struct InjectionAnalyzer;

impl InjectionAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze AST and generated code to compute injection ranges and injections
    ///
    /// Note: Ranges are collected during code generation with correct positions.
    /// This method post-processes them to compute injection prefix/suffix.
    pub fn analyze(&self, _ast: &Ast, code: &str, ranges: Vec<Range>) -> (Vec<Range>, Vec<Injection>) {
        let injections = compute_injections(code, &ranges);
        (ranges, injections)
    }
}
