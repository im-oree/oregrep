use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::path::PathBuf;

use crate::engine::state::{read_focus, write_focus};

#[derive(Args)]
pub struct FocusArgs {
    #[command(subcommand)]
    pub action: FocusAction,
}

#[derive(Subcommand)]
pub enum FocusAction {
    /// Set the focus path (subsequent commands can default to this)
    Set { path: PathBuf },
    /// Show current focus
    Show,
    /// Clear focus
    Clear,
}

pub fn run(args: FocusArgs) -> Result<()> {
    match args.action {
        FocusAction::Set { path } => {
            if !path.exists() { anyhow::bail!("Path not found: {}", path.display()); }
            let canonical = std::fs::canonicalize(&path)?;
            let clean = strip_extended_prefix(canonical);
            write_focus(Some(&clean))?;
            println!("{} {}", "Focus set:".green().bold(), clean.display().to_string().cyan());
        }
        FocusAction::Show => {
            match read_focus()? {
                Some(p) => println!("{}", p.display().to_string().cyan()),
                None => println!("{}", "(no focus set)".dimmed()),
            }
        }
        FocusAction::Clear => {
            write_focus(None)?;
            println!("{}", "Focus cleared.".green());
        }
    }
    Ok(())
}

fn strip_extended_prefix(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p
    }
}
