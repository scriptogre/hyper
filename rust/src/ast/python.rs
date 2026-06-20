//! Output Python AST. Ruff-faithful (`ruff_python_ast` names/shapes); end goal
//! is swapping to ruff's crates. `Code` is the unparsed-source seam: holds
//! verbatim Python text plus its `.hyper` source range; shrinks as we adopt
//! ruff's parser.

use crate::ast::TextRange;

#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: String,
}

impl Identifier {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: Identifier,
    pub asname: Option<Identifier>,
}

/// `from <module> import <names>` (level 0 = absolute). `level > 0` for
/// relative imports (`.module`, `..module`).
#[derive(Debug, Clone)]
pub struct StmtImportFrom {
    pub module: Option<Identifier>,
    pub names: Vec<Alias>,
    pub level: u32,
}

/// Verbatim Python source with its `.hyper` range. Synthetic range = no IDE
/// injection. Used for nodes that aren't lowered into structured AST yet
/// (user imports, control-flow conditions, statement lines).
#[derive(Debug, Clone)]
pub struct Code {
    pub source: String,
    pub range: TextRange,
}

/// Output expression. `Code` is the verbatim-source seam; `Call`/`Name` are
/// structured nodes lowering builds (compiler-invented calls like `escape(x)`).
#[derive(Debug, Clone)]
pub enum Expr {
    Call(ExprCall),
    Name(ExprName),
    Code(Code),
}

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub func: Box<Expr>,
    pub arguments: Arguments,
}

#[derive(Debug, Clone)]
pub struct Arguments {
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprName {
    pub id: Identifier,
}
