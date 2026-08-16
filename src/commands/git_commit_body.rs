use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::commit_msg::{analyze_diff, compose_message};
use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitCommitBodyArgs {
    /// Subject line you want
    subject: String,

    /// Analyze staged (default: staged)
    #[arg(short = 'u', long)]
    unstaged: bool,

    /// Just preview, don't commit
    #[arg(short = 'p', long)]
    preview: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitCommitBodyArgs) -> Result<()> {
    ensure_repo()?;
    let a = analyze_diff(!args.unstaged)?;
    let body_msg = compose_message(&a, "simple", true);
    // Take just the body part (skip first line + blank)
    let body = body_msg.splitn(3, '\n').nth(2).unwrap_or("");
    let full = if body.is_empty() { args.subject.clone() } else { format!("{}\n\n{}", args.subject, body) };

    println!("{}", "Message:".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", full);
    println!("{}", "─".repeat(60).dimmed());

    if args.preview { return Ok(()); }

    if !args.yes {
        let ok = crate::engine::confirm::confirm("Commit?", false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }
    git(&["commit", "-m", &full])?;
    println!("{}", "Committed.".green().bold());
    Ok(())
}
