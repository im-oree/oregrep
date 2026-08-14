use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitSearchArgs {
    /// Text to search in git history
    query: String,

    /// Search commit messages
    #[arg(long)]
    messages: bool,

    /// Search introduced/removed content (default)
    #[arg(long)]
    content: bool,

    /// Limit results
    #[arg(short = 'n', long, default_value = "50")]
    limit: usize,
}

pub fn run(args: GitSearchArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["log".to_string(), "--color=always".to_string()];
    let limit_str = args.limit.to_string();
    cmd.push("-n".to_string());
    cmd.push(limit_str);
    if args.messages {
        cmd.push(format!("--grep={}", args.query));
    } else {
        // Content search: pickaxe
        cmd.push(format!("-S{}", args.query));
        cmd.push("--pickaxe-all".to_string());
    }
    cmd.push("--pretty=format:%C(yellow)%h%C(reset) %C(cyan)%an%C(reset) %C(dim)%ar%C(reset) %s".to_string());
    let args_ref: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&args_ref)?;
    let mode = if args.messages { "commit messages" } else { "history content" };
    println!("{}", format!("Search '{}' in {}", args.query, mode).cyan().bold());
    println!();
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
    Ok(())
}
