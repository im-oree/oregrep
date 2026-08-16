use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::index::resolve_db_path;
use crate::engine::paths::canonicalize_clean;

#[derive(Args)]
pub struct IndexLocateArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
}

pub fn run(args: IndexLocateArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let db_path = resolve_db_path(&root_abs)?;
    println!("{}", db_path.display());
    Ok(())
}
