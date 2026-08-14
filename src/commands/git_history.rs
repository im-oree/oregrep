use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitHistoryArgs {
    /// File whose history to show
    file: String,

    /// Max commits (default 20)
    #[arg(short = 'n', long, default_value = "20")]
    limit: usize,

    /// Show patches (full diff per commit)
    #[arg(short = 'p', long)]
    patch: bool,
}

pub fn run(args: GitHistoryArgs) -> Result<()> {
    ensure_repo()?;
    let limit_str = args.limit.to_string();
    let mut cmd = vec!["log", "-n", limit_str.as_str(), "--color=always", "--pretty=format:%C(yellow)%h%C(reset) %C(cyan)%an%C(reset) %C(dim)%ar%C(reset) %s"];
    if args.patch { cmd.push("-p"); }
    cmd.push("--follow");
    cmd.push("--");
    cmd.push(&args.file);
    let out = git(&cmd)?;
    println!("{}", format!("History of {}", args.file).cyan().bold());
    println!();
    print!("{}", out);
    if !out.ends_with('\n') { println!(); }
    Ok(())
}
