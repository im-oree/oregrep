use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::engine::encoding::decode_bytes;

/// Result of a patch application.
#[derive(Debug)]
pub struct PatchResult {
    pub matches_found: usize,
    pub replacements_made: usize,
}

/// Detect the newline style used in the file content.
/// Returns "\r\n" if any CRLF found, else "\n".
fn detect_newline(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Detect if the raw bytes start with a UTF-8 BOM.
fn has_bom(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xEF, 0xBB, 0xBF])
}

/// Read a file preserving encoding info for later re-write.
/// Returns (decoded_content, had_bom, newline_style).
pub fn read_for_patch(file: &Path) -> Result<(String, bool, &'static str)> {
    let bytes = fs::read(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let bom = has_bom(&bytes);
    let content = decode_bytes(&bytes);
    let nl = detect_newline(&content);
    Ok((content, bom, nl))
}

/// Write content back atomically, preserving BOM if it was present.
/// Uses temp file + rename for atomicity.
pub fn write_atomic(file: &Path, content: &str, add_bom: bool) -> Result<()> {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    let fname = file
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
        .to_string_lossy();
    let tmp_name = format!(".{}.oretmp", fname);
    let tmp_path = parent.join(tmp_name);

    let mut bytes: Vec<u8> = Vec::with_capacity(content.len() + 3);
    if add_bom {
        bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    bytes.extend_from_slice(content.as_bytes());

    fs::write(&tmp_path, &bytes)
        .with_context(|| format!("Failed to write temp file: {}", tmp_path.display()))?;

    // Atomic rename
    fs::rename(&tmp_path, file)
        .with_context(|| format!("Failed to rename {} to {}", tmp_path.display(), file.display()))?;

    Ok(())
}

/// Apply a single find/replace to content.
/// Mode determines which occurrences to replace.
#[derive(Debug, Clone, Copy)]
pub enum PatchMode {
    /// Replace exactly one occurrence (fail if 0 or >1)
    Once,
    /// Replace all occurrences
    All,
    /// Replace only the Nth occurrence (1-indexed)
    Nth(usize),
    /// Replace only the first
    First,
    /// Replace only the last
    Last,
}

pub fn apply_patch(content: &str, find: &str, replace: &str, mode: PatchMode) -> Result<(String, PatchResult)> {
    if find.is_empty() {
        anyhow::bail!("Find pattern cannot be empty");
    }

    // Count occurrences
    let matches: Vec<usize> = content
        .match_indices(find)
        .map(|(idx, _)| idx)
        .collect();
    let matches_found = matches.len();

    let mut new_content = String::with_capacity(content.len());
    let replacements_made;

    match mode {
        PatchMode::Once => {
            if matches_found == 0 {
                anyhow::bail!("Find pattern not found in content");
            }
            if matches_found > 1 {
                anyhow::bail!(
                    "Find pattern matches {} times, expected exactly 1. Use --all or --nth N to disambiguate.",
                    matches_found
                );
            }
            new_content.push_str(&content.replacen(find, replace, 1));
            replacements_made = 1;
        }
        PatchMode::All => {
            new_content.push_str(&content.replace(find, replace));
            replacements_made = matches_found;
        }
        PatchMode::First => {
            if matches_found == 0 {
                anyhow::bail!("Find pattern not found in content");
            }
            new_content.push_str(&content.replacen(find, replace, 1));
            replacements_made = 1;
        }
        PatchMode::Last => {
            if matches_found == 0 {
                anyhow::bail!("Find pattern not found in content");
            }
            let last_idx = *matches.last().unwrap();
            new_content.push_str(&content[..last_idx]);
            new_content.push_str(replace);
            new_content.push_str(&content[last_idx + find.len()..]);
            replacements_made = 1;
        }
        PatchMode::Nth(n) => {
            if n == 0 {
                anyhow::bail!("Nth is 1-indexed, cannot be 0");
            }
            if n > matches_found {
                anyhow::bail!("Requested match #{} but only {} occurrences exist", n, matches_found);
            }
            let target_idx = matches[n - 1];
            new_content.push_str(&content[..target_idx]);
            new_content.push_str(replace);
            new_content.push_str(&content[target_idx + find.len()..]);
            replacements_made = 1;
        }
    }

    Ok((
        new_content,
        PatchResult {
            matches_found,
            replacements_made,
        },
    ))
}

/// A single patch operation parsed from a .orepatch file.
#[derive(Debug)]
pub struct PatchOp {
    pub file: String,
    pub find: String,
    pub replace: String,
}

/// Parse an .orepatch file into individual operations.
/// Format:
///   file: path
///   ---
///   find:
///   <lines>
///   ---
///   replace:
///   <lines>
///   ===
pub fn parse_patch_file(content: &str) -> Result<Vec<PatchOp>> {
    let mut ops = Vec::new();
    // Split by === on its own line
    let blocks: Vec<&str> = content.split("\n===\n").collect();

    for (i, block) in blocks.iter().enumerate() {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        let op = parse_single_op(trimmed)
            .with_context(|| format!("Failed to parse patch block #{}", i + 1))?;
        ops.push(op);
    }

    Ok(ops)
}

fn parse_single_op(block: &str) -> Result<PatchOp> {
    // Expect: file: ...\n---\nfind:\n<...>\n---\nreplace:\n<...>
    let parts: Vec<&str> = block.split("\n---\n").collect();
    if parts.len() != 3 {
        anyhow::bail!("Patch block must have exactly 3 sections separated by ---");
    }

    // Section 1: file: <path>
    let file_line = parts[0].trim();
    let file = file_line
        .strip_prefix("file:")
        .ok_or_else(|| anyhow::anyhow!("First section must start with 'file:'"))?
        .trim()
        .to_string();

    // Section 2: find:\n<content>
    let find_section = parts[1];
    let find = find_section
        .strip_prefix("find:\n")
        .or_else(|| find_section.strip_prefix("find:"))
        .ok_or_else(|| anyhow::anyhow!("Second section must start with 'find:'"))?
        .to_string();

    // Section 3: replace:\n<content>
    let replace_section = parts[2];
    let replace = replace_section
        .strip_prefix("replace:\n")
        .or_else(|| replace_section.strip_prefix("replace:"))
        .ok_or_else(|| anyhow::anyhow!("Third section must start with 'replace:'"))?
        .to_string();

    Ok(PatchOp {
        file,
        find,
        replace,
    })
}
