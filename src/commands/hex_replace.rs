use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::hex::{find_all, parse_hex_bytes, parse_hex_pattern};

#[derive(Args)]
pub struct HexReplaceArgs {
    file: PathBuf,

    /// Hex pattern to find (wildcards ?? allowed)
    find: String,

    /// Hex bytes to replace with (must NOT contain wildcards, MUST be same length)
    replace: String,

    /// Replace all occurrences (default: fail if not exactly 1)
    #[arg(short = 'a', long)]
    all: bool,

    /// Replace only Nth (1-indexed)
    #[arg(short = 'n', long)]
    nth: Option<usize>,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: HexReplaceArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let mut bytes = std::fs::read(&args.file)?;
    let pat = parse_hex_pattern(&args.find)?;
    let rep = parse_hex_bytes(&args.replace)?;
    if pat.is_empty() { anyhow::bail!("Empty find pattern"); }
    if pat.len() != rep.len() {
        anyhow::bail!("Replace length ({}) must equal find length ({}). Use hex-patch for variable-length.", rep.len(), pat.len());
    }

    let hits = find_all(&bytes, &pat);
    if hits.is_empty() {
        println!("{} No matches", "!".yellow());
        return Ok(());
    }
    let targets: Vec<usize> = if args.all {
        hits.clone()
    } else if let Some(n) = args.nth {
        if n == 0 || n > hits.len() { anyhow::bail!("--nth out of range (1..{})", hits.len()); }
        vec![hits[n - 1]]
    } else {
        if hits.len() > 1 {
            anyhow::bail!("Found {} matches, expected 1. Use --all or --nth N.", hits.len());
        }
        vec![hits[0]]
    };

    println!("{} {} matches, replacing {}",
        "Found:".cyan(),
        hits.len().to_string().yellow(),
        targets.len().to_string().green()
    );

    if args.dry_run {
        for t in &targets {
            println!("  {} would replace at {:#010x}", "[DRY]".yellow(), t);
        }
        return Ok(());
    }

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }

    for t in &targets {
        for (i, b) in rep.iter().enumerate() {
            bytes[t + i] = *b;
        }
    }
    std::fs::write(&args.file, &bytes)?;
    println!("{} {} ({} replacements, same-size)",
        "Patched:".green().bold(),
        args.file.display().to_string().cyan(),
        targets.len().to_string().yellow()
    );
    Ok(())
}
