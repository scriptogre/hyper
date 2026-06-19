use crate::helpers::compile;
use hyper::generate::Language;
use libtest_mimic::Failed;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let result = compile(path)?;

    for language in [Language::Python, Language::Html] {
        let type_name = match language {
            Language::Python => "Python",
            Language::Html => "HTML",
        };

        let mut typed: Vec<_> = result
            .segments
            .iter()
            .filter(|s| s.language == language)
            .collect();
        typed.sort_by_key(|s| (s.source_start, s.source_end));

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
