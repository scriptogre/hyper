mod brace_collector;
mod html_segments;
mod output;
mod print;
mod python;

pub use brace_collector::collect_expression_braces;
pub use html_segments::{
    collect_component_attr_expr_spans, html_segments_for_component, html_segments_for_element,
};
pub use output::{
    ExpressionBrace, Language, Output, Segment, convert_braces_to_utf16, segments_source_to_utf16,
    validate_python_segments,
};
pub use python::PythonGenerator;

use crate::ast::Ast;
use crate::plugins::Context;

/// Generator options
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub function_name: Option<String>,
    pub include_ranges: bool,
}

/// Generation result
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub code: String,
    pub segments: Vec<Segment>,
    pub expression_braces: Vec<ExpressionBrace>,
}

/// Generator trait - converts AST to code
pub trait Generator {
    fn generate(&self, ast: &Ast, ctx: &Context, options: &CompileOptions) -> CompileResult;
}
