mod html_segments;
mod output;
mod print;
mod python;

pub use html_segments::{
    collect_component_attr_expr_spans, html_segments_for_component, html_segments_for_element,
};
pub use output::{
    ExpressionBrace, Language, Output, Segment, convert_braces_to_utf16, segments_source_to_utf16,
    validate_python_segments,
};
pub use python::PythonGenerator;

use crate::ast::Ast;

/// Generator options
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    pub function_name: Option<String>,
    pub include_ranges: bool,
}

/// Generated Python and its source mapping.
#[derive(Debug, Clone)]
pub struct PythonOutput {
    pub code: String,
    pub component_name: Option<String>,
    pub segments: Vec<Segment>,
    pub expression_braces: Vec<ExpressionBrace>,
}

/// Generator trait - converts AST to code
pub trait Generator {
    fn generate(&self, ast: &Ast, options: &CompileOptions) -> PythonOutput;
}

pub(crate) use crate::parse::tokenizer::template_interpolations;
