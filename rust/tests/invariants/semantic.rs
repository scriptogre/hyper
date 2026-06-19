use crate::helpers::{compile, substring_utf16, utf16_len};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;
    let source_units: Vec<u16> = source.encode_utf16().collect();
    let source_utf16_len = source_units.len();

    for seg in &result.segments {
        if seg.source_start >= seg.source_end {
            continue;
        }
        if seg.source_end > source_utf16_len {
            return Err(format!(
                "Segment [{}, {}] out of bounds for source UTF-16 len {}",
                seg.source_start, seg.source_end, source_utf16_len
            )
            .into());
        }
        let text = substring_utf16(&source, seg.source_start, seg.source_end);

        if seg.source_start > 0 {
            let prev = substring_utf16(&source, seg.source_start - 1, seg.source_start);
            let prev_char = prev.chars().next().unwrap_or(' ');
            let first_char = text.chars().next().unwrap_or(' ');
            if prev_char.is_alphanumeric() && first_char.is_alphanumeric() {
                return Err(format!(
                    "Segment [{}, {}] starts mid-identifier: prev='{}', text={:?}",
                    seg.source_start, seg.source_end, prev_char, text
                )
                .into());
            }
        }

        if seg.source_end < utf16_len(&source) {
            let last_char = text.chars().last().unwrap_or(' ');
            let next = substring_utf16(&source, seg.source_end, seg.source_end + 1);
            let next_char = next.chars().next().unwrap_or(' ');
            if last_char.is_alphanumeric() && next_char.is_alphanumeric() {
                return Err(format!(
                    "Segment [{}, {}] ends mid-identifier: text={:?}, next='{}'",
                    seg.source_start, seg.source_end, text, next_char
                )
                .into());
            }
        }
    }
    Ok(())
}
