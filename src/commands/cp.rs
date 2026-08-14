use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::{Path, PathBuf};

use crate::engine::backup::create_backup;
use crate::engine::confirm::confirm;

#[derive(Args)]
pub struct CpArgs {
    /// Source file or directory
    src: PathBuf,

    /// Destination path
    dst: PathBuf,

    /// Recursive (for directories)
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,

    /// Force overwrite
    #[arg(long)]
    force: bool,

    /// Skip backup on overwrite
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: CpArgs) -> Result<()> {
    if !args.src.exists() {
        anyhow::bail!("Source not found: {}", args.src.display());
    }

    let target = if args.dst.is_dir() {
        let fname = args.src.file_name().ok_or_else(|| anyhow::anyhow!("Invalid src filename"))?;
        args.dst.join(fname)
    } else {
        args.dst.clone()
    };

    if args.src.is_dir() && !args.recursive {
        anyhow::bail!("Source is a directory. Use -r to copy recursively.");
    }

    if target.exists() && !args.src.is_dir() {
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

    if let Some(p) = target.parent() {
        if !p.as_os_str().is_empty() && !p.exists() {
            std::fs::create_dir_all(p)?;
        }
    }

    if args.src.is_dir() {
        let (count, size) = copy_dir_recursive(&args.src, &target)?;
        println!("{} {} -> {}  ({} files, {})",
            "Copied dir:".green().bold(),
            args.src.display().to_string().cyan(),
            target.display().to_string().green(),
            count.to_string().yellow(),
            format_size(size).yellow()
        );
    } else {
        std::fs::copy(&args.src, &target)?;
        println!("{} {} -> {}",
            "Copied:".green().bold(),
            args.src.display().to_string().cyan(),
            target.display().to_string().green()
        );
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(usize, u64)> {
    std::fs::create_dir_all(dst)?;
    let mut count = 0usize;
    let mut total = 0u64;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if path.is_dir() {
            let (c, s) = copy_dir_recursive(&path, &dst_path)?;
            count += c;
            total += s;
        } else {
            let sz = std::fs::copy(&path, &dst_path)?;
            count += 1;
            total += sz;
        }
    }
    Ok((count, total))
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
