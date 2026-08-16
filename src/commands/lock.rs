use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::compile::{load_locks, save_locks};

#[derive(Args)]
pub struct LockArgs {
    files: Vec<PathBuf>,
}

pub fn run(args: LockArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("At least one file required"); }
    let mut reg = load_locks()?;
    for f in &args.files {
        if !f.exists() { println!("  {} {}", "MISSING".red(), f.display()); continue; }
        let canonical = std::fs::canonicalize(f)?.to_string_lossy().to_string();
        let clean = canonical.strip_prefix(r"\\?\").unwrap_or(&canonical).to_string();
        if reg.locked.contains(&clean) {
            println!("  {} {} (already locked)", "SKIP".yellow(), clean.dimmed());
            continue;
        }
        reg.locked.push(clean.clone());
        println!("  {} {}", "LOCKED".green().bold(), clean.cyan());
    }
    save_locks(&reg)?;
    Ok(())
}
