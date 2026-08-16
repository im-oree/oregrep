use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};
use crate::engine::patch::unescape_arg;

#[derive(Args)]
pub struct PatchInsertArgs {
    /// File to modify
    file: PathBuf,

    /// Line number to insert relative to (1-indexed; 0 = prepend to file)
    line: usize,

    /// Text to insert. Use \n for multi-line content.
    #[arg(default_value = "")]
    text: String,

    /// Insert before the specified line (default: after)
    #[arg(long, conflicts_with = "after")]
    before: bool,

    /// Insert after the specified line (this is the default)
    #[arg(long, conflicts_with = "before")]
    after: bool,

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

pub fn run(args: PatchInsertArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let insert_before = args.before; // if neither --before nor --after, default is after

    // Unescape \n \t \\ in the text
    let text_unescaped = unescape_arg(&args.text);
    let text_normalized = text_unescaped
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    let new_lines: Vec<String> = if text_normalized.is_empty() {
        Vec::new()
    } else {
        text_normalized
            .split('\n')
            .map(|s| s.to_string())
            .collect()
    };

    let line_num = args.line;
    let added = new_lines.len();

    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };

    let new_lines_clone = new_lines.clone();
    let result = edit_lines(&args.file, &opts, move |mut lines| {
        let total = lines.len();

        // Determine insertion index
        let insert_idx = if line_num == 0 {
            // --after 0 or --before 0: both mean prepend
            0
        } else if insert_before {
            // Insert before line N → index N-1
            if line_num > total {
                anyhow::bail!(
                    "Line {} out of range (file has {} lines)",
                    line_num,
                    total
                );
            }
            line_num - 1
        } else {
            // Insert after line N → index N
            if line_num > total {
                anyhow::bail!(
                    "Line {} out of range (file has {} lines)",
                    line_num,
                    total
                );
            }
            line_num
        };

        // splice with empty range = pure insert
        lines.splice(insert_idx..insert_idx, new_lines_clone.clone());
        Ok(lines)
    })?;

    let position_label = if line_num == 0 {
        "beginning of file".to_string()
    } else if insert_before {
        format!("before line {}", line_num)
    } else {
        format!("after line {}", line_num)
    };

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

    println!(
        "{} {} — {} line{} inserted {}",
        if args.dry_run {
            "Would insert:".yellow().bold()
        } else {
            "Inserted:".green().bold()
        },
        args.file.display().to_string().cyan(),
        added.to_string().yellow(),
        if added == 1 { "" } else { "s" },
        position_label.dimmed()
    );

    println!(
        "{} {} -> {}",
        "Line count:".dimmed(),
        result.lines_before.to_string().dimmed(),
        result.lines_after.to_string().dimmed()
    );

    Ok(())
}
