use crate::helpers::{compile, utf16_len};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    // Segment source positions are UTF-16, compiled positions are UTF-16
    let source_utf16_len = utf16_len(&source);
    let compiled_len = utf16_len(&result.code);

    for (i, seg) in result.segments.iter().enumerate() {
        if seg.source_start > source_utf16_len {
            return Err(format!(
                "Segment {} source_start ({}) > source UTF-16 length ({})",
                i, seg.source_start, source_utf16_len,
            )
            .into());
        }
        if seg.source_end > source_utf16_len {
            return Err(format!(
                "Segment {} source_end ({}) > source UTF-16 length ({})",
                i, seg.source_end, source_utf16_len,
            )
            .into());
        }
        if seg.compiled_start > compiled_len {
            return Err(format!(
                "Segment {} compiled_start ({}) > compiled UTF-16 length ({})",
                i, seg.compiled_start, compiled_len,
            )
            .into());
        }
        if seg.compiled_end > compiled_len {
            return Err(format!(
                "Segment {} compiled_end ({}) > compiled UTF-16 length ({})",
                i, seg.compiled_end, compiled_len,
            )
            .into());
        }
        if seg.source_start > seg.source_end {
            return Err(format!(
                "Segment {} source_start ({}) > source_end ({})",
                i, seg.source_start, seg.source_end,
            )
            .into());
        }
        if seg.compiled_start > seg.compiled_end {
            return Err(format!(
                "Segment {} compiled_start ({}) > compiled_end ({})",
                i, seg.compiled_start, seg.compiled_end,
            )
            .into());
        }
    }

    Ok(())
}
