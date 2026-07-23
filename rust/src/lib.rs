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

#[cfg(feature = "python-extension")]
mod python_module;

use generate::Generator;
use std::path::Path;

/// Compile a `.hyper` source string to Python.
pub fn compile(source: &str, options: &CompileOptions) -> Result<CompileResult, CompileError> {
    let parsed = parse::HyperParser::new().parse_file(source)?;
    let mut ast = lower::lower(parsed.nodes, source, parsed.has_separator);

    plugins::run(&mut ast)?;

    let mut result = generate::PythonGenerator::new().generate(&ast, options);

    if options.include_ranges {
        generate::validate_python_segments(source, &result.code, &mut result.segments);
        // Convert source offsets from byte to UTF-16 last; validation expects byte offsets.
        generate::segments_source_to_utf16(source, &mut result.segments);
    }

    Ok(result)
}

/// Compile a `.hyper` source string to Python code, deriving the component name
/// from the filename when one is provided.
pub fn compile_to_python(source: &str, filename: Option<&str>) -> Result<String, CompileError> {
    compile_python_file(source, filename).map(|result| result.code)
}

// Import hooks also need the mode from the same parse and compile pass.
fn compile_python_file(
    source: &str,
    filename: Option<&str>,
) -> Result<CompileResult, CompileError> {
    let options = CompileOptions {
        function_name: filename.and_then(function_name_from_filename),
        include_ranges: false,
    };
    compile(source, &options)
}

fn function_name_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}

pub use ast::{Ast, FileMode, Node, Position, TextRange};
pub use error::{CompileError, ParseError, ParseResult};
pub use generate::{CompileOptions, CompileResult};
pub use parse::Parser;
pub use plugins::{Flow, Plugin, walk};
