use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::hex::format_hex_dump;

#[derive(Args)]
pub struct HexDiffArgs {
    file_a: PathBuf,
    file_b: PathBuf,

    /// Max differences to show (0 = all)
    #[arg(short = 'n', long, default_value = "50")]
    max: usize,

    /// Context bytes around each diff
    #[arg(short = 'C', long, default_value = "0")]
    context: usize,
}

pub fn run(args: HexDiffArgs) -> Result<()> {
    if !args.file_a.exists() { anyhow::bail!("File not found: {}", args.file_a.display()); }
    if !args.file_b.exists() { anyhow::bail!("File not found: {}", args.file_b.display()); }
    let a = std::fs::read(&args.file_a)?;
    let b = std::fs::read(&args.file_b)?;

    let mut diffs: Vec<usize> = Vec::new();
    let min_len = a.len().min(b.len());
    for i in 0..min_len {
        if a[i] != b[i] { diffs.push(i); }
    }
    let size_note = if a.len() != b.len() {
        format!("sizes differ: A={} B={}, diff after byte {}", a.len(), b.len(), min_len)
    } else {
        format!("both {} bytes", a.len())
    };

    println!("{}  {}", "Hex diff:".cyan().bold(), size_note.dimmed());
    println!("{} {}",  "A:".red(), args.file_a.display().to_string().cyan());
    println!("{} {}",  "B:".green(), args.file_b.display().to_string().cyan());
    println!("{} {} byte differences (in common region)", "Diffs:".yellow(), diffs.len().to_string().yellow());

    let n = if args.max == 0 { diffs.len() } else { diffs.len().min(args.max) };
    for offset in diffs.iter().take(n) {
        let start = offset.saturating_sub(args.context);
        let end = (offset + 1 + args.context).min(min_len);
        println!("\n{} at {:#010x}", "@@".magenta(), offset);
        println!("  {} {}", "A:".red(), format_hex_dump(&a[start..end], start, 16).trim_end());
        println!("  {} {}", "B:".green(), format_hex_dump(&b[start..end], start, 16).trim_end());
    }
    if diffs.len() > n {
        println!("\n{} {} more differences not shown (use -n 0 for all)", "…".dimmed(), (diffs.len() - n).to_string().dimmed());
    }
    if diffs.is_empty() && a.len() == b.len() {
        println!("\n{}", "Files are byte-identical.".green().bold());
    } else if a.len() != b.len() {
        std::process::exit(1);
    } else if !diffs.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
