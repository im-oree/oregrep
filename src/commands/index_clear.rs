use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::index::resolve_db_path;
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct IndexClearArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: IndexClearArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let db_path = resolve_db_path(&root_abs)?;
    if !db_path.exists() {
        println!("{}", "No index to clear.".yellow());
        return Ok(());
    }
    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Delete index at {}?", db_path.display()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }
    std::fs::remove_file(&db_path)?;
    // Also remove -wal / -shm sidecars if present
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
    // Try to remove .ore-index/ dir if empty
    if let Some(parent) = db_path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
    println!("{} {}", "Cleared:".green().bold(), db_path.display().to_string().cyan());
    Ok(())
}
