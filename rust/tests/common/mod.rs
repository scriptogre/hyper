#![allow(dead_code)]

use hyper::CompileOptions;
use hyper::CompileResult;
use hyper::generate::{Language, Segment};

/// Compile source with default options (no ranges, default function name).
pub fn compile(source: &str) -> String {
    hyper::compile(source, &CompileOptions::default())
        .unwrap()
        .code
}

/// Compile source with ranges enabled, returning the full result.
pub fn compile_with_ranges(source: &str, name: &str) -> CompileResult {
    hyper::compile(
        source,
        &CompileOptions {
            function_name: Some(name.to_string()),
            include_ranges: true,
        },
    )
    .unwrap()
}

pub fn python_segments(result: &CompileResult) -> Vec<&Segment> {
    result
        .segments
        .iter()
        .filter(|s| s.language == Language::Python)
        .collect()
}

pub fn html_segments(result: &CompileResult) -> Vec<&Segment> {
    result
        .segments
        .iter()
        .filter(|s| s.language == Language::Html)
        .collect()
}
