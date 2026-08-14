use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::hex::{find_all, format_hex_dump, parse_hex_pattern};

#[derive(Args)]
pub struct HexFindArgs {
    file: PathBuf,

    /// Hex pattern (supports ?? wildcards, spaces optional). Examples: "deadbeef", "de ad ?? ef"
    pattern: String,

    /// Show N bytes of context around each match
    #[arg(short = 'C', long, default_value = "16")]
    context: usize,

    /// Max matches to show (0 = unlimited)
    #[arg(short = 'n', long, default_value = "0")]
    max: usize,

    /// Only show offsets, no hex dump
    #[arg(short = 'o', long)]
    offsets_only: bool,
}

pub fn run(args: HexFindArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let bytes = std::fs::read(&args.file)?;
    let pat = parse_hex_pattern(&args.pattern)?;
    if pat.is_empty() { anyhow::bail!("Empty pattern"); }

    let hits = find_all(&bytes, &pat);
    let n = if args.max == 0 { hits.len() } else { hits.len().min(args.max) };

    println!("{} {} matches (showing {})",
        "Found:".cyan().bold(),
        hits.len().to_string().yellow(),
        n.to_string().yellow()
    );

    for offset in hits.iter().take(n) {
        if args.offsets_only {
            println!("  {}  {:#010x}", offset.to_string().dimmed(), offset);
            continue;
        }
        let ctx_start = offset.saturating_sub(args.context);
        let ctx_end = (offset + pat.len() + args.context).min(bytes.len());
        println!("\n{} {} (match at {:#010x})", "──".magenta(), format!("[{}]", offset).yellow(), offset);
        print!("{}", format_hex_dump(&bytes[ctx_start..ctx_end], ctx_start, 16));
    }
    Ok(())
}
