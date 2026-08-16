use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitUndoCommitArgs {
    /// How many commits to undo (default 1)
    #[arg(short = 'n', long, default_value = "1")]
    count: usize,

    /// Hard reset (loses changes) — default is soft (keeps changes staged)
    #[arg(long)]
    hard: bool,

    /// Mixed reset (keeps changes unstaged)
    #[arg(long, conflicts_with = "hard")]
    mixed: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitUndoCommitArgs) -> Result<()> {
    ensure_repo()?;
    let target = format!("HEAD~{}", args.count);

    // Show what will be undone
    let log = git(&["log", &format!("-{}", args.count), "--oneline"])?;
    println!("{}", "Commits to undo:".cyan().bold());
    for line in log.lines() { println!("  {}", line); }

    let mode = if args.hard { "hard".red().to_string() }
        else if args.mixed { "mixed".yellow().to_string() }
        else { "soft".green().to_string() };
    println!("\nReset mode: {}", mode);
    if args.hard {
        println!("{}", "  WARNING: hard reset will DISCARD uncommitted changes.".red().bold());
    }

    if !args.yes {
        let ok = crate::engine::confirm::confirm("Proceed?", false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let flag = if args.hard { "--hard" } else if args.mixed { "--mixed" } else { "--soft" };
    git(&["reset", flag, &target])?;
    println!("{} {} commit(s) undone ({})", "Done:".green().bold(), args.count, mode);
    Ok(())
}
