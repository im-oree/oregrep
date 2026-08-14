use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct SliceArgs {
    /// File to slice
    file: PathBuf,

    /// Start pattern (regex)
    #[arg(short = 's', long)]
    start: String,

    /// End pattern (regex). If omitted, slices from start to EOF
    #[arg(short = 'e', long)]
    end: Option<String>,

    /// Include the start line
    #[arg(long, default_value = "true")]
    include_start: bool,

    /// Include the end line
    #[arg(long, default_value = "true")]
    include_end: bool,

    /// Extract every occurrence, not just the first
    #[arg(short = 'a', long)]
    all: bool,

    /// Print with a header for each slice
    #[arg(short = 'L', long)]
    label: bool,

    /// Show line numbers
    #[arg(short = 'N', long)]
    number: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: SliceArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }
    let content = read_file_smart(&args.file)?;
    let lines: Vec<&str> = content.lines().collect();

    let start_pat = if args.ignore_case { format!("(?i){}", args.start) } else { args.start.clone() };
    let start_re = Regex::new(&start_pat)?;
    let end_re = if let Some(e) = &args.end {
        let ep = if args.ignore_case { format!("(?i){}", e) } else { e.clone() };
        Some(Regex::new(&ep)?)
    } else {
        None
    };

    let mut slices: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if start_re.is_match(lines[i]) {
            let slice_start = i;
            let mut slice_end = lines.len() - 1;
            if let Some(er) = &end_re {
                for j in (i + 1)..lines.len() {
                    if er.is_match(lines[j]) {
                        slice_end = j;
                        break;
                    }
                }
            }
            let s = if args.include_start { slice_start } else { (slice_start + 1).min(slice_end) };
            let e = if args.include_end { slice_end } else { slice_end.saturating_sub(1).max(s) };
            slices.push((s, e));
            if !args.all { break; }
            i = slice_end + 1;
        } else {
            i += 1;
        }
    }

    if slices.is_empty() {
        eprintln!("{} No slices matched", "!".yellow());
        return Ok(());
    }

    let mut out_buf = String::new();
    let to_file = args.output.is_some();

    for (s, e) in &slices {
        if args.label {
            let hdr = format!("=== {}:{}-{} ===", args.file.display(), s + 1, e + 1);
            if to_file { out_buf.push_str(&hdr); out_buf.push('\n'); }
            else { println!("{}", hdr.cyan().bold()); }
        }
        for i in *s..=*e {
            if i >= lines.len() { break; }
            let line = lines[i];
            let s = if args.number { format!("{:>6} | {}", i + 1, line) } else { line.to_string() };
            if to_file { out_buf.push_str(&s); out_buf.push('\n'); }
            else { println!("{}", s); }
        }
    }

    if let Some(o) = args.output {
        std::fs::write(&o, out_buf)?;
        println!("{} {}", "Wrote:".green().bold(), o.display().to_string().cyan());
    }

    Ok(())
}
