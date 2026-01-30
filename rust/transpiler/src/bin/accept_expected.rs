//! Binary to generate/update .expected.py and .expected.json files
//!
//! Usage:
//!   cargo run --bin accept_expected            # Update all
//!   cargo run --bin accept_expected -- basic   # Update only tests matching "basic"

use hyper_transpiler::{GenerateOptions, Pipeline};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let filter: Option<String> = std::env::args().nth(1);
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut updated = 0;
    let mut skipped = 0;

    for entry in WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
    {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        // Apply filter if provided
        if let Some(ref f) = filter {
            if !path_str.contains(f) {
                skipped += 1;
                continue;
            }
        }

        process_file(path);
        updated += 1;
    }

    println!("Updated {} files, skipped {}", updated, skipped);
}

fn process_file(path: &Path) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {:?}: {}", path, e);
            return;
        }
    };

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Template");

    let is_error_test = path.to_string_lossy().contains("/errors/");

    let mut pipeline = Pipeline::standard();

    // First compile without ranges for output
    let options = GenerateOptions {
        function_name: Some(name.to_string()),
        include_ranges: false,
    };

    let result = pipeline.compile(&source, &options);

    match result {
        Ok(output) => {
            // Write .expected.py
            let expected_py = path.with_extension("expected.py");
            if let Err(e) = fs::write(&expected_py, &output.code) {
                eprintln!("Failed to write {:?}: {}", expected_py, e);
            } else {
                println!("  wrote {}", expected_py.display());
            }

            // Remove any stale .expected.err if this now compiles
            let expected_err = path.with_extension("expected.err");
            if expected_err.exists() {
                let _ = fs::remove_file(&expected_err);
            }

            // Now compile with ranges for injections
            let options_with_ranges = GenerateOptions {
                function_name: Some(name.to_string()),
                include_ranges: true,
            };
            if let Ok(result_with_ranges) = pipeline.compile(&source, &options_with_ranges) {
                if !result_with_ranges.injections.is_empty()
                    || !result_with_ranges.ranges.is_empty()
                {
                    let expected_json = path.with_extension("expected.json");
                    let json = serde_json::json!({
                        "injections": result_with_ranges.injections,
                        "ranges": result_with_ranges.ranges,
                    });
                    if let Err(e) =
                        fs::write(&expected_json, serde_json::to_string_pretty(&json).unwrap())
                    {
                        eprintln!("Failed to write {:?}: {}", expected_json, e);
                    } else {
                        println!("  wrote {}", expected_json.display());
                    }
                }
            }
        }
        Err(e) => {
            if is_error_test {
                // Write .expected.err
                let expected_err = path.with_extension("expected.err");
                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");
                if let Err(err) = fs::write(&expected_err, e.render(&source, filename)) {
                    eprintln!("Failed to write {:?}: {}", expected_err, err);
                } else {
                    println!("  wrote {}", expected_err.display());
                }
            } else {
                eprintln!(
                    "ERROR: {:?} failed to compile but is not in errors/: {}",
                    path, e
                );
            }
        }
    }
}
