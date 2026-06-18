//! Hyper transpiler
//!
//! Pipeline: Parse → Plugins → Generate
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
    let mut ast = parse::HyperParser::new().parse(source)?;

    let ctx = plugins::run(&mut ast)?;

    let mut result = generate::PythonGenerator::new().generate(&ast, &ctx, options);

    if options.include_ranges {
        generate::validate_python_ranges(source, &result.code, &mut result.ranges);
        result.injections = generate::compute_injections(&result.code, source, &result.ranges);
    }

    Ok(result)
}

/// Compile via the new Ruff-AST pipeline (Parse → Lower → Print).
///
/// Transitional: this lowers the hyper AST to a Ruff [`ast::ModModule`] and
/// prints it with Ruff's stock code generator. It is not yet source-map-aware
/// and does not yet match the byte-for-byte output of [`compile`]; it exists so
/// the lowering pass can be exercised end-to-end while the rest of the new
/// pipeline (plugins on the Ruff AST, source-map-aware printer) is built out.
pub fn compile_via_ast(
    source: &str,
    function_name: Option<&str>,
) -> Result<String, CompileError> {
    use ruff_python_codegen::{Generator, Indentation, Mode};
    use ruff_source_file::LineEnding;

    let mut ast = parse::HyperParser::new().parse(source)?;

    // Run the one AST-transform plugin (reserved-keyword renaming) so identifiers
    // like `class` become `class_` before lowering. The remaining plugins (helper
    // detection, async/slot/mutable-default/spread analysis) will be reimplemented
    // as Ruff-AST passes in Phase 3.
    let mut ctx = plugins::Context::new();
    plugins::RenameReservedKeywords.run(&mut ast, &mut ctx)?;

    let mut module = lower::lower(&ast, function_name)?;
    lower::transform::apply_async(&mut module);
    lower::transform::apply_slots(&mut module);
    lower::transform::apply_helper_imports(&mut module);

    let indent = Indentation::default();
    let mut code = String::new();
    for stmt in &module.body {
        let generator = Generator::new(&indent, LineEnding::default()).with_mode(Mode::Default);
        code.push_str(&generator.stmt(stmt));
        code.push('\n');
    }
    Ok(code)
}

pub use ast::{Ast, Node, Position, Span};
pub use error::{CompileError, ParseError, ParseResult};
pub use generate::{CompileOptions, CompileResult};
pub use parse::Parser;
pub use plugins::{Context, Flow, Plugin, walk};
