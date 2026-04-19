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
pub mod error;
pub mod generate;
pub mod html;
pub mod parser;
pub mod transform;

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

        // Validate: at most one blessed spread name per template
        let mut unique_spread_names: Vec<&str> = Vec::new();
        for (name, _) in &metadata.implicit_spreads {
            if !unique_spread_names.contains(&name.as_str()) {
                unique_spread_names.push(name);
            }
        }
        if unique_spread_names.len() > 1 {
            let names_list = unique_spread_names
                .iter()
                .map(|n| format!("{{**{n}}}"))
                .collect::<Vec<_>>()
                .join(" and ");
            return Err(CompileError::Generate(format!(
                "Cannot use {names_list} in the same template — only one spread parameter is allowed per component"
            )));
        }

        // Generate
        let mut result = self.generator.generate(&ast, metadata, options);

        // Validate injection ranges: drop any Python range where source text ≠ compiled text.
        // This prevents malformed virtual Python files in IDE injection (e.g. when the
        // transpiler renames `class` → `class_`, the source fragment would be a keyword).
        if options.include_ranges {
            generate::validate_python_ranges(source, &result.code, &mut result.ranges);
            result.injections = generate::compute_injections(&result.code, source, &result.ranges);
        }

        Ok(result)
    }
}

// Re-export commonly used types
pub use ast::{Ast, Node, Position, Span};
pub use error::{CompileError, ParseError, ParseResult};
pub use generate::{GenerateOptions, GenerateResult};
pub use parser::Parser;
pub use transform::{Transformer, Visitor};
