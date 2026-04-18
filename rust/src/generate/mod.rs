mod injection_analyzer;
mod output;
mod python;

pub use injection_analyzer::InjectionAnalyzer;
pub use output::{
    ExpressionBrace, Injection, Mapping, Output, Range, RangeType, compute_injections,
    convert_braces_to_utf16, validate_python_ranges,
};
pub use python::PythonGenerator;

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
    pub expression_braces: Vec<ExpressionBrace>,
}

/// Generator trait - converts AST to code
pub trait Generator {
    fn generate(
        &self,
        ast: &Ast,
        metadata: &TransformMetadata,
        options: &GenerateOptions,
    ) -> GenerateResult;
}
