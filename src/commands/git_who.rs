use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitWhoArgs {
    /// File to analyze
    file: String,
}

pub fn run(args: GitWhoArgs) -> Result<()> {
    ensure_repo()?;
    let out = git(&["log", "--pretty=format:%an", "--follow", "--", &args.file])?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in out.lines() {
        *counts.entry(line.to_string()).or_insert(0) += 1;
    }
    if counts.is_empty() {
        println!("{}", "No history for that file.".yellow());
        return Ok(());
    }
    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", format!("Contributors to {}:", args.file).cyan().bold());
    let total: usize = sorted.iter().map(|(_, c)| c).sum();
    for (name, count) in sorted {
        let pct = (count as f64 / total as f64) * 100.0;
        println!("  {:>4} ({:>5.1}%)  {}",
            count.to_string().yellow(),
            pct,
            name.cyan()
        );
    }
    Ok(())
}
