#![allow(dead_code)]

use hyper_transpiler::{GenerateOptions, GenerateResult, Pipeline};
use hyper_transpiler::generate::{Injection, Range, RangeType};

/// Compile source with default options (no ranges, default function name).
pub fn compile(source: &str) -> String {
    let mut pipeline = Pipeline::standard();
    pipeline.compile(source, &GenerateOptions::default()).unwrap().code
}

/// Compile source with ranges enabled, returning the full result.
pub fn compile_with_ranges(source: &str, name: &str) -> GenerateResult {
    let mut pipeline = Pipeline::standard();
    pipeline.compile(source, &GenerateOptions {
        function_name: Some(name.to_string()),
        include_ranges: true,
    }).unwrap()
}

pub fn python_ranges(result: &GenerateResult) -> Vec<&Range> {
    result.ranges.iter().filter(|r| r.range_type == RangeType::Python).collect()
}

pub fn python_injections(result: &GenerateResult) -> Vec<&Injection> {
    result.injections.iter().filter(|i| i.injection_type == "python").collect()
}

pub fn html_ranges(result: &GenerateResult) -> Vec<&Range> {
    result.ranges.iter().filter(|r| r.range_type == RangeType::Html).collect()
}

pub fn html_injections(result: &GenerateResult) -> Vec<&Injection> {
    result.injections.iter().filter(|i| i.injection_type == "html").collect()
}
