use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct LineArgs {
    /// File to read
    file: PathBuf,

    /// Line number or range (e.g. "42" or "10:20" or "10-20")
    range: String,

    /// Suppress line numbers
    #[arg(short = 'N', long)]
    no_number: bool,

    /// Include N lines of context before/after
    #[arg(short = 'C', long, default_value = "0")]
    context: usize,
}

pub fn run(args: LineArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let content = read_file_smart(&args.file)?;
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    // Parse range
    let (start, end) = parse_range(&args.range, total)?;

    let ctx_start = start.saturating_sub(args.context);
    let ctx_end = (end + args.context).min(total);

    for i in ctx_start..ctx_end {
        let lineno = i + 1;
        let is_target = i + 1 >= start + 1 && i + 1 <= end;
        let line = lines[i];
        if args.no_number {
            println!("{}", line);
        } else if is_target {
            println!("{:>6} | {}", lineno.to_string().green().bold(), line);
        } else {
            println!("{:>6} | {}", lineno.to_string().dimmed(), line.dimmed());
        }
    }

    Ok(())
}

/// Parse "42" -> (41, 42), "10:20" -> (9, 20), "10-20" -> (9, 20).
/// Returns 0-indexed start (inclusive) and exclusive end.
fn parse_range(s: &str, total: usize) -> Result<(usize, usize)> {
    let sep = if s.contains(':') { ':' } else { '-' };
    let parts: Vec<&str> = s.split(sep).collect();
    match parts.len() {
        1 => {
            let n: usize = parts[0].parse().map_err(|_| anyhow::anyhow!("Invalid line number: {}", s))?;
            if n == 0 || n > total {
                anyhow::bail!("Line {} out of range (file has {} lines)", n, total);
            }
            Ok((n - 1, n))
        }
        2 => {
            let a: usize = parts[0].parse().map_err(|_| anyhow::anyhow!("Invalid range start: {}", parts[0]))?;
            let b: usize = parts[1].parse().map_err(|_| anyhow::anyhow!("Invalid range end: {}", parts[1]))?;
            if a == 0 || b == 0 || a > total {
                anyhow::bail!("Range {}-{} out of bounds (file has {} lines)", a, b, total);
            }
            Ok((a - 1, b.min(total)))
        }
        _ => anyhow::bail!("Invalid range format: {}. Use N or N:M or N-M", s),
    }
}
