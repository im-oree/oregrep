use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct StaleFilesArgs {
    /// "180 days ago", "1 year ago", etc.
    #[arg(short = 'o', long, default_value = "180 days ago")]
    older_than: String,
    #[arg(short = 'p', long)]
    path: Option<String>,
    #[arg(short = 'n', long, default_value = "50")]
    top: usize,
}

pub fn run(args: StaleFilesArgs) -> Result<()> {
    ensure_repo()?;
    // For each tracked file, find last commit date
    let ls = git(&["ls-files"])?;
    let mut rows: Vec<(String, String)> = Vec::new();
    for f in ls.lines() {
        if f.trim().is_empty() { continue; }
        if let Some(p) = &args.path { if !f.contains(p) { continue; } }
        let last = git(&["log", "-1", "--format=%ad", "--date=short", "--", f]).unwrap_or_default();
        let last = last.trim().to_string();
        if !last.is_empty() { rows.push((f.to_string(), last)); }
    }
    // Filter by date: cutoff = date of the last commit before --older-than
    let cutoff = git(&["log", "-1", "--format=%ad", "--date=short", &format!("--before={}", args.older_than)]).unwrap_or_default();
    let cutoff = cutoff.trim().to_string();
    if cutoff.is_empty() {
        println!("{}", "Could not compute cutoff date. Try a different --older-than value.".yellow());
        return Ok(());
    }
    rows.retain(|(_, d)| d.as_str() < cutoff.as_str());
    rows.sort_by(|a, b| a.1.cmp(&b.1));

    println!("{} older than {} (cutoff: {})", "Stale files:".dimmed(), args.older_than.yellow(), cutoff.dimmed());
    for (f, d) in rows.iter().take(args.top) {
        println!("  {}  {}", d.dimmed(), f.cyan());
    }
    println!("\n{} {} stale files", "Total:".bold(), rows.len().to_string().yellow());
    Ok(())
}
