use crate::helpers::compile;
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    for range in &result.ranges {
        if range.source_start >= range.source_end {
            continue;
        }
        let text = source
            .get(range.source_start..range.source_end)
            .ok_or_else(|| {
                format!(
                    "Range [{}, {}] out of bounds for source len {}",
                    range.source_start,
                    range.source_end,
                    source.len()
                )
            })?;

        // Check: range should not start mid-identifier
        if range.source_start > 0 {
            let prev_char = source.as_bytes()[range.source_start - 1] as char;
            let first_char = text.chars().next().unwrap_or(' ');
            if prev_char.is_alphanumeric() && first_char.is_alphanumeric() {
                return Err(format!(
                    "Range [{}, {}] starts mid-identifier: prev='{}', text={:?}",
                    range.source_start, range.source_end, prev_char, text
                )
                .into());
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
                )
                .into());
            }
        }
    }
    Ok(())
}
