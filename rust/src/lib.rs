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
//! let result = compile(source, &CompileOptions::default());
//! println!("{}", result.python.unwrap().code);
//! ```

pub mod ast;
pub mod diagnostic;
pub mod error;
pub mod generate;
pub mod html;
pub mod lower;
pub mod parse;
pub mod plugins;

#[cfg(feature = "python-extension")]
mod python_module;

use generate::Generator;
use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::path::Path;

/// Python output and diagnostics from one compilation.
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub python: Option<PythonOutput>,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompilationResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Some(DiagnosticSeverity::ERROR))
    }

    pub fn into_python(self) -> Result<PythonOutput, Vec<Diagnostic>> {
        match self.python {
            Some(python) if !self.has_errors() => Ok(python),
            _ => Err(self.diagnostics),
        }
    }

    pub fn render_diagnostics(&self, source: &str, filename: &str, color: bool) -> String {
        self.diagnostics
            .iter()
            .map(|diagnostic| diagnostic::render_diagnostic(diagnostic, source, filename, color))
            .collect()
    }
}

/// Compile a `.hyper` source string.
pub fn compile(source: &str, options: &CompileOptions) -> CompilationResult {
    match compile_python(source, options) {
        Ok(python) => CompilationResult {
            python: Some(python),
            diagnostics: Vec::new(),
        },
        Err(error) => CompilationResult {
            python: None,
            diagnostics: vec![error.diagnostic(source)],
        },
    }
}

fn compile_python(source: &str, options: &CompileOptions) -> Result<PythonOutput, CompileError> {
    let parsed = parse::HyperParser::new().parse_file(source)?;
    let expression_braces = parsed.expression_braces;
    let mut ast = lower::lower(parsed.nodes, source, parsed.has_separator);

    plugins::run(&mut ast, options.function_name.as_deref())?;

    let mut result = generate::PythonGenerator::new().generate(&ast, options);

    if options.include_ranges {
        generate::validate_python_segments(source, &result.code, &mut result.segments);
        // Convert source offsets from byte to UTF-16 last; validation expects byte offsets.
        generate::segments_source_to_utf16(source, &mut result.segments);
        result.expression_braces = generate::convert_braces_to_utf16(source, &expression_braces);
    }

    Ok(result)
}

/// Compile a `.hyper` source string to Python code, deriving the component name
/// from the filename when one is provided.
pub fn compile_to_python(source: &str, filename: Option<&str>) -> Result<String, Vec<Diagnostic>> {
    compile_python_file(source, filename)
        .into_python()
        .map(|python| python.code)
}

// Import hooks also need the mode from the same parse and compile pass.
fn compile_python_file(source: &str, filename: Option<&str>) -> CompilationResult {
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

pub use ast::{Ast, Node, Position, TextRange};
pub use error::{CompileError, ParseError, ParseResult};
pub use generate::{CompileOptions, PythonOutput};
pub use parse::Parser;
pub use plugins::{Flow, Plugin, walk};
