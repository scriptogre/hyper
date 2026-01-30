//! Per-file test runner using libtest-mimic
//!
//! Each .hyper file becomes its own test, enabling parallel execution
//! and clear per-file failure output.

use hyper_transpiler::{GenerateOptions, Pipeline};
use libtest_mimic::{Arguments, Trial, Failed};
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args = Arguments::from_args();
    let tests = collect_tests();
    libtest_mimic::run(&args, tests).exit();
}

fn collect_tests() -> Vec<Trial> {
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut tests = Vec::new();

    for entry in walkdir::WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
    {
        let path = entry.path().to_path_buf();
        let is_error_test = path.to_string_lossy().contains("/errors/");

        // Create test name from path: "basic/hello" or "errors/unclosed_if"
        let rel_path = path.strip_prefix(&test_dir).unwrap();
        let test_name = rel_path
            .with_extension("")
            .to_string_lossy()
            .replace('/', "::");

        if is_error_test {
            // Error test
            let p = path.clone();
            tests.push(Trial::test(format!("error::{}", test_name), move || {
                run_error_test(&p)
            }));
        } else {
            // Output test
            let p1 = path.clone();
            tests.push(Trial::test(format!("output::{}", test_name), move || {
                run_output_test(&p1)
            }));

            // Injection test (only if .expected.json exists)
            let expected_json = path.with_extension("expected.json");
            if expected_json.exists() {
                let p2 = path.clone();
                tests.push(Trial::test(format!("inject::{}", test_name), move || {
                    run_injection_test(&p2)
                }));
            }
        }
    }

    tests
}

fn run_output_test(path: &PathBuf) -> Result<(), Failed> {
    let expected_py = path.with_extension("expected.py");
    if !expected_py.exists() {
        return Err(format!("Missing: {}", expected_py.display()).into());
    }

    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let expected = fs::read_to_string(&expected_py).map_err(|e| e.to_string())?;
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");

    let mut pipeline = Pipeline::standard();
    let options = GenerateOptions {
        function_name: Some(name.to_string()),
        include_ranges: false,
    };

    match pipeline.compile(&source, &options) {
        Ok(result) => {
            if result.code.trim() != expected.trim() {
                Err(format!(
                    "Output mismatch\n--- expected ---\n{}\n--- actual ---\n{}",
                    expected.trim(),
                    result.code.trim()
                ).into())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("Compile error: {}", e).into()),
    }
}

fn run_injection_test(path: &PathBuf) -> Result<(), Failed> {
    let expected_json = path.with_extension("expected.json");
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let expected_str = fs::read_to_string(&expected_json).map_err(|e| e.to_string())?;
    let expected: serde_json::Value = serde_json::from_str(&expected_str)
        .map_err(|e| e.to_string())?;
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
                Err(format!(
                    "Injection mismatch\n--- expected ---\n{}\n--- actual ---\n{}",
                    serde_json::to_string_pretty(&expected).unwrap(),
                    serde_json::to_string_pretty(&actual).unwrap()
                ).into())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("Compile error: {}", e).into()),
    }
}

fn run_error_test(path: &PathBuf) -> Result<(), Failed> {
    let expected_err = path.with_extension("expected.err");
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Template");
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");

    let mut pipeline = Pipeline::standard();
    let options = GenerateOptions {
        function_name: Some(name.to_string()),
        include_ranges: false,
    };

    match pipeline.compile(&source, &options) {
        Ok(_) => {
            if expected_err.exists() {
                Err("Expected error but compiled successfully".into())
            } else {
                Ok(()) // No .expected.err means it's allowed to compile
            }
        }
        Err(e) => {
            if expected_err.exists() {
                let expected = fs::read_to_string(&expected_err).map_err(|e| e.to_string())?;
                let actual = e.render(&source, filename);

                if actual.trim() != expected.trim() {
                    Err(format!(
                        "Error mismatch\n--- expected ---\n{}\n--- actual ---\n{}",
                        expected.trim(),
                        actual.trim()
                    ).into())
                } else {
                    Ok(())
                }
            } else {
                Err(format!("Missing .expected.err file for error: {}", e).into())
            }
        }
    }
}
