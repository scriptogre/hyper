use crate::helpers::compile;
use libtest_mimic::Failed;
use std::fs;
use std::path::PathBuf;

/// Validate that every expression brace position points to an actual { or } in source.
pub fn run(path: &PathBuf) -> Result<(), Failed> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let result = compile(path)?;

    let source_utf16: Vec<u16> = source.encode_utf16().collect();

    for (i, brace) in result.expression_braces.iter().enumerate() {
        // Brace positions are UTF-16 offsets. For ASCII sources they equal byte offsets,
        // but we need to handle the general case.
        if brace.open >= source_utf16.len() {
            return Err(format!(
                "Brace {} open position ({}) >= source UTF-16 length ({})",
                i,
                brace.open,
                source_utf16.len()
            )
            .into());
        }
        if brace.close >= source_utf16.len() {
            return Err(format!(
                "Brace {} close position ({}) >= source UTF-16 length ({})",
                i,
                brace.close,
                source_utf16.len()
            )
            .into());
        }

        // Convert UTF-16 offset to byte offset for character checking
        let open_char = utf16_offset_to_char(&source, brace.open);
        let close_char = utf16_offset_to_char(&source, brace.close);

        if open_char != '{' {
            return Err(format!(
                "Brace {} open at UTF-16 offset {} should be '{{', got '{}' (source context: {:?})",
                i,
                brace.open,
                open_char,
                context_around(&source, brace.open)
            )
            .into());
        }
        if close_char != '}' {
            return Err(format!(
                "Brace {} close at UTF-16 offset {} should be '}}', got '{}' (source context: {:?})",
                i, brace.close, close_char,
                context_around(&source, brace.close)
            ).into());
        }

        if brace.open >= brace.close {
            return Err(format!(
                "Brace {} open ({}) >= close ({})",
                i, brace.open, brace.close
            )
            .into());
        }
    }

    Ok(())
}

/// Convert a UTF-16 offset to the character at that position.
fn utf16_offset_to_char(s: &str, utf16_offset: usize) -> char {
    let mut current_utf16 = 0;
    for ch in s.chars() {
        if current_utf16 == utf16_offset {
            return ch;
        }
        current_utf16 += ch.len_utf16();
    }
    '\0'
}

/// Get a short context string around a UTF-16 offset for error messages.
fn context_around(s: &str, utf16_offset: usize) -> String {
    let start = utf16_offset.saturating_sub(5);
    let end = (utf16_offset + 6).min(s.encode_utf16().count());
    let units: Vec<u16> = s.encode_utf16().collect();
    String::from_utf16_lossy(&units[start..end])
}
