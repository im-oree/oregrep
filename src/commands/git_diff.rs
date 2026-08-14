use anyhow::Result;
use clap::Args;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitDiffArgs {
    /// Specific file (default: entire repo)
    file: Option<String>,

    /// Show staged diff instead of unstaged
    #[arg(short = 's', long)]
    staged: bool,

    /// Diff against a specific commit
    #[arg(short = 'c', long)]
    commit: Option<String>,

    /// Stats only
    #[arg(long)]
    stat: bool,
}

pub fn run(args: GitDiffArgs) -> Result<()> {
    ensure_repo()?;
    let mut cmd: Vec<String> = vec!["diff".to_string()];
    if args.staged { cmd.push("--cached".to_string()); }
    if args.stat { cmd.push("--stat".to_string()); }
    cmd.push("--color=always".to_string());
    if let Some(c) = &args.commit { cmd.push(c.clone()); }
    if let Some(f) = &args.file {
        cmd.push("--".to_string());
        cmd.push(f.clone());
    }
    let args_ref: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
    let out = git(&args_ref)?;
    print!("{}", out);
    Ok(())
}
