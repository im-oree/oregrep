use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct InsertArgs {
    /// File to modify
    file: PathBuf,

    /// Line number to insert AT (1-indexed). Existing line N shifts down.
    /// Use 0 to insert at start, or a number greater than line count to append.
    line: usize,

    /// Text to insert. Use \n for multiple lines.
    text: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: InsertArgs) -> Result<()> {
    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };

    // Handle literal \n in argument -> real newlines
    let inserted: Vec<String> = args
        .text
        .replace("\\n", "\n")
        .split('\n')
        .map(|s| s.to_string())
        .collect();

    let target_line = args.line;
    let result = edit_lines(&args.file, &opts, move |mut lines| {
        let total = lines.len();
        let idx = if target_line == 0 {
            0
        } else if target_line > total {
            total
        } else {
            target_line - 1
        };
        for (i, l) in inserted.into_iter().enumerate() {
            lines.insert(idx + i, l);
        }
        Ok(lines)
    })?;

    print_result("Inserted", &args.file, &result, args.dry_run);
    Ok(())
}

fn print_result(action: &str, file: &std::path::Path, r: &crate::engine::edit::EditResult, dry: bool) {
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
