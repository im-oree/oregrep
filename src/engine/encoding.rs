use anyhow::Result;
use encoding_rs::{Encoding, UTF_8};
use std::fs;
use std::path::Path;

/// Read a file, auto-detecting encoding, returning decoded UTF-8 string.
/// Handles UTF-8, UTF-16 LE/BE, and falls back to Windows-1252 for legacy content.
pub fn read_file_smart(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(decode_bytes(&bytes))
}

/// Decode raw bytes into a String, detecting encoding.
pub fn decode_bytes(bytes: &[u8]) -> String {
    // Check for BOMs first
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (cow, _, _) = UTF_8.decode(&bytes[3..]);
        return cow.into_owned();
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return cow.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return cow.into_owned();
    }

    // Try UTF-8 first
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }

    // Fall back to chardet detection
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding: &'static Encoding = detector.guess(None, true);
    let (cow, _, _) = encoding.decode(bytes);
    cow.into_owned()
}

/// Check if a file is likely binary (has null bytes in first 8KB).
pub fn is_binary(path: &Path) -> Result<bool> {
    let bytes = fs::read(path)?;
    let sample_size = bytes.len().min(8192);
    Ok(bytes[..sample_size].contains(&0))
}
