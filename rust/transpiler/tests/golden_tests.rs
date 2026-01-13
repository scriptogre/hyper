use hyper_transpiler::{GenerateOptions, Pipeline};
use std::fs;
use std::path::Path;

/// Derive a snapshot name from the file path
/// e.g., "tests/basic/hello.hyper" -> "basic@hello"
fn snapshot_name(path: &Path) -> String {
    let parent = path.parent().and_then(|p| p.file_name()).unwrap_or_default();
    let stem = path.file_stem().unwrap_or_default();
    format!("{}@{}", parent.to_string_lossy(), stem.to_string_lossy())
}

#[test]
fn test_transpile_output() {
    // glob! base path is relative to the test file's directory (tests/)
    insta::glob!("*/*.hyper", |path| {
        // Skip error test files (they're expected to fail)
        if path.to_string_lossy().contains("/errors/") {
            return;
        }

        let source = fs::read_to_string(path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();

        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: false,
        };

        let mut pipeline = Pipeline::standard();
        let result = pipeline.compile(&source, &options);

        match result {
            Ok(r) => {
                insta::with_settings!({
                    snapshot_path => "../snapshots",
                    prepend_module_to_snapshot => false,
                    snapshot_suffix => "output",
                }, {
                    insta::assert_snapshot!(snapshot_name(path), r.code.trim());
                });
            }
            Err(e) => {
                panic!("Expected successful compilation for {:?}, got error: {}", path, e);
            }
        }
    });
}

#[test]
fn test_transpile_injections() {
    insta::glob!("*/*.hyper", |path| {
        // Skip error test files
        if path.to_string_lossy().contains("/errors/") {
            return;
        }

        let source = fs::read_to_string(path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();

        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: true,
        };

        let mut pipeline = Pipeline::standard();
        if let Ok(result) = pipeline.compile(&source, &options) {
            insta::with_settings!({
                snapshot_path => "../snapshots",
                prepend_module_to_snapshot => false,
                snapshot_suffix => "injections",
            }, {
                insta::assert_yaml_snapshot!(snapshot_name(path), serde_json::json!({
                    "ranges": result.ranges,
                    "injections": result.injections,
                }));
            });
        }
    });
}

#[test]
fn test_transpile_errors() {
    insta::glob!("errors/*.hyper", |path| {
        let source = fs::read_to_string(path).unwrap();
        let name = path.file_stem().unwrap().to_str().unwrap();

        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: false,
        };

        let mut pipeline = Pipeline::standard();
        let result = pipeline.compile(&source, &options);

        match result {
            Ok(_) => {
                // If it compiles, that's also valid - just snapshot the output
                // Some "error" tests might actually be edge cases that should compile
                let result = pipeline.compile(&source, &options).unwrap();
                insta::with_settings!({
                    snapshot_path => "../snapshots",
                    prepend_module_to_snapshot => false,
                    snapshot_suffix => "output",
                }, {
                    insta::assert_snapshot!(snapshot_name(path), result.code.trim());
                });
            }
            Err(e) => {
                insta::with_settings!({
                    snapshot_path => "../snapshots",
                    prepend_module_to_snapshot => false,
                    snapshot_suffix => "error",
                }, {
                    insta::assert_snapshot!(snapshot_name(path), format!("{}", e));
                });
            }
        }
    });
}
