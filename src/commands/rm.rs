use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::confirm::confirm;

#[derive(Args)]
pub struct RmArgs {
    /// Files or directories to delete
    paths: Vec<PathBuf>,

    /// Recursive (required for directories)
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,

    /// Force (ignore missing, no backup)
    #[arg(short = 'f', long)]
    force: bool,

    /// Skip backup before delete
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: RmArgs) -> Result<()> {
    if args.paths.is_empty() {
        anyhow::bail!("At least one path required");
    }

    // Validate first
    let mut targets: Vec<PathBuf> = Vec::new();
    for p in &args.paths {
        if !p.exists() {
            if args.force {
                continue;
            }
            anyhow::bail!("Not found: {}", p.display());
        }
        if p.is_dir() && !args.recursive {
            anyhow::bail!("{} is a directory. Use -r to remove recursively.", p.display());
        }
        targets.push(p.clone());
    }

    if targets.is_empty() {
        println!("{}", "Nothing to delete.".yellow());
        return Ok(());
    }

    println!("{}", "Will delete:".cyan());
    for t in &targets {
        let mark = if t.is_dir() { "DIR " } else { "FILE" };
        println!("  {} {}", mark.dimmed(), t.display().to_string().yellow());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing deleted]".yellow().bold());
        return Ok(());
    }

    let ok = confirm(&format!("Delete {} item(s)?", targets.len()), args.yes)?;
    if !ok {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    let mut deleted = 0usize;
    let mut errors = 0usize;
    for t in &targets {
        // Backup files (not dirs)
        if !args.no_backup && !args.force && t.is_file() {
            let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
            if let Err(e) = create_backup(t, &label) {
                eprintln!("  {} backup failed for {}: {}", "WARN".yellow(), t.display(), e);
            }
        }
        let res = if t.is_dir() {
            std::fs::remove_dir_all(t)
        } else {
            std::fs::remove_file(t)
        };
        match res {
            Ok(_) => {
                deleted += 1;
                println!("  {} {}", "OK".green(), t.display().to_string().dimmed());
            }
            Err(e) => {
                errors += 1;
                eprintln!("  {} {}: {}", "ERR".red(), t.display(), e);
            }
        }
    }
    println!("\n{} {} deleted, {} errors",
        "Done:".green().bold(),
        deleted.to_string().green(),
        errors.to_string().red()
    );
    Ok(())
}
