use crate::helpers::{compile, utf16_len};
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

pub fn run(path: &PathBuf) -> Result<(), Failed> {
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
