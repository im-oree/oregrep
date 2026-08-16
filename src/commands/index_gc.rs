use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::index::{gc_missing, open_index_if_exists};
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct IndexGcArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
}

pub fn run(args: IndexGcArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found."),
    };
    let removed = gc_missing(&conn)?;
    conn.execute("VACUUM", [])?;
    println!("{} {} orphaned entries removed, database vacuumed",
        "GC:".green().bold(), removed.to_string().yellow());
    Ok(())
}
