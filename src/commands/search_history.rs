use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct SearchHistoryArgs {
    /// String to search for (finds commits where it was added or removed)
    query: String,

    /// Restrict to a specific path
    #[arg(short = 'p', long)]
    path: Option<String>,

    /// Max commits to scan
    #[arg(short = 'n', long, default_value = "100")]
    limit: usize,

    /// Show the actual diff hunks
    #[arg(short = 'd', long)]
    diff: bool,

    /// Regex mode instead of literal
    #[arg(short = 'r', long)]
    regex: bool,
}

pub fn run(args: SearchHistoryArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["log".to_string(), "--color=always".to_string()];
    let limit_str = args.limit.to_string();
    cmd.push("-n".to_string());
    cmd.push(limit_str);

    if args.regex {
        cmd.push(format!("-G{}", args.query));
    } else {
        cmd.push(format!("-S{}", args.query));
        cmd.push("--pickaxe-all".to_string());
    }
    if args.diff { cmd.push("-p".to_string()); }
    cmd.push("--pretty=format:%C(yellow)%h%C(reset) %C(cyan)%an%C(reset) %C(dim)%ar%C(reset) %s".to_string());
    if let Some(p) = &args.path {
        cmd.push("--".to_string());
        cmd.push(p.clone());
    }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;
    let mode = if args.regex { "regex" } else { "literal" };
    println!("{}", format!("History search '{}' ({})", args.query, mode).cyan().bold());
    if let Some(p) = &args.path { println!("{} {}", "Scoped to:".dimmed(), p.dimmed()); }
    println!();
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
    Ok(())
}
