use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct SortLinesArgs {
    file: PathBuf,

    /// Reverse (descending)
    #[arg(short = 'r', long)]
    reverse: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Numeric sort
    #[arg(short = 'n', long)]
    numeric: bool,

    /// Unique (dedupe after sorting)
    #[arg(short = 'u', long)]
    unique: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: SortLinesArgs) -> Result<()> {
    let opts = EditOptions { no_backup: args.no_backup, label: args.label.clone(), dry_run: args.dry_run };
    let rev = args.reverse;
    let ic = args.ignore_case;
    let num = args.numeric;
    let uniq = args.unique;

    let result = edit_lines(&args.file, &opts, move |mut lines| {
        if num {
            lines.sort_by(|a, b| {
                let na: f64 = a.trim().parse().unwrap_or(f64::MAX);
                let nb: f64 = b.trim().parse().unwrap_or(f64::MAX);
                na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
            });
        } else if ic {
            lines.sort_by_key(|s| s.to_lowercase());
        } else {
            lines.sort();
        }
        if rev { lines.reverse(); }
        if uniq {
            lines.dedup();
        }
        Ok(lines)
    })?;

    print_generic("Sorted", &args.file, &result, args.dry_run);
    Ok(())
}

fn print_generic(action: &str, file: &std::path::Path, r: &crate::engine::edit::EditResult, dry: bool) {
    let tag = if dry { "[DRY RUN]".yellow().bold().to_string() } else { format!("{}", action.green().bold()) };
    println!("{} {} ({} -> {} lines)",
        tag, file.display().to_string().cyan(),
        r.lines_before.to_string().yellow(), r.lines_after.to_string().yellow());
    if let Some(b) = &r.backup_path {
        println!("  {} {}", "Backup:".dimmed(), b.display().to_string().dimmed());
    }
}
