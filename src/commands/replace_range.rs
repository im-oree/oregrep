use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, parse_line_range, EditOptions};

#[derive(Args)]
pub struct ReplaceRangeArgs {
    /// File to modify
    file: PathBuf,

    /// Range to replace: "42", "10:20", "10-20"
    range: String,

    /// Replacement text. Use \n for multiple lines. Empty = delete range.
    #[arg(default_value = "")]
    text: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: ReplaceRangeArgs) -> Result<()> {
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
    let range_str = args.range.clone();
    let is_delete = new_lines.is_empty();

    let result = edit_lines(&args.file, &opts, move |lines| {
        let (from, to) = parse_line_range(&range_str, lines.len())?;
        let cap = lines.len().saturating_sub(to - from + 1) + new_lines.len();
        let mut out: Vec<String> = Vec::with_capacity(cap);
        for (i, l) in lines.into_iter().enumerate() {
            let lineno = i + 1;
            if lineno < from {
                out.push(l);
            } else if lineno == from {
                for nl in &new_lines {
                    out.push(nl.clone());
                }
            } else if lineno > to {
                out.push(l);
            }
            // lines in (from..=to) are skipped
        }
        Ok(out)
    })?;

    let action = if is_delete { "Deleted range" } else { "Replaced range" };
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
