use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::history::{get_entry, list_recent, mark_undone};
use crate::engine::index::open_index_if_exists;
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct UndoArgs {
    #[arg(default_value = ".")]
    root: PathBuf,

    /// Undo the last N operations (default 1)
    #[arg(short = 'n', long, default_value = "1")]
    count: usize,

    /// Only undo entries for this file
    #[arg(short = 'f', long)]
    file: Option<String>,

    /// Preview what would be undone
    #[arg(long)]
    dry_run: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: UndoArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found. Undo requires the index."),
    };

    // Get last N non-undone entries (optionally filtered)
    let candidates = list_recent(&conn, args.count as i64, args.file.as_deref(), false)?;
    if candidates.is_empty() {
        println!("{}", "Nothing to undo.".yellow());
        return Ok(());
    }

    println!("{} {} operation(s):", "Will undo:".cyan().bold(), candidates.len().to_string().yellow());
    for e in &candidates {
        let file = e.file.as_deref().unwrap_or("");
        let bak = e.backup.as_deref().map(|s| format!(" ← {}", s)).unwrap_or_default();
        println!("  [{}] {} {}{}", e.id.to_string().dimmed(), e.operation.magenta(), file.cyan(), bak.dimmed());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing undone]".yellow().bold());
        return Ok(());
    }
    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Undo {} operations?", candidates.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let mut done = 0usize;
    let mut skipped = 0usize;
    for e in &candidates {
        let entry = match get_entry(&conn, e.id) { Some(x) => x, None => continue };
        let restored = apply_undo(&entry)?;
        if restored {
            mark_undone(&conn, e.id)?;
            done += 1;
            println!("  {} [{}] restored", "OK".green(), e.id.to_string().dimmed());
        } else {
            skipped += 1;
            println!("  {} [{}] cannot undo (no backup / unsupported op)", "SKIP".yellow(), e.id.to_string().dimmed());
        }
    }
    println!("\n{} {} undone, {} skipped", "Done:".green().bold(), done.to_string().green(), skipped.to_string().yellow());
    Ok(())
}

fn apply_undo(entry: &crate::engine::history::HistoryEntry) -> Result<bool> {
    // Only ops with a recorded backup can be reliably undone
    let backup = match &entry.backup {
        Some(b) => PathBuf::from(b),
        None => return Ok(false),
    };
    let target = match &entry.file {
        Some(f) => PathBuf::from(f),
        None => return Ok(false),
    };
    if !backup.exists() { return Ok(false); }
    // Restore backup → target
    std::fs::copy(&backup, &target)?;
    Ok(true)
}
