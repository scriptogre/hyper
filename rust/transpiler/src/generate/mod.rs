mod python;
mod output;
mod injection_analyzer;

pub use python::PythonGenerator;
pub use output::{Output, Mapping, Range, RangeType, Injection, compute_injections};
pub use injection_analyzer::InjectionAnalyzer;

use crate::ast::Ast;
use crate::transform::TransformMetadata;

/// Generator options
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    pub function_name: Option<String>,
    pub include_ranges: bool,
}

/// Generation result
#[derive(Debug, Clone)]
pub struct GenerateResult {
    pub code: String,
    pub mappings: Vec<Mapping>,
    pub ranges: Vec<Range>,
    pub injections: Vec<Injection>,
}

/// Generator trait - converts AST to code
pub trait Generator {
    fn generate(&self, ast: &Ast, metadata: &TransformMetadata, options: &GenerateOptions) -> GenerateResult;
}
