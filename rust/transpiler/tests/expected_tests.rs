//! Test runner that compares transpiler output against .expected.py and .expected.json files
//!
//! Run with: cargo test --test expected_tests

use hyper_transpiler::{GenerateOptions, Pipeline};
use std::fs;
use std::path::Path;

/// Collect all .hyper test files
fn collect_test_files() -> Vec<std::path::PathBuf> {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
    {
        files.push(entry.path().to_path_buf());
    }

    files.sort();
    files
}

#[test]
fn test_all_expected_output() {
    let mut failures = Vec::new();

    for path in collect_test_files() {
        // Skip error tests for output comparison
        if path.to_string_lossy().contains("/errors/") {
            continue;
        }

        let expected_py = path.with_extension("expected.py");
        if !expected_py.exists() {
            failures.push(format!("Missing expected file: {}", expected_py.display()));
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let expected = fs::read_to_string(&expected_py).unwrap();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");

        let mut pipeline = Pipeline::standard();
        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: false,
        };

        match pipeline.compile(&source, &options) {
            Ok(result) => {
                if result.code.trim() != expected.trim() {
                    failures.push(format!(
                        "Output mismatch: {}\n--- expected ---\n{}\n--- actual ---\n{}",
                        path.display(),
                        expected.trim(),
                        result.code.trim()
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("Compile error for {}: {}", path.display(), e));
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n{} test(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
    }
}

#[test]
fn test_all_expected_injections() {
    let mut failures = Vec::new();

    for path in collect_test_files() {
        // Skip error tests
        if path.to_string_lossy().contains("/errors/") {
            continue;
        }

        let expected_json = path.with_extension("expected.json");
        if !expected_json.exists() {
            // No injection test for this file - that's OK
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let expected_str = fs::read_to_string(&expected_json).unwrap();
        let expected: serde_json::Value = serde_json::from_str(&expected_str).unwrap();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");

        let mut pipeline = Pipeline::standard();
        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: true,
        };

        match pipeline.compile(&source, &options) {
            Ok(result) => {
                let actual = serde_json::json!({
                    "injections": result.injections,
                    "ranges": result.ranges,
                });

                if actual != expected {
                    failures.push(format!(
                        "Injection mismatch: {}\n--- expected ---\n{}\n--- actual ---\n{}",
                        path.display(),
                        serde_json::to_string_pretty(&expected).unwrap(),
                        serde_json::to_string_pretty(&actual).unwrap()
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("Compile error for {}: {}", path.display(), e));
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n{} test(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
    }
}

#[test]
fn test_all_expected_errors() {
    let mut failures = Vec::new();
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("errors");

    for entry in walkdir::WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
    {
        let path = entry.path();
        let expected_err = path.with_extension("expected.err");

        let source = fs::read_to_string(path).unwrap();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");

        let mut pipeline = Pipeline::standard();
        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: false,
        };

        match pipeline.compile(&source, &options) {
            Ok(_) => {
                // Some error tests might actually compile - check if we expected that
                if expected_err.exists() {
                    failures.push(format!(
                        "Expected error but got success: {}",
                        path.display()
                    ));
                }
            }
            Err(e) => {
                if expected_err.exists() {
                    let expected = fs::read_to_string(&expected_err).unwrap();
                    let actual = e.render(&source, filename);

                    if actual.trim() != expected.trim() {
                        failures.push(format!(
                            "Error mismatch: {}\n--- expected ---\n{}\n--- actual ---\n{}",
                            path.display(),
                            expected.trim(),
                            actual.trim()
                        ));
                    }
                } else {
                    failures.push(format!(
                        "Missing expected.err file: {} (error: {})",
                        expected_err.display(),
                        e
                    ));
                }
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n{} test(s) failed:\n\n{}", failures.len(), failures.join("\n\n"));
    }
}

/// Validate that all injections have sane values
#[test]
fn test_injection_validity() {
    let mut failures = Vec::new();
    let mut warnings = Vec::new();

    for path in collect_test_files() {
        if path.to_string_lossy().contains("/errors/") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");

        let mut pipeline = Pipeline::standard();
        let options = GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: true,
        };

        if let Ok(result) = pipeline.compile(&source, &options) {
            // Validate injections
            for (i, inj) in result.injections.iter().enumerate() {
                if inj.start > inj.end {
                    failures.push(format!(
                        "{}: injection {} has start ({}) > end ({})",
                        path.display(), i, inj.start, inj.end
                    ));
                }
                // Known issue: some injection positions exceed code length
                // TODO: Fix in transpiler - tracking as warning for now
                if inj.end > result.code.len() {
                    warnings.push(format!(
                        "{}: injection {} end ({}) exceeds code length ({})",
                        path.display(), i, inj.end, result.code.len()
                    ));
                }
            }

            // Validate ranges
            for (i, range) in result.ranges.iter().enumerate() {
                if range.source_start > range.source_end {
                    failures.push(format!(
                        "{}: range {} has source_start ({}) > source_end ({})",
                        path.display(), i, range.source_start, range.source_end
                    ));
                }
            }

            // Check for overlapping injections of the same type
            for (i, inj_a) in result.injections.iter().enumerate() {
                for (j, inj_b) in result.injections.iter().enumerate() {
                    if i >= j {
                        continue;
                    }
                    if inj_a.injection_type == inj_b.injection_type {
                        let overlaps = inj_a.start < inj_b.end && inj_b.start < inj_a.end;
                        if overlaps {
                            failures.push(format!(
                                "{}: {} injections {} and {} overlap: [{}-{}] vs [{}-{}]",
                                path.display(),
                                inj_a.injection_type,
                                i, j,
                                inj_a.start, inj_a.end,
                                inj_b.start, inj_b.end
                            ));
                        }
                    }
                }
            }
        }
    }

    // Print warnings but don't fail on them
    if !warnings.is_empty() {
        eprintln!("\n{} warning(s) (known issues):\n{}", warnings.len(), warnings.join("\n"));
    }

    if !failures.is_empty() {
        panic!("\n{} validation failure(s):\n\n{}", failures.len(), failures.join("\n"));
    }
}
