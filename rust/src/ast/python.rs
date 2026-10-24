//! Output nodes Ruff cannot hold. User source stays verbatim in `Code`, so
//! printing never rewrites it; compiler-generated statements use Ruff nodes.

use crate::ast::TextRange;

/// Verbatim Python source with its `.hyper` range. Synthetic range = no IDE
/// injection.
#[derive(Debug, Clone)]
pub struct Code {
    pub source: String,
    pub range: TextRange,
}

/// Compiler-invented `helper(arg)` or `helper("attr_name", arg)` around user `Code`.
#[derive(Debug, Clone)]
pub struct HelperCall {
    pub helper: &'static str,
    pub attr_name: Option<String>,
    pub arg: Code,
}
