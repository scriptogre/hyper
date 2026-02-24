//! Injection highlighting validation tests.
//!
//! Validates injection correctness across ALL test files by checking:
//! - Virtual Python round-trip (prefix + source + suffix == compiled code)
//! - Compiled position monotonicity
//! - No overlapping ranges within the same type
//! - All positions within bounds

use hyper_transpiler::generate::RangeType;
use hyper_transpiler::{GenerateOptions, Pipeline};
use libtest_mimic::{Arguments, Failed, Trial};
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

        // Skip error tests — they don't compile successfully
        if path.to_string_lossy().contains("/errors/") {
            continue;
        }

        let rel_path = path.strip_prefix(&test_dir).unwrap();
        let test_name = rel_path
            .with_extension("")
            .to_string_lossy()
            .replace('/', "::");

        // Test A: Virtual Python round-trip
        let p = path.clone();
        tests.push(Trial::test(
            format!("roundtrip::{}", test_name),
            move || run_roundtrip_test(&p),
        ));

        // Test B: Compiled position monotonicity
        let p = path.clone();
        tests.push(Trial::test(
            format!("monotonic::{}", test_name),
            move || run_monotonicity_test(&p),
        ));

        // Test C: No overlapping ranges within same type
        let p = path.clone();
        tests.push(Trial::test(
            format!("no_overlap::{}", test_name),
            move || run_no_overlap_test(&p),
        ));

        // Test D: Source and compiled positions within bounds
        let p = path.clone();
        tests.push(Trial::test(format!("bounds::{}", test_name), move || {
            run_bounds_test(&p)
        }));

        // Test E: Source ranges extract to meaningful text (not mid-identifier)
        let p = path.clone();
        tests.push(Trial::test(format!("semantic::{}", test_name), move || {
            run_semantic_test(&p)
        }));
    }

    tests
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compile a .hyper file with ranges enabled.
fn compile(path: &PathBuf) -> Result<hyper_transpiler::GenerateResult, Failed> {
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
fn substring_utf16(s: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let units: Vec<u16> = s.encode_utf16().collect();
    let end = end.min(units.len());
    let start = start.min(end);
    String::from_utf16_lossy(&units[start..end])
}

/// UTF-16 length of a string.
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

// ---------------------------------------------------------------------------
// Test A: Virtual Python round-trip
// ---------------------------------------------------------------------------

fn run_roundtrip_test(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    let python_injections: Vec<_> = result
        .injections
        .iter()
        .filter(|inj| inj.injection_type == "python")
        .collect();

    if python_injections.is_empty() {
        // No Python injections — nothing to round-trip (template may be pure HTML).
        return Ok(());
    }

    // Reconstruct the virtual Python file the way JetBrains does:
    //   virtual = prefix_0 + source[start_0..end_0]
    //           + prefix_1 + source[start_1..end_1]
    //           + ...
    //           + prefix_n + source[start_n..end_n] + suffix_n
    //
    // Only the last injection carries a non-empty suffix; all others have "".

    let mut virtual_python = String::new();
    for inj in &python_injections {
        virtual_python.push_str(&inj.prefix);
        virtual_python.push_str(&substring_utf16(&source, inj.start, inj.end));
        virtual_python.push_str(&inj.suffix);
    }

    if virtual_python != result.code {
        // Build a helpful diff-like message
        let vp_lines: Vec<&str> = virtual_python.lines().collect();
        let code_lines: Vec<&str> = result.code.lines().collect();
        let max = vp_lines.len().max(code_lines.len());
        let mut diffs = String::new();
        for i in 0..max {
            let vp = vp_lines.get(i).unwrap_or(&"<missing>");
            let co = code_lines.get(i).unwrap_or(&"<missing>");
            if vp != co {
                diffs.push_str(&format!(
                    "  line {}: virtual={:?}  compiled={:?}\n",
                    i + 1,
                    vp,
                    co
                ));
            }
        }
        return Err(format!(
            "Virtual Python != compiled code\n\
             virtual len={} compiled len={}\n\
             First differing lines:\n{}",
            virtual_python.len(),
            result.code.len(),
            diffs
        )
        .into());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test B: Compiled position monotonicity
// ---------------------------------------------------------------------------

fn run_monotonicity_test(path: &PathBuf) -> Result<(), Failed> {
    let result = compile(path)?;

    // Filter to Python ranges that need injection, sorted by compiled position.
    // Compiled monotonicity is what matters for correct virtual Python reconstruction —
    // source order may differ (e.g. docstrings appear before parameters in source
    // but after them in compiled output).
    let mut python_ranges: Vec<_> = result
        .ranges
        .iter()
        .filter(|r| r.range_type == RangeType::Python && r.needs_injection)
        .collect();
    python_ranges.sort_by_key(|r| r.compiled_start);

    for window in python_ranges.windows(2) {
        let a = window[0];
        let b = window[1];
        if a.compiled_end > b.compiled_start {
            return Err(format!(
                "Compiled positions not monotonic: range ending at compiled={} \
                 overlaps range starting at compiled={}\n\
                 range A: source=[{}..{}] compiled=[{}..{}]\n\
                 range B: source=[{}..{}] compiled=[{}..{}]",
                a.compiled_end,
                b.compiled_start,
                a.source_start,
                a.source_end,
                a.compiled_start,
                a.compiled_end,
                b.source_start,
                b.source_end,
                b.compiled_start,
                b.compiled_end,
            )
            .into());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test C: No overlapping ranges within same type
// ---------------------------------------------------------------------------

fn run_no_overlap_test(path: &PathBuf) -> Result<(), Failed> {
    let result = compile(path)?;

    for range_type in [RangeType::Python, RangeType::Html] {
        let type_name = match range_type {
            RangeType::Python => "Python",
            RangeType::Html => "HTML",
        };

        let mut typed: Vec<_> = result
            .ranges
            .iter()
            .filter(|r| r.range_type == range_type)
            .collect();
        typed.sort_by_key(|r| (r.source_start, r.source_end));

        for window in typed.windows(2) {
            let a = window[0];
            let b = window[1];
            if a.source_end > b.source_start {
                return Err(format!(
                    "{} source ranges overlap:\n\
                     range A: source=[{}..{}]\n\
                     range B: source=[{}..{}]",
                    type_name, a.source_start, a.source_end, b.source_start, b.source_end,
                )
                .into());
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test D: Source and compiled positions within bounds
// ---------------------------------------------------------------------------

fn run_bounds_test(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    // Range source positions are byte offsets, compiled positions are UTF-16
    let source_byte_len = source.len();
    let source_utf16_len = utf16_len(&source);
    let compiled_len = utf16_len(&result.code);

    for (i, range) in result.ranges.iter().enumerate() {
        // Source positions are byte offsets
        if range.source_start > source_byte_len {
            return Err(format!(
                "Range {} source_start ({}) > source byte length ({})",
                i, range.source_start, source_byte_len,
            )
            .into());
        }
        if range.source_end > source_byte_len {
            return Err(format!(
                "Range {} source_end ({}) > source byte length ({})",
                i, range.source_end, source_byte_len,
            )
            .into());
        }
        // Compiled positions are UTF-16
        if range.compiled_start > compiled_len {
            return Err(format!(
                "Range {} compiled_start ({}) > compiled UTF-16 length ({})",
                i, range.compiled_start, compiled_len,
            )
            .into());
        }
        if range.compiled_end > compiled_len {
            return Err(format!(
                "Range {} compiled_end ({}) > compiled UTF-16 length ({})",
                i, range.compiled_end, compiled_len,
            )
            .into());
        }
        if range.source_start > range.source_end {
            return Err(format!(
                "Range {} source_start ({}) > source_end ({})",
                i, range.source_start, range.source_end,
            )
            .into());
        }
        if range.compiled_start > range.compiled_end {
            return Err(format!(
                "Range {} compiled_start ({}) > compiled_end ({})",
                i, range.compiled_start, range.compiled_end,
            )
            .into());
        }
    }

    // Injection positions are UTF-16 (converted from byte offsets in compute_injections)
    for (i, inj) in result.injections.iter().enumerate() {
        if inj.start > source_utf16_len {
            return Err(format!(
                "Injection {} start ({}) > source UTF-16 length ({})",
                i, inj.start, source_utf16_len,
            )
            .into());
        }
        if inj.end > source_utf16_len {
            return Err(format!(
                "Injection {} end ({}) > source UTF-16 length ({})",
                i, inj.end, source_utf16_len,
            )
            .into());
        }
        if inj.start > inj.end {
            return Err(format!(
                "Injection {} start ({}) > end ({})",
                i, inj.start, inj.end,
            )
            .into());
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test E: Semantic validation — source ranges extract to meaningful text
// ---------------------------------------------------------------------------

fn run_semantic_test(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    for range in &result.ranges {
        if range.source_start >= range.source_end {
            continue;
        }
        let text = source.get(range.source_start..range.source_end)
            .ok_or_else(|| format!(
                "Range [{}, {}] out of bounds for source len {}",
                range.source_start, range.source_end, source.len()
            ))?;

        // Check: range should not start mid-identifier
        if range.source_start > 0 {
            let prev_char = source.as_bytes()[range.source_start - 1] as char;
            let first_char = text.chars().next().unwrap_or(' ');
            if prev_char.is_alphanumeric() && first_char.is_alphanumeric() {
                return Err(format!(
                    "Range [{}, {}] starts mid-identifier: prev='{}', text={:?}",
                    range.source_start, range.source_end, prev_char, text
                ).into());
            }
        }

        // Check: range should not end mid-identifier
        if range.source_end < source.len() {
            let last_char = text.chars().last().unwrap_or(' ');
            let next_char = source.as_bytes()[range.source_end] as char;
            if last_char.is_alphanumeric() && next_char.is_alphanumeric() {
                return Err(format!(
                    "Range [{}, {}] ends mid-identifier: text={:?}, next='{}'",
                    range.source_start, range.source_end, text, next_char
                ).into());
            }
        }
    }
    Ok(())
}
