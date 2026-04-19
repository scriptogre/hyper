//! Binary to generate/update .expected.py and .expected.json files
//!
//! Usage:
//!   cargo run --bin accept_expected                      # Show what would change
//!   cargo run --bin accept_expected -- --apply           # Write (interactive only)
//!   cargo run --bin accept_expected -- basic --apply     # Write matching "basic"

use hyper_transpiler::{GenerateOptions, Pipeline};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let apply = args.iter().any(|a| a == "--apply");
    let filter: Option<&str> = args.iter().find(|a| *a != "--apply").map(|s| s.as_str());

    if apply {
        print!("{} file(s) will be updated. Type YES to confirm: ", {
            let test_dir_pre = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
            let mut count = 0;
            for entry in WalkDir::new(&test_dir_pre)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
            {
                let path = entry.path();
                if let Some(f) = filter
                    && !path.to_string_lossy().contains(f)
                {
                    continue;
                }
                if process_file(path, false) {
                    count += 1;
                }
            }
            count
        });
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim() != "YES" {
            println!("Aborted.");
            std::process::exit(1);
        }
    }

    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut changed = 0;
    let mut _skipped = 0;

    for entry in WalkDir::new(&test_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "hyper").unwrap_or(false))
    {
        let path = entry.path();
        if let Some(f) = filter
            && !path.to_string_lossy().contains(f)
        {
            _skipped += 1;
            continue;
        }
        if process_file(path, apply) {
            changed += 1;
        }
    }

    if apply {
        println!("Updated {} file(s).", changed);
    } else if changed > 0 {
        println!(
            "\n{} file(s) would change. Run with --apply to write.",
            changed
        );
    } else {
        println!("All expected files are up to date.");
    }
}

fn process_file(path: &Path, write: bool) -> bool {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {:?}: {}", path, e);
            return false;
        }
    };

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Template");
    let is_error_test = path.to_string_lossy().contains("/errors/");
    let mut pipeline = Pipeline::standard();
    let mut has_changes = false;

    let result = pipeline.compile(
        &source,
        &GenerateOptions {
            function_name: Some(name.to_string()),
            include_ranges: false,
        },
    );

    match result {
        Ok(output) => {
            let expected_py = path.with_extension("expected.py");
            has_changes |= write_if_changed(&expected_py, &output.code, write);

            let expected_err = path.with_extension("expected.err");
            if expected_err.exists() {
                if write {
                    let _ = fs::remove_file(&expected_err);
                }
                has_changes = true;
            }

            let result_with_ranges = pipeline.compile(
                &source,
                &GenerateOptions {
                    function_name: Some(name.to_string()),
                    include_ranges: true,
                },
            );
            if let Ok(r) = result_with_ranges
                && (!r.injections.is_empty() || !r.ranges.is_empty())
            {
                let expected_json = path.with_extension("expected.json");
                let json = serde_json::json!({
                    "injections": r.injections,
                    "ranges": r.ranges,
                });
                let content = serde_json::to_string_pretty(&json).unwrap();
                has_changes |= write_if_changed(&expected_json, &content, write);
            }
        }
        Err(e) => {
            if is_error_test {
                let expected_err = path.with_extension("expected.err");
                let filename = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                has_changes |= write_if_changed(&expected_err, &e.render(&source, filename), write);
            } else {
                eprintln!(
                    "ERROR: {:?} failed to compile but is not in errors/: {}",
                    path, e
                );
            }
        }
    }

    has_changes
}

fn write_if_changed(path: &Path, new_content: &str, write: bool) -> bool {
    let existing = fs::read_to_string(path).unwrap_or_default();
    if path.exists() && existing == new_content {
        return false;
    }

    let rel = path
        .to_string_lossy()
        .rsplit("/tests/")
        .next()
        .unwrap_or(&path.to_string_lossy())
        .to_string();

    if write {
        if let Err(e) = fs::write(path, new_content) {
            eprintln!("Failed to write {:?}: {}", path, e);
        } else {
            println!("  wrote {}", rel);
        }
    } else if rel.ends_with(".py") {
        println!("=== {} ===", rel);
        println!("{}", new_content);
    } else {
        println!("  CHANGED: {}", rel);
    }
    true
}
