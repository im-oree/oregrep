use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct SinceArgs {
    /// A date, duration, or git ref. Examples: "yesterday", "3 days ago", "HEAD~10", "v1.0.0"
    when: String,
    /// Include diff stats
    #[arg(short = 's', long)]
    stat: bool,
    /// Only these paths
    #[arg(short = 'p', long)]
    path: Option<String>,
}

pub fn run(args: SinceArgs) -> Result<()> {
    ensure_repo()?;
    // Decide: ref-like → use range, else --since=
    let is_ref = args.when.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '~' || c == '^' || c == '_' || c == '-') && !args.when.contains(' ');

    let mut cmd = vec!["log".to_string()];
    if is_ref {
        cmd.push(format!("{}..HEAD", args.when));
    } else {
        cmd.push(format!("--since={}", args.when));
    }
    if args.stat { cmd.push("--stat".to_string()); }
    cmd.push("--pretty=format:%h %an %ar %s".to_string());
    if let Some(p) = &args.path {
        cmd.push("--".to_string());
        cmd.push(p.clone());
    }
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let log = git(&refs)?;
    if log.trim().is_empty() {
        println!("{}", "No commits.".dimmed());
        return Ok(());
    }
    println!("{} {}", "Changes since:".cyan().bold(), args.when.yellow());
    println!();
    print!("{}", log);
    if !log.ends_with('\n') { println!(); }

    // Also show working tree changes
    let status = git(&["status", "--porcelain"]).unwrap_or_default();
    let dirty = status.lines().count();
    if dirty > 0 {
        println!("\n{} {} uncommitted files", "Working tree:".cyan().bold(), dirty.to_string().yellow());
    }
    Ok(())
}
