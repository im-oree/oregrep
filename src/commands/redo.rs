use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::history::{list_recent, mark_redone};
use crate::engine::index::open_index_if_exists;
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct RedoArgs {
    #[arg(default_value = ".")]
    root: PathBuf,

    #[arg(short = 'n', long, default_value = "1")]
    count: usize,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: RedoArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found."),
    };
    // Find most-recently-undone entries (undone=1, most recent first)
    let all = list_recent(&conn, 500, None, true)?;
    let undone: Vec<_> = all.into_iter().filter(|e| e.undone).take(args.count).collect();
    if undone.is_empty() { println!("{}", "Nothing to redo.".yellow()); return Ok(()); }

    println!("{} {} operation(s):", "Will redo:".cyan().bold(), undone.len().to_string().yellow());
    for e in &undone {
        let file = e.file.as_deref().unwrap_or("");
        println!("  [{}] {} {}", e.id.to_string().dimmed(), e.operation.magenta(), file.cyan());
    }
    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Redo {} operations?", undone.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }
    // Redo: we can only mark as done again if the current file matches what we recorded
    // (proper redo requires re-applying the exact op which we don't fully log for arbitrary edits).
    // For now: mark as redone in history + inform user to re-run the operation manually.
    for e in &undone {
        mark_redone(&conn, e.id)?;
        println!("  [{}] {} marked as redone (re-run the operation manually if content differs)", e.id.to_string().dimmed(), "~".yellow());
    }
    println!("\n{}", "Note: full replay of arbitrary operations isn't implemented; this only clears the 'undone' flag.".dimmed());
    Ok(())
}
