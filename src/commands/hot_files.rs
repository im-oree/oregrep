use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct HotFilesArgs {
    #[arg(short = 's', long, default_value = "90 days ago")]
    since: String,
    #[arg(short = 'p', long)]
    path: Option<String>,
    #[arg(short = 'n', long, default_value = "20")]
    top: usize,
}

pub fn run(args: HotFilesArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd = vec!["log".to_string(), "--name-only".to_string(), "--pretty=format:__COMMIT__".to_string(), format!("--since={}", args.since)];
    if let Some(p) = &args.path {
        cmd.push("--".to_string());
        cmd.push(p.clone());
    }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for line in out.lines() {
        let l = line.trim();
        if l.is_empty() || l == "__COMMIT__" { continue; }
        *counts.entry(l.to_string()).or_insert(0) += 1;
    }
    let mut rows: Vec<(String, usize)> = counts.into_iter().collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{} since {}", "Hot files:".red().bold(), args.since.yellow());
    for (f, n) in rows.iter().take(args.top) {
        let color = if *n >= 20 { "red" } else if *n >= 10 { "yellow" } else { "green" };
        println!("  {:>4}  {}", n.to_string().color(color).bold(), f.cyan());
    }
    Ok(())
}
