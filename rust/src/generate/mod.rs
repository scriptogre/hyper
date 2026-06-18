mod brace_collector;
mod injection_analyzer;
mod output;
mod python;

pub use brace_collector::{collect_expression_braces, collect_tag_highlights};
pub use injection_analyzer::{
    InjectionAnalyzer, collect_component_attr_expr_spans, html_ranges_for_component,
    html_ranges_for_element,
};
pub use output::{
    ExpressionBrace, Injection, Mapping, Output, Range, RangeType, TagHighlight, TagHighlightKind,
    compute_injections, convert_braces_to_utf16, convert_tag_highlights_to_utf16,
    validate_python_ranges,
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
    pub mappings: Vec<Mapping>,
    pub ranges: Vec<Range>,
    pub injections: Vec<Injection>,
    pub expression_braces: Vec<ExpressionBrace>,
    pub tag_highlights: Vec<TagHighlight>,
}

/// Generator trait - converts AST to code
pub trait Generator {
    fn generate(&self, ast: &Ast, ctx: &Context, options: &CompileOptions) -> CompileResult;
}
