//! Position conversion utilities.
//!
//! This module provides helpers for converting between byte offsets and UTF-16 offsets.
//! IDEs use UTF-16 offsets for language injection, but we track byte offsets internally
//! for simplicity.

/// Convert a byte offset to a UTF-16 offset.
///
/// # Arguments
/// * `source` - The source string
/// * `byte_offset` - The byte offset to convert
///
/// # Returns
/// The UTF-16 offset corresponding to the byte offset.
pub fn byte_to_utf16(source: &str, byte_offset: usize) -> usize {
    let byte_offset = byte_offset.min(source.len());
    source[..byte_offset].encode_utf16().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        let source = "hello world";
        assert_eq!(byte_to_utf16(source, 0), 0);
        assert_eq!(byte_to_utf16(source, 5), 5);
        assert_eq!(byte_to_utf16(source, 11), 11);
    }

    #[test]
    fn test_emoji() {
        let source = "hello 👋 world";
        // 👋 is 4 bytes but 2 UTF-16 code units
        assert_eq!(byte_to_utf16(source, 6), 6);  // before emoji
        assert_eq!(byte_to_utf16(source, 10), 8); // after emoji (6 + 2)
    }

    #[test]
    fn test_multibyte() {
        let source = "café";
        // é is 2 bytes but 1 UTF-16 code unit
        assert_eq!(byte_to_utf16(source, 3), 3);  // before é
        assert_eq!(byte_to_utf16(source, 5), 4);  // after é
    }

    #[test]
    fn test_out_of_bounds() {
        let source = "hello";
        assert_eq!(byte_to_utf16(source, 100), 5); // clamped to length
    }
}
