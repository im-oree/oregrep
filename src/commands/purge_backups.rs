use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::engine::confirm::confirm;

#[derive(Args)]
pub struct PurgeBackupsArgs {
    /// Root path (default: current dir)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Only match this label suffix (e.g. "CAMFIX" -> *.bakCAMFIX)
    #[arg(long)]
    label: Option<String>,

    /// Only backups older than this many minutes
    #[arg(long)]
    older_than: Option<u64>,

    /// Only backups newer than this many minutes (useful for session-only)
    #[arg(long)]
    newer_than: Option<u64>,

    /// Restrict to files matching this substring
    #[arg(long)]
    matching: Option<String>,

    /// Include hidden dirs
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    no_ignore: bool,

    /// Dry run
    #[arg(long)]
    dry_run: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: PurgeBackupsArgs) -> Result<()> {
    if !args.path.exists() {
        anyhow::bail!("Path not found: {}", args.path.display());
    }

    let now = SystemTime::now();
    let mut walker = ignore::WalkBuilder::new(&args.path);
    walker.hidden(!args.hidden)
        .git_ignore(!args.no_ignore)
        .git_global(!args.no_ignore)
        .git_exclude(!args.no_ignore);

    let mut candidates: Vec<PathBuf> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in walker.build().flatten() {
        let path = entry.path();
        if !path.is_file() { continue; }
        let fname = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Must contain .bak
        let bak_pos = match fname.find(".bak") {
            Some(p) => p,
            None => continue,
        };
        // Label filter
        if let Some(lbl) = &args.label {
            let suffix = &fname[bak_pos + 4..];
            if suffix != lbl.as_str() {
                continue;
            }
        }
        // Matching filter (original filename before .bak)
        if let Some(m) = &args.matching {
            let base = &fname[..bak_pos];
            if !base.contains(m.as_str()) {
                continue;
            }
        }
        // Age filters
        let meta = match std::fs::metadata(path) { Ok(m) => m, Err(_) => continue };
        let modified = match meta.modified() { Ok(t) => t, Err(_) => continue };
        let age_secs = now.duration_since(modified).unwrap_or(Duration::ZERO).as_secs();
        if let Some(older) = args.older_than {
            if age_secs < older * 60 { continue; }
        }
        if let Some(newer) = args.newer_than {
            if age_secs > newer * 60 { continue; }
        }
        total_size += meta.len();
        candidates.push(path.to_path_buf());
    }

    if candidates.is_empty() {
        println!("{} No backup files matched.", "No-op:".yellow());
        return Ok(());
    }

    println!("{} {} backup files ({})",
        "Found:".cyan(),
        candidates.len().to_string().yellow(),
        format_size(total_size).green()
    );
    for p in &candidates {
        println!("  {}", p.display().to_string().dimmed());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing deleted]".yellow().bold());
        return Ok(());
    }

    let ok = confirm(&format!("Delete {} backup files?", candidates.len()), args.yes)?;
    if !ok {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    let mut deleted = 0;
    let mut errors = 0;
    for p in &candidates {
        match std::fs::remove_file(p) {
            Ok(_) => deleted += 1,
            Err(e) => {
                eprintln!("  {} {}: {}", "ERR".red(), p.display(), e);
                errors += 1;
            }
        }
    }
    println!("\n{} {} deleted, {} errors, {} freed",
        "Done:".green().bold(),
        deleted.to_string().green(),
        errors.to_string().red(),
        format_size(total_size).green()
    );
    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
