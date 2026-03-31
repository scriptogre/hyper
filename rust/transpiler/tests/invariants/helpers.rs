use hyper_transpiler::{GenerateOptions, GenerateResult, Pipeline};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

/// Compile a .hyper file with ranges enabled.
pub fn compile(path: &PathBuf) -> Result<GenerateResult, Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Template");

    let mut pipeline = Pipeline::standard();
    let options = GenerateOptions {
        function_name: Some(name.to_string()),
        include_ranges: true,
    };

    pipeline
        .compile(&source, &options)
        .map_err(|e| format!("Compile error: {}", e).into())
}

/// Extract a substring from `s` using UTF-16 offsets.
pub fn substring_utf16(s: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let units: Vec<u16> = s.encode_utf16().collect();
    let end = end.min(units.len());
    let start = start.min(end);
    String::from_utf16_lossy(&units[start..end])
}

/// UTF-16 length of a string.
pub fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}
