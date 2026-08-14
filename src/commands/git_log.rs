use anyhow::Result;
use clap::Args;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitLogArgs {
    /// Max commits
    #[arg(short = 'n', long, default_value = "20")]
    limit: usize,

    /// Graph view
    #[arg(short = 'g', long)]
    graph: bool,

    /// Only my commits (matches git config user.name)
    #[arg(long)]
    mine: bool,

    /// Filter by author substring
    #[arg(long)]
    author: Option<String>,

    /// Filter by commit message substring
    #[arg(long)]
    grep: Option<String>,

    /// Since date (e.g. "2 weeks ago", "2025-01-01")
    #[arg(long)]
    since: Option<String>,

    /// Until date
    #[arg(long)]
    until: Option<String>,
}

pub fn run(args: GitLogArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["log".to_string(), "--color=always".to_string()];
    let limit_str = args.limit.to_string();
    cmd.push("-n".to_string());
    cmd.push(limit_str);
    if args.graph { cmd.push("--graph".to_string()); }
    if args.mine {
        let me = git(&["config", "user.name"]).unwrap_or_default().trim().to_string();
        if !me.is_empty() {
            cmd.push(format!("--author={}", me));
        }
    }
    if let Some(a) = &args.author { cmd.push(format!("--author={}", a)); }
    if let Some(g) = &args.grep { cmd.push(format!("--grep={}", g)); }
    if let Some(s) = &args.since { cmd.push(format!("--since={}", s)); }
    if let Some(u) = &args.until { cmd.push(format!("--until={}", u)); }
    cmd.push("--pretty=format:%C(yellow)%h%C(reset) %C(cyan)%an%C(reset) %C(dim)%ar%C(reset) %s".to_string());
    let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&refs)?;
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
    Ok(())
}
