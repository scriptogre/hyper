pub mod tokenizer;
mod tree_builder;

pub use tokenizer::{Position, TextRange, Token, tokenize};
use tree_builder::TreeBuilder;

use crate::ast::Node;
use crate::error::ParseResult;
use std::sync::Arc;

/// Parsed syntax plus file markers that do not become render nodes.
pub(crate) struct ParsedFile {
    pub nodes: Vec<Node>,
    pub has_separator: bool,
}

/// Parser trait - converts source code to a flat node stream (lowered later).
pub trait Parser {
    fn parse(&self, source: &str) -> ParseResult<Vec<Node>>;
}

/// Hyper template parser
pub struct HyperParser {
    // Configuration only, no state
}

impl HyperParser {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn parse_file(&self, source: &str) -> ParseResult<ParsedFile> {
        let tokens = tokenize(source)?;
        let source_arc: Arc<str> = Arc::from(source);
        let mut builder = TreeBuilder::new(tokens, source_arc);
        let nodes = builder.build()?;
        Ok(ParsedFile {
            nodes,
            has_separator: builder.has_separator(),
        })
    }
}

impl Default for HyperParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for HyperParser {
    fn parse(&self, source: &str) -> ParseResult<Vec<Node>> {
        self.parse_file(source).map(|file| file.nodes)
    }
}
