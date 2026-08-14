use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::{create_backup, list_backups};

#[derive(Args)]
pub struct BackupArgs {
    /// File to back up
    file: PathBuf,

    /// Label suffix (e.g. "CAMFIX" -> file.ext.bakCAMFIX). Defaults to timestamp.
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Just list existing backups, don't create new
    #[arg(long)]
    list: bool,
}

pub fn run(args: BackupArgs) -> Result<()> {
    if args.list {
        let backups = list_backups(&args.file)?;
        if backups.is_empty() {
            println!("No backups found for {}", args.file.display());
        } else {
            println!("{}", format!("Backups for {}:", args.file.display()).cyan().bold());
            for b in &backups {
                let meta = std::fs::metadata(b).ok();
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        let dt: chrono::DateTime<chrono::Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());
                println!("  {}  {}  {}",
                    b.display().to_string().yellow(),
                    format!("{}B", size).dimmed(),
                    modified.dimmed()
                );
            }
        }
        return Ok(());
    }

    let label = args.label.unwrap_or_else(|| {
        chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
    });

    let backup_path = create_backup(&args.file, &label)?;
    println!("{} {}",
        "Backup created:".green(),
        backup_path.display().to_string().cyan()
    );

    Ok(())
}
