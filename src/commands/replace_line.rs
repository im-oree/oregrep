use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct ReplaceLineArgs {
    /// File to modify
    file: PathBuf,

    /// Line number to replace (1-indexed)
    line: usize,

    /// New content for that line. Use \n for multi-line replacement. Empty = delete line.
    #[arg(default_value = "")]
    text: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: ReplaceLineArgs) -> Result<()> {
    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };
    let text = args.text.replace("\\n", "\n");
    let new_lines: Vec<String> = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').map(|s| s.to_string()).collect()
    };
    let target = args.line;
    let is_delete = new_lines.is_empty();

    let result = edit_lines(&args.file, &opts, move |mut lines| {
        let total = lines.len();
        if target == 0 || target > total {
            anyhow::bail!("Line {} out of range (file has {} lines)", target, total);
        }
        let idx = target - 1;
        lines.remove(idx);
        for (i, l) in new_lines.into_iter().enumerate() {
            lines.insert(idx + i, l);
        }
        Ok(lines)
    })?;
    let action = if is_delete { "Deleted line" } else { "Replaced line" };
    print_generic(action, &args.file, &result, args.dry_run);
    Ok(())
}

fn print_generic(action: &str, file: &std::path::Path, r: &crate::engine::edit::EditResult, dry: bool) {
    let tag = if dry { "[DRY RUN]".yellow().bold().to_string() } else { format!("{}", action.green().bold()) };
    println!("{} {} ({} -> {} lines)",
        tag,
        file.display().to_string().cyan(),
        r.lines_before.to_string().yellow(),
        r.lines_after.to_string().yellow()
    );
    if let Some(b) = &r.backup_path {
        println!("  {} {}", "Backup:".dimmed(), b.display().to_string().dimmed());
    }
}
