//! Hyper transpiler v2 - Plugin-based pipeline architecture
//!
//! This crate implements a clean three-stage pipeline:
//! 1. Parser: Source → AST
//! 2. Transformer: AST → AST (through plugins)
//! 3. Generator: AST → Python code
//!
//! # Example
//!
//! ```
//! use hyper_transpiler::{Pipeline, GenerateOptions};
//!
//! let source = "<div>{name}</div>\n";
//! let mut pipeline = Pipeline::standard();
//! let result = pipeline.compile(source, &GenerateOptions::default()).unwrap();
//! println!("{}", result.code);
//! ```

pub mod ast;
pub mod parser;
pub mod transform;
pub mod generate;
pub mod error;
pub mod html;

use generate::{Generator, PythonGenerator};
use parser::HyperParser;

/// Complete compilation pipeline
pub struct Pipeline {
    parser: Box<dyn Parser>,
    transformer: Transformer,
    generator: Box<dyn Generator>,
}

impl Pipeline {
    /// Create a new pipeline with the standard configuration
    pub fn standard() -> Self {
        Self {
            parser: Box::new(HyperParser::new()),
            transformer: transform::standard_plugins(),
            generator: Box::new(PythonGenerator::new()),
        }
    }

    /// Create a custom pipeline
    pub fn custom(
        parser: Box<dyn Parser>,
        transformer: Transformer,
        generator: Box<dyn Generator>,
    ) -> Self {
        Self {
            parser,
            transformer,
            generator,
        }
    }

    /// Compile source code to Python
    pub fn compile(
        &mut self,
        source: &str,
        options: &GenerateOptions,
    ) -> Result<GenerateResult, CompileError> {
        // Parse
        let mut ast = self.parser.parse(source)?;

        // Transform
        let metadata = self.transformer.transform(&mut ast);

        // Generate
        Ok(self.generator.generate(&ast, metadata, options))
    }
}

// Re-export commonly used types
pub use ast::{Ast, Node, Position, Span};
pub use error::{CompileError, ParseError};
pub use generate::{GenerateOptions, GenerateResult};
pub use parser::Parser;
pub use transform::{Transformer, Visitor};
