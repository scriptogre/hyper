//! Hyper transpiler
//!
//! Pipeline: Parse → Lower → Plugins → Generate
//!
//! # Example
//!
//! ```
//! use hyper::{compile, CompileOptions};
//!
//! let source = "<div>{name}</div>\n";
//! let result = compile(source, &CompileOptions::default()).unwrap();
//! println!("{}", result.code);
//! ```

pub mod ast;
pub mod error;
pub mod generate;
pub mod html;
pub mod lower;
pub mod parse;
pub mod plugins;

use generate::Generator;

/// Compile a `.hyper` source string to Python.
pub fn compile(source: &str, options: &CompileOptions) -> Result<CompileResult, CompileError> {
    let nodes = parse::HyperParser::new().parse(source)?;
    let mut ast = lower::lower(nodes, source);

    let ctx = plugins::run(&mut ast)?;

    let mut result = generate::PythonGenerator::new().generate(&ast, &ctx, options);

    if options.include_ranges {
        generate::validate_python_ranges(source, &result.code, &mut result.segments);
        result.injections = generate::compute_injections(&result.code, source, &result.segments);
        // Convert source offsets from byte to UTF-16 last, after validation and
        // injection computation (both expect byte offsets).
        generate::segments_source_to_utf16(source, &mut result.segments);
    }

    Ok(result)
}

pub use ast::{Ast, Node, Position, TextRange};
pub use error::{CompileError, ParseError, ParseResult};
pub use generate::{CompileOptions, CompileResult};
pub use parse::Parser;
pub use plugins::{Context, Flow, Plugin, walk};
