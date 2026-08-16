use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitStashNamedArgs {
    #[command(subcommand)]
    pub action: StashAction,
}

#[derive(Subcommand)]
pub enum StashAction {
    /// Save current changes with a named label
    Save { name: String },
    /// List named stashes
    List,
    /// Apply a named stash (keeps it in stash list)
    Apply { name: String },
    /// Pop a named stash (removes it)
    Pop { name: String },
    /// Drop a named stash without applying
    Drop { name: String },
    /// Show contents of a named stash
    Show { name: String },
}

pub fn run(args: GitStashNamedArgs) -> Result<()> {
    ensure_repo()?;
    match args.action {
        StashAction::Save { name } => {
            let msg = format!("ore-named:{}", name);
            git(&["stash", "push", "-m", &msg])?;
            println!("{} {}", "Stashed as:".green().bold(), name.cyan());
        }
        StashAction::List => {
            let out = git(&["stash", "list"])?;
            let re = regex::Regex::new(r"^(stash@\{\d+\}):\s+On\s+\S+:\s+ore-named:(.+)$").unwrap();
            let mut found = 0;
            for line in out.lines() {
                if let Some(cap) = re.captures(line) {
                    println!("  {} → {}", cap[2].to_string().cyan(), cap[1].to_string().dimmed());
                    found += 1;
                }
            }
            if found == 0 { println!("{}", "(no named stashes)".dimmed()); }
        }
        StashAction::Apply { name } => {
            let idx = find_stash_index(&name)?;
            git(&["stash", "apply", &idx])?;
            println!("{} {}", "Applied:".green().bold(), name.cyan());
        }
        StashAction::Pop { name } => {
            let idx = find_stash_index(&name)?;
            git(&["stash", "pop", &idx])?;
            println!("{} {}", "Popped:".green().bold(), name.cyan());
        }
        StashAction::Drop { name } => {
            let idx = find_stash_index(&name)?;
            git(&["stash", "drop", &idx])?;
            println!("{} {}", "Dropped:".green().bold(), name.cyan());
        }
        StashAction::Show { name } => {
            let idx = find_stash_index(&name)?;
            let out = git(&["stash", "show", "-p", &idx])?;
            print!("{}", out);
        }
    }
    Ok(())
}

fn find_stash_index(name: &str) -> Result<String> {
    let out = git(&["stash", "list"])?;
    let re = regex::Regex::new(r"^(stash@\{\d+\}):\s+On\s+\S+:\s+ore-named:(.+)$").unwrap();
    for line in out.lines() {
        if let Some(cap) = re.captures(line) {
            if cap[2].trim() == name {
                return Ok(cap[1].to_string());
            }
        }
    }
    anyhow::bail!("Named stash not found: {}", name)
}
