use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::git::{changed_files, ensure_repo};

#[derive(Args)]
pub struct GitStatusArgs {
    /// Short format (single-char per file)
    #[arg(short = 's', long)]
    short: bool,
}

pub fn run(args: GitStatusArgs) -> Result<()> {
    ensure_repo()?;
    let files = changed_files()?;
    if files.is_empty() {
        println!("{}", "Clean working tree.".green());
        return Ok(());
    }

    let mut staged = 0;
    let mut unstaged = 0;
    let mut untracked = 0;

    for (status, path) in &files {
        let (label, color) = interpret_status(status);
        if status.starts_with('?') { untracked += 1; }
        else {
            if status.chars().next().unwrap_or(' ') != ' ' { staged += 1; }
            if status.chars().nth(1).unwrap_or(' ') != ' ' { unstaged += 1; }
        }

        if args.short {
            println!("{} {}", status.color(color), path);
        } else {
            println!("  {:<12} {}", label.color(color).bold(), path.cyan());
        }
    }

    println!("\n{} staged, {} unstaged, {} untracked",
        staged.to_string().green(),
        unstaged.to_string().yellow(),
        untracked.to_string().red()
    );
    Ok(())
}

fn interpret_status(s: &str) -> (&'static str, &'static str) {
    match s.chars().next().unwrap_or(' ') {
        'M' => ("modified", "yellow"),
        'A' => ("added", "green"),
        'D' => ("deleted", "red"),
        'R' => ("renamed", "cyan"),
        'C' => ("copied", "cyan"),
        'U' => ("conflict", "red"),
        '?' => ("untracked", "red"),
        _ => match s.chars().nth(1).unwrap_or(' ') {
            'M' => ("modified*", "yellow"),
            'D' => ("deleted*", "red"),
            _ => ("unknown", "white"),
        }
    }
}
