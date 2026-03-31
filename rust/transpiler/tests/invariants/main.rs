//! Invariant validation tests.
//!
//! Property-based checks that run across ALL .hyper test files.
//! Each module validates one structural invariant.

mod helpers;
mod roundtrip;
mod monotonicity;
mod no_overlap;
mod bounds;
mod semantic;
mod completeness;
mod braces;
mod html_completeness;

use libtest_mimic::{Arguments, Trial};
use std::path::Path;

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
            move || roundtrip::run(&p),
        ));

        // Test B: Compiled position monotonicity
        let p = path.clone();
        tests.push(Trial::test(
            format!("monotonic::{}", test_name),
            move || monotonicity::run(&p),
        ));

        // Test C: No overlapping ranges within same type
        let p = path.clone();
        tests.push(Trial::test(
            format!("no_overlap::{}", test_name),
            move || no_overlap::run(&p),
        ));

        // Test D: Source and compiled positions within bounds
        let p = path.clone();
        tests.push(Trial::test(format!("bounds::{}", test_name), move || {
            bounds::run(&p)
        }));

        // Test E: Source ranges extract to meaningful text (not mid-identifier)
        let p = path.clone();
        tests.push(Trial::test(format!("semantic::{}", test_name), move || {
            semantic::run(&p)
        }));

        // Test F: Every Python-bearing token in the body has a corresponding range
        let p = path.clone();
        tests.push(Trial::test(
            format!("completeness::{}", test_name),
            move || completeness::run(&p),
        ));

        // Test G: Expression brace positions point to actual { and } characters
        let p = path.clone();
        tests.push(Trial::test(
            format!("braces::{}", test_name),
            move || braces::run(&p),
        ));

        // Test H: Every HTML element/component/slot tag has HTML injection ranges
        let p = path.clone();
        tests.push(Trial::test(
            format!("html_completeness::{}", test_name),
            move || html_completeness::run(&p),
        ));
    }

    tests
}
