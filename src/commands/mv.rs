use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::confirm::confirm;

#[derive(Args)]
pub struct MvArgs {
    /// Source file or directory
    src: PathBuf,

    /// Destination path
    dst: PathBuf,

    /// Bypass confirmation for overwrites
    #[arg(short = 'y', long)]
    yes: bool,

    /// Force overwrite even if target exists (no backup)
    #[arg(long)]
    force: bool,

    /// Skip backing up target on overwrite
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: MvArgs) -> Result<()> {
    if !args.src.exists() {
        anyhow::bail!("Source not found: {}", args.src.display());
    }

    // If dst is a directory, target = dst + src filename
    let target = if args.dst.is_dir() {
        let fname = args.src.file_name().ok_or_else(|| anyhow::anyhow!("Invalid src filename"))?;
        args.dst.join(fname)
    } else {
        args.dst.clone()
    };

    if target.exists() {
        if !args.force {
            let ok = confirm(&format!("Overwrite existing {}?", target.display()), args.yes)?;
            if !ok {
                println!("{}", "Aborted.".yellow());
                return Ok(());
            }
        }
        if !args.no_backup && target.is_file() {
            let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
            let bak = create_backup(&target, &label)?;
            println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
        }
    }

    if args.dry_run {
        println!("{} {} -> {}",
            "[DRY]".yellow(),
            args.src.display().to_string().cyan(),
            target.display().to_string().green()
        );
        return Ok(());
    }

    // Ensure parent exists
    if let Some(p) = target.parent() {
        if !p.as_os_str().is_empty() && !p.exists() {
            std::fs::create_dir_all(p)?;
        }
    }

    std::fs::rename(&args.src, &target)?;
    println!("{} {} -> {}",
        "Moved:".green().bold(),
        args.src.display().to_string().cyan(),
        target.display().to_string().green()
    );
    Ok(())
}
