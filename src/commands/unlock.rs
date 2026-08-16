use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::compile::{load_locks, save_locks};

#[derive(Args)]
pub struct UnlockArgs {
    files: Vec<PathBuf>,

    /// Unlock all
    #[arg(short = 'a', long)]
    all: bool,
}

pub fn run(args: UnlockArgs) -> Result<()> {
    let mut reg = load_locks()?;
    if args.all {
        let n = reg.locked.len();
        reg.locked.clear();
        save_locks(&reg)?;
        println!("{} {} files", "Unlocked:".green().bold(), n.to_string().yellow());
        return Ok(());
    }
    if args.files.is_empty() { anyhow::bail!("At least one file required, or use --all"); }
    let mut removed = 0usize;
    for f in &args.files {
        let canonical = match std::fs::canonicalize(f) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => f.to_string_lossy().to_string(),
        };
        let clean = canonical.strip_prefix(r"\\?\").unwrap_or(&canonical).to_string();
        let before = reg.locked.len();
        reg.locked.retain(|p| p != &clean);
        if reg.locked.len() < before {
            removed += 1;
            println!("  {} {}", "UNLOCKED".green().bold(), clean.cyan());
        } else {
            println!("  {} {} (not locked)", "SKIP".yellow(), clean.dimmed());
        }
    }
    save_locks(&reg)?;
    println!("\n{} {} files unlocked", "Summary:".bold(), removed.to_string().green());
    Ok(())
}
