use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::{list_backups, restore_backup};

#[derive(Args)]
pub struct RestoreArgs {
    /// File to restore
    file: PathBuf,

    /// Label of the backup to restore (e.g. "CAMFIX"). If omitted, uses most recent.
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: RestoreArgs) -> Result<()> {
    let label = if let Some(l) = args.label {
        l
    } else {
        // Find most recent backup
        let backups = list_backups(&args.file)?;
        if backups.is_empty() {
            anyhow::bail!("No backups found for {}", args.file.display());
        }
        let most_recent = backups
            .iter()
            .max_by_key(|p| {
                std::fs::metadata(p)
                    .and_then(|m| m.modified())
                    .ok()
            })
            .unwrap();

        // Extract label from filename
        let fname = args.file.file_name().unwrap().to_string_lossy().to_string();
        let backup_fname = most_recent.file_name().unwrap().to_string_lossy().to_string();
        let prefix = format!("{}.bak", fname);
        backup_fname
            .strip_prefix(&prefix)
            .unwrap_or("")
            .to_string()
    };

    let backup_path = restore_backup(&args.file, &label)?;
    println!("{} {} {} {}",
        "Restored".green(),
        args.file.display().to_string().cyan(),
        "from".dimmed(),
        backup_path.display().to_string().yellow()
    );

    Ok(())
}
