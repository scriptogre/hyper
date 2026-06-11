pub mod tokenizer;
mod tree_builder;

pub use tokenizer::{Position, Span, Token, tokenize};
use tree_builder::TreeBuilder;

use crate::ast::Ast;
use crate::error::ParseResult;
use std::sync::Arc;

/// Parser trait - converts source code to AST
pub trait Parser {
    fn parse(&self, source: &str) -> ParseResult<Ast>;
}

/// Hyper template parser
pub struct HyperParser {
    // Configuration only, no state
}

impl HyperParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for HyperParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for HyperParser {
    fn parse(&self, source: &str) -> ParseResult<Ast> {
        // Tokenize
        let tokens = tokenize(source);

        // Build AST
        let source_arc: Arc<str> = Arc::from(source);
        let mut builder = TreeBuilder::new(tokens, source_arc.clone());
        let nodes = builder.build()?;

        Ok(Ast::new(nodes, source_arc))
    }
}
