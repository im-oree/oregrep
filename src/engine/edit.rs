use anyhow::Result;
use std::path::Path;

use crate::engine::backup::create_backup;
use crate::engine::patch::{read_for_patch, write_atomic};

/// Common pipeline used by all surgical edit commands:
/// - Read file with encoding preserved
/// - Apply the transformation
/// - Optionally back up
/// - Atomic write
pub struct EditOptions {
    pub no_backup: bool,
    pub label: Option<String>,
    pub dry_run: bool,
}

pub struct EditResult {
    pub lines_before: usize,
    pub lines_after: usize,
    pub backup_path: Option<std::path::PathBuf>,
}

/// Perform an edit by running `transform` on the file's line list.
/// The transform receives Vec<String> and returns Vec<String>.
pub fn edit_lines<F>(file: &Path, opts: &EditOptions, transform: F) -> Result<EditResult>
where
    F: FnOnce(Vec<String>) -> Result<Vec<String>>,
{
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    let (content, had_bom, newline) = read_for_patch(file)?;
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let lines_before = lines.len();

    let new_lines = transform(lines)?;
    let lines_after = new_lines.len();

    // Preserve trailing newline behavior: if original ended in newline, keep it
    let trailing_nl = content.ends_with('\n') || content.ends_with("\r\n");
    let mut new_content = new_lines.join(newline);
    if trailing_nl {
        new_content.push_str(newline);
    }

    let mut backup_path = None;
    if !opts.dry_run && !opts.no_backup {
        let label = opts
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        backup_path = Some(create_backup(file, &label)?);
    }

    if !opts.dry_run {
        write_atomic(file, &new_content, had_bom)?;
    }

    Ok(EditResult {
        lines_before,
        lines_after,
        backup_path,
    })
}

/// Parse a line spec: "42" -> (42, 42), "10:20" -> (10, 20), "10-20" -> (10, 20).
/// Returns 1-indexed inclusive (from, to).
pub fn parse_line_range(s: &str, total: usize) -> Result<(usize, usize)> {
    let sep = if s.contains(':') { ':' } else { '-' };
    let parts: Vec<&str> = s.split(sep).collect();
    match parts.len() {
        1 => {
            let n: usize = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid line number: {}", s))?;
            if n == 0 || n > total {
                anyhow::bail!("Line {} out of range (file has {} lines)", n, total);
            }
            Ok((n, n))
        }
        2 => {
            let a: usize = parts[0]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid range start: {}", parts[0]))?;
            let b: usize = parts[1]
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid range end: {}", parts[1]))?;
            if a == 0 || b == 0 {
                anyhow::bail!("Line numbers are 1-indexed");
            }
            if a > b {
                anyhow::bail!("Range start ({}) > end ({})", a, b);
            }
            if a > total {
                anyhow::bail!("Range {}-{} out of bounds (file has {} lines)", a, b, total);
            }
            Ok((a, b.min(total)))
        }
        _ => anyhow::bail!("Invalid range format: {}. Use N or N:M or N-M", s),
    }
}
