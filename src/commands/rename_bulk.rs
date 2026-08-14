use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::walker::{collect_files, parse_excludes, WalkConfig};

#[derive(Args)]
pub struct RenameBulkArgs {
    /// Regex pattern applied to filenames
    pattern: String,

    /// Replacement (supports $1, $2 capture groups)
    replacement: String,

    /// Root path
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Recurse into subdirectories
    #[arg(short = 'R', long)]
    recursive: bool,

    /// Extensions to include
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Excludes
    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Include hidden
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Don't respect gitignore
    #[arg(long)]
    no_ignore: bool,

    /// Dry run
    #[arg(long)]
    dry_run: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Match against full path, not just filename
    #[arg(long)]
    full_path: bool,
}

pub fn run(args: RenameBulkArgs) -> Result<()> {
    let mut pattern = args.pattern.clone();
    if args.ignore_case {
        pattern = format!("(?i){}", pattern);
    }
    let re = Regex::new(&pattern)?;

    let mut cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(crate::engine::walker::parse_extensions).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: true,
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: false,
    };
    if !args.recursive {
        // Non-recursive: only immediate children by post-filter
    }

    let all_files = collect_files(&cfg)?;
    let candidates: Vec<PathBuf> = if args.recursive {
        all_files
    } else {
        all_files
            .into_iter()
            .filter(|p| p.parent() == Some(args.path.as_path()))
            .collect()
    };
    // suppress unused warning
    let _ = &mut cfg;

    let mut renamed = 0;
    let mut skipped = 0;
    for path in &candidates {
        let target = if args.full_path {
            path.to_string_lossy().to_string()
        } else {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        if !re.is_match(&target) {
            continue;
        }
        let new_name = re.replace_all(&target, args.replacement.as_str()).into_owned();
        if new_name == target {
            skipped += 1;
            continue;
        }

        let new_path = if args.full_path {
            PathBuf::from(new_name)
        } else {
            path.parent().map(|p| p.join(&new_name)).unwrap_or_else(|| PathBuf::from(&new_name))
        };

        if args.dry_run {
            println!("  {} {} -> {}",
                "[DRY]".yellow(),
                path.display().to_string().cyan(),
                new_path.display().to_string().green()
            );
            continue;
        }

        if new_path.exists() {
            eprintln!("  {} target already exists: {}", "SKIP".yellow(), new_path.display());
            skipped += 1;
            continue;
        }

        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::rename(path, &new_path)?;
        renamed += 1;
        println!("  {} {} -> {}",
            "OK".green(),
            path.display().to_string().cyan(),
            new_path.display().to_string().green()
        );
    }

    println!("\n{}", "Summary:".bold());
    println!("  Renamed: {}", renamed.to_string().green());
    if skipped > 0 {
        println!("  Skipped: {}", skipped.to_string().yellow());
    }
    if args.dry_run {
        println!("  {}", "[DRY RUN — nothing was renamed]".yellow().bold());
    }
    Ok(())
}
