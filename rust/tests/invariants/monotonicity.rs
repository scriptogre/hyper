use crate::helpers::compile;
use hyper_transpiler::generate::RangeType;
use libtest_mimic::Failed;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
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
