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

/// Unescape CLI argument escape sequences so that literal \n \r \t
/// passed from a shell or GUI tokenizer become real control characters.
/// Handles: \n → LF, \r\n → CRLF (as \r + \n), \r → CR, \t → TAB, \\ → \
/// Called BEFORE newline normalization so the result can then be
/// re-normalized to the file's actual line ending style.
pub fn unescape_arg(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('n') => { chars.next(); out.push('\n'); }
                Some('r') => {
                    chars.next();
                    // Check for \r\n sequence
                    if chars.peek() == Some(&'\\') {
                        // peek two ahead — consume the \ then check for n
                        let mut tmp = chars.clone();
                        tmp.next(); // consume '\'
                        if tmp.peek() == Some(&'n') {
                            chars.next(); // consume '\'
                            chars.next(); // consume 'n'
                            out.push('\n'); // normalize \r\n → \n (file normalizer handles the rest)
                        } else {
                            out.push('\r');
                        }
                    } else {
                        out.push('\r');
                    }
                }
                Some('t') => { chars.next(); out.push('\t'); }
                Some('\\') => { chars.next(); out.push('\\'); }
                _ => { out.push('\\'); }
            }
        } else {
            out.push(c);
        }
    }
    out
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
    Once,
    All,
    Nth(usize),
    First,
    Last,
}

pub fn apply_patch(content: &str, find: &str, replace: &str, mode: PatchMode) -> Result<(String, PatchResult)> {
    if find.is_empty() {
        anyhow::bail!("Find pattern cannot be empty");
    }

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
///
/// Handles BOM, CRLF line endings, and UTF-16 encoded input.
pub fn parse_patch_file(content: &str) -> Result<Vec<PatchOp>> {
    // Strip UTF-8 BOM if present as a character
    let content = content.trim_start_matches('\u{FEFF}');

    // Normalize line endings: CRLF -> LF
    let normalized = content.replace("\r\n", "\n");

    // Try explicit === separator first
    let mut blocks: Vec<String> = normalized
        .split("\n===\n")
        .map(|s| s.to_string())
        .collect();

    // If only one block and no === separator was found, try splitting by
    // "double blank line + file:" pattern to support the human-friendly
    // format that omits === between ops.
    if blocks.len() == 1 && !normalized.contains("\n===\n") {
        blocks = split_by_file_marker(&normalized);
    }

    let mut ops = Vec::new();
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

/// Split on lines starting with "file:" — treats each as the start of a new op.
/// Used when the user omits === separators between ops.
fn split_by_file_marker(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut blocks: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("file:") && !current.is_empty() {
            // Start of new block — flush current
            blocks.push(current.join("\n"));
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

fn parse_single_op(block: &str) -> Result<PatchOp> {
    // Try explicit --- separator first
    if block.contains("\n---\n") {
        let parts: Vec<&str> = block.split("\n---\n").collect();
        if parts.len() == 3 {
            return parse_three_sections(parts[0], parts[1], parts[2]);
        }
    }

    // Fallback: parse by "file:", "find:", "replace:" markers on their own lines
    parse_by_markers(block)
}

fn parse_three_sections(file_sec: &str, find_sec: &str, replace_sec: &str) -> Result<PatchOp> {
    let file_line = file_sec.trim();
    let file = file_line
        .strip_prefix("file:")
        .ok_or_else(|| anyhow::anyhow!("First section must start with 'file:', got: {:?}", file_line))?
        .trim()
        .to_string();

    let find = find_sec
        .strip_prefix("find:\n")
        .or_else(|| find_sec.strip_prefix("find:"))
        .ok_or_else(|| anyhow::anyhow!("Second section must start with 'find:'"))?
        .to_string();

    let replace = replace_sec
        .strip_prefix("replace:\n")
        .or_else(|| replace_sec.strip_prefix("replace:"))
        .ok_or_else(|| anyhow::anyhow!("Third section must start with 'replace:'"))?
        .to_string();

    Ok(PatchOp { file, find, replace })
}

/// Parse a block that uses "file:", "find:", "replace:" as line-prefix markers
/// (no --- separator needed). Everything between find: and replace: is the find
/// content; everything after replace: is the replace content.
fn parse_by_markers(block: &str) -> Result<PatchOp> {
    let lines: Vec<&str> = block.split('\n').collect();
    let mut file: Option<String> = None;
    let mut find_start: Option<usize> = None;
    let mut replace_start: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // "file: <path>"
        if let Some(rest) = trimmed.strip_prefix("file:") {
            if file.is_none() {
                let val = rest.trim();
                if val.is_empty() {
                    anyhow::bail!("'file:' marker has empty value on line {}", i + 1);
                }
                file = Some(val.to_string());
            }
            continue;
        }
        // "find:" or "find:" with trailing whitespace only
        if trimmed == "find:" {
            if find_start.is_none() {
                find_start = Some(i + 1);
            }
            continue;
        }
        // "replace:" or "replace:" with trailing whitespace only
        if trimmed == "replace:" {
            if replace_start.is_none() {
                replace_start = Some(i + 1);
            }
            continue;
        }
    }

    let file = file.ok_or_else(||
        anyhow::anyhow!("Missing 'file:' marker.\n  A patch block must start with:  file: <path>")
    )?;
    let find_s = find_start.ok_or_else(||
        anyhow::anyhow!("Missing 'find:' marker in block for file '{}'.\n  Add a line containing exactly:  find:", file)
    )?;
    let replace_s = replace_start.ok_or_else(||
        anyhow::anyhow!("Missing 'replace:' marker in block for file '{}'.\n  Add a line containing exactly:  replace:", file)
    )?;

    if replace_s <= find_s {
        anyhow::bail!("'replace:' must come after 'find:' in block for file '{}'", file);
    }

    // Find content: lines between find_s and (replace_s - 1)
    let mut find_lines: Vec<&str> = lines[find_s..replace_s - 1].to_vec();
    while find_lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        find_lines.pop();
    }
    if find_lines.is_empty() {
        anyhow::bail!("'find:' section is empty in block for file '{}'", file);
    }

    // Replace content: everything from replace_s to end
    let mut replace_lines: Vec<&str> = lines[replace_s..].to_vec();
    while replace_lines.last().map(|s| s.trim().is_empty()).unwrap_or(false) {
        replace_lines.pop();
    }

    Ok(PatchOp {
        file,
        find: find_lines.join("\n"),
        replace: replace_lines.join("\n"),
    })
}

/// Validate that a .orepatch file is parseable and return a summary.
/// Used by --validate flag and by external tools (GUI pre-flight).
pub fn validate_patch_content(content: &str) -> Result<PatchValidation> {
    let ops = parse_patch_file(content)?;
    let mut files = std::collections::BTreeSet::new();
    for op in &ops {
        files.insert(op.file.clone());
    }
    Ok(PatchValidation {
        op_count: ops.len(),
        file_count: files.len(),
        files: files.into_iter().collect(),
    })
}

#[derive(Debug)]
pub struct PatchValidation {
    pub op_count: usize,
    pub file_count: usize,
    pub files: Vec<String>,
}
