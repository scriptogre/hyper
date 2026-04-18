use crate::helpers::compile;
use hyper_transpiler::generate::RangeType;
use libtest_mimic::Failed;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
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
