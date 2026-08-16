use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::compile::load_locks;

#[derive(Args)]
pub struct LocksArgs {}

pub fn run(_args: LocksArgs) -> Result<()> {
    let reg = load_locks()?;
    if reg.locked.is_empty() {
        println!("{}", "(no locked files)".dimmed());
        return Ok(());
    }
    println!("{} {} locked file(s):", "Locks:".cyan().bold(), reg.locked.len().to_string().yellow());
    for p in &reg.locked {
        println!("  {} {}", "🔒".red(), p.cyan());
    }
    Ok(())
}
