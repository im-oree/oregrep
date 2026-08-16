use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitAmendArgs {
    /// New commit message (leaves existing if omitted)
    #[arg(short = 'm', long)]
    message: Option<String>,

    /// Also add all staged changes (like --amend without --no-edit)
    #[arg(short = 'n', long)]
    no_edit: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: GitAmendArgs) -> Result<()> {
    ensure_repo()?;

    let head = git(&["log", "-1", "--pretty=format:%h %s"]).unwrap_or_default();
    println!("{} {}", "Amending:".cyan().bold(), head);

    if !args.yes {
        let ok = crate::engine::confirm::confirm("Proceed?", false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    if let Some(m) = &args.message {
        git(&["commit", "--amend", "-m", m])?;
    } else if args.no_edit {
        git(&["commit", "--amend", "--no-edit"])?;
    } else {
        git(&["commit", "--amend", "--no-edit"])?;
    }
    println!("{}", "Amended.".green().bold());
    Ok(())
}
