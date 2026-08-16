use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct ExtractArgs {
    /// File to extract from (omit if using --spec or --spec-file)
    file: Option<PathBuf>,

    /// Line ranges. Comma-separated. Formats: "10", "10-30", "10:30", "10-30,50-70,100"
    ranges: Option<String>,

    /// Multi-file spec: "file1:10-30,file2:5-15,file3:100-200"
    #[arg(long)]
    spec: Option<String>,

    /// Load specs from a file (one spec per line, format: "path:range1,range2")
    #[arg(long)]
    spec_file: Option<PathBuf>,

    /// Prepend a === file:range === label before each chunk
    #[arg(short = 'L', long)]
    label: bool,

    /// Include N lines of context before/after each range
    #[arg(short = 'C', long, default_value = "0")]
    context: usize,

    /// Show line numbers
    #[arg(short = 'n', long = "number", visible_alias = "line-numbers")]
    number: bool,

    /// Merge overlapping/adjacent ranges within same file
    #[arg(short = 'm', long)]
    merge: bool,

    /// Write output to file instead of stdout
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Suppress colors (for pipe/redirect)
    #[arg(long)]
    plain: bool,
}

pub fn run(args: ExtractArgs) -> Result<()> {
    let mut jobs: Vec<(PathBuf, Vec<(usize, usize)>)> = Vec::new();

    if let Some(sf) = &args.spec_file {
        let content = read_file_smart(sf).with_context(|| format!("Reading spec file: {}", sf.display()))?;
        for (idx, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let (path, ranges) = parse_spec_line(line)
                .with_context(|| format!("Spec file line {}: {}", idx + 1, line))?;
            jobs.push((path, ranges));
        }
    } else if let Some(spec) = &args.spec {
        // Multiple files separated by commas is tricky since ranges also use commas.
        // Use ';' as file separator: "file1:10-20;file2:5-10,15-20"
        for chunk in spec.split(';') {
            let chunk = chunk.trim();
            if chunk.is_empty() { continue; }
            let (path, ranges) = parse_spec_line(chunk)?;
            jobs.push((path, ranges));
        }
    } else {
        let file = args.file.clone().ok_or_else(|| anyhow::anyhow!("Provide a file+ranges, or use --spec / --spec-file"))?;
        let ranges = args.ranges.clone().ok_or_else(|| anyhow::anyhow!("Provide ranges (e.g. \"10-30,50-70\")"))?;
        let parsed = parse_ranges(&ranges)?;
        jobs.push((file, parsed));
    }

    // Collect output
    let mut output_buf = String::new();
    let write_out = |s: &str, buf: &mut String, out_file: bool| {
        if out_file {
            buf.push_str(s);
            buf.push('\n');
        } else {
            println!("{}", s);
        }
    };

    let to_file = args.output.is_some();
    let plain = args.plain || to_file;

    for (path, mut ranges) in jobs {
        if !path.exists() {
            eprintln!("{} {}", "MISSING:".red(), path.display());
            continue;
        }
        let content = read_file_smart(&path)?;
        let all_lines: Vec<&str> = content.lines().collect();
        let total = all_lines.len();

        if args.merge {
            ranges = merge_ranges(ranges);
        }

        for (from, to) in ranges {
            let f = from.saturating_sub(args.context).max(1);
            let t = (to + args.context).min(total);
            if f > total { continue; }

            if args.label {
                let hdr = format!("=== {}:{}-{} ===", path.display(), from, to);
                let styled = if plain { hdr.clone() } else { hdr.cyan().bold().to_string() };
                write_out(&styled, &mut output_buf, to_file);
            }

            for i in f..=t {
                if i == 0 || i > total { continue; }
                let line = all_lines[i - 1];
                let s = if args.number {
                    format!("{:>6} | {}", i, line)
                } else {
                    line.to_string()
                };
                write_out(&s, &mut output_buf, to_file);
            }
        }
    }

    if let Some(out) = args.output {
        std::fs::write(&out, output_buf)?;
        println!("{} {}", "Wrote:".green().bold(), out.display().to_string().cyan());
    }

    Ok(())
}

fn parse_spec_line(s: &str) -> Result<(PathBuf, Vec<(usize, usize)>)> {
    // "path:range1,range2,range3"
    // On Windows we may have "C:\path\file.rs:10-20"
    // Split from the RIGHT on the last colon that isn't part of a drive letter
    let (path_str, range_str) = split_path_and_ranges(s)
        .ok_or_else(|| anyhow::anyhow!("Spec must be 'path:ranges', got: {}", s))?;
    let path = PathBuf::from(path_str);
    let ranges = parse_ranges(range_str)?;
    Ok((path, ranges))
}

fn split_path_and_ranges(s: &str) -> Option<(&str, &str)> {
    // Find the LAST ':' that's followed by a digit
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b':' {
            if i + 1 < bytes.len() && bytes[i + 1].is_ascii_digit() {
                // But not if this is drive letter (like "C:")
                if i >= 1 && bytes[i - 1].is_ascii_alphabetic() && (i == 1 || bytes[i - 2] == b'/' || bytes[i - 2] == b'\\') {
                    continue;
                }
                return Some((&s[..i], &s[i + 1..]));
            }
        }
    }
    None
}

fn parse_ranges(s: &str) -> Result<Vec<(usize, usize)>> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let p = part.trim();
        if p.is_empty() { continue; }
        let (from, to) = parse_single_range(p)?;
        out.push((from, to));
    }
    Ok(out)
}

fn parse_single_range(s: &str) -> Result<(usize, usize)> {
    let sep = if s.contains(':') { ':' } else { '-' };
    let parts: Vec<&str> = s.split(sep).collect();
    match parts.len() {
        1 => {
            let n: usize = parts[0].parse().map_err(|_| anyhow::anyhow!("Bad line number: {}", s))?;
            Ok((n, n))
        }
        2 => {
            let a: usize = parts[0].parse().map_err(|_| anyhow::anyhow!("Bad range start: {}", parts[0]))?;
            let b: usize = parts[1].parse().map_err(|_| anyhow::anyhow!("Bad range end: {}", parts[1]))?;
            Ok((a.min(b), a.max(b)))
        }
        _ => anyhow::bail!("Invalid range: {}", s),
    }
}

fn merge_ranges(mut ranges: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    if ranges.is_empty() { return ranges; }
    ranges.sort_by_key(|r| r.0);
    let mut out = vec![ranges[0]];
    for r in ranges.into_iter().skip(1) {
        let last = out.last_mut().unwrap();
        if r.0 <= last.1 + 1 {
            last.1 = last.1.max(r.1);
        } else {
            out.push(r);
        }
    }
    out
}
