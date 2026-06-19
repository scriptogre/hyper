use crate::helpers::compile;
use hyper::generate::Language;
use libtest_mimic::Failed;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let result = compile(path)?;

    // Filter to Python ranges that need injection, sorted by compiled position.
    // Compiled monotonicity is what matters for correct virtual Python reconstruction —
    // source order may differ (e.g. docstrings appear before parameters in source
    // but after them in compiled output).
    let mut python_segments: Vec<_> = result
        .segments
        .iter()
        .filter(|s| s.language == Language::Python && s.needs_injection)
        .collect();
    python_segments.sort_by_key(|s| s.compiled_start);

    for window in python_segments.windows(2) {
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
