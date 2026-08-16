use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, parse_line_range, EditOptions};
use crate::engine::patch::{read_for_patch, unescape_arg};

#[derive(Args)]
pub struct PatchLinesArgs {
    /// File to patch
    file: PathBuf,

    /// Line number or inclusive range: N, N:M, or N-M
    range: String,

    /// Replacement text. Use \n for multi-line content. Omit or pass empty to delete the range.
    #[arg(default_value = "")]
    text: String,

    /// Skip creating a backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label (default: timestamp)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run: show what would change, don't write
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: PatchLinesArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let (content, _, _) = read_for_patch(&args.file)?;
    let total_lines = content.lines().count();
    let (start, end) = parse_line_range(&args.range, total_lines)?;

    let replacement_unescaped = unescape_arg(&args.text);
    let replacement_normalized = replacement_unescaped
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    let replacement_lines: Vec<String> = if replacement_normalized.is_empty() {
        Vec::new()
    } else {
        replacement_normalized
            .split('\n')
            .map(|s| s.to_string())
            .collect()
    };

    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };

    let replacement_lines_for_edit = replacement_lines.clone();
    let result = edit_lines(&args.file, &opts, move |mut lines| {
        let start_idx = start - 1;
        let end_idx = end; // inclusive 1-indexed -> exclusive vec range
        lines.splice(start_idx..end_idx, replacement_lines_for_edit.clone());
        Ok(lines)
    })?;

    if args.dry_run {
        println!(
            "{} {}",
            "[DRY RUN]".yellow().bold(),
            args.file.display().to_string().cyan()
        );
    }

    if let Some(ref backup_path) = result.backup_path {
        println!(
            "{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    let removed = end - start + 1;
    let added = replacement_lines.len();

    println!(
        "{} {} lines {}-{} ({} removed, {} added)",
        if args.dry_run { "Would patch:".yellow().bold() } else { "Patched:".green().bold() },
        args.file.display().to_string().cyan(),
        start.to_string().yellow(),
        end.to_string().yellow(),
        removed.to_string().yellow(),
        added.to_string().green()
    );

    println!(
        "{} {} -> {}",
        "Line count:".dimmed(),
        result.lines_before.to_string().dimmed(),
        result.lines_after.to_string().dimmed()
    );

    Ok(())
}
