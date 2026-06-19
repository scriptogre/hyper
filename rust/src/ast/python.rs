//! Output Python AST. Ruff-faithful (`ruff_python_ast` names/shapes); end goal
//! is swapping to ruff's crates. Synthetic nodes only for now (no source
//! ranges); range-carrying variants land when the printer needs source maps.

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
