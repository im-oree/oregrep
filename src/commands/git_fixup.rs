use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitFixupArgs {
    /// Target commit SHA (or ref like HEAD~3)
    target: String,

    /// Also start an interactive autosquash rebase after creating the fixup commit
    #[arg(short = 'r', long)]
    rebase: bool,
}

pub fn run(args: GitFixupArgs) -> Result<()> {
    ensure_repo()?;

    // Verify target exists
    let show = git(&["log", "-1", "--pretty=format:%h %s", &args.target])?;
    println!("{} {}", "Fixing up:".cyan().bold(), show);

    git(&["commit", "--fixup", &args.target])?;
    println!("{} fixup commit created", "OK:".green().bold());

    if args.rebase {
        println!("{}", "Running rebase --autosquash…".dimmed());
        let range = format!("{}~", args.target);
        // Use env vars to skip interactive editor
        let output = std::process::Command::new("git")
            .env("GIT_SEQUENCE_EDITOR", if cfg!(windows) { "cmd /C exit" } else { "true" })
            .args(["rebase", "-i", "--autosquash", &range])
            .output()?;
        if !output.status.success() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            anyhow::bail!("rebase failed");
        }
        println!("{}", "Rebase complete.".green().bold());
    } else {
        println!("Run `git rebase -i --autosquash {}~` to squash it in.", args.target.dimmed());
    }
    Ok(())
}
