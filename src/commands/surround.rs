use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, parse_line_range, EditOptions};

#[derive(Args)]
pub struct SurroundArgs {
    /// File to modify
    file: PathBuf,

    /// Range to surround: "10:20"
    range: String,

    /// Text to insert BEFORE the range. Use \n for multi-line.
    #[arg(short = 'B', long)]
    before: String,

    /// Text to insert AFTER the range. Use \n for multi-line.
    #[arg(short = 'A', long)]
    after: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: SurroundArgs) -> Result<()> {
    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };
    let before_text = args.before.replace("\\n", "\n");
    let after_text = args.after.replace("\\n", "\n");
    let before_lines: Vec<String> = if before_text.is_empty() {
        Vec::new()
    } else {
        before_text.split('\n').map(|s| s.to_string()).collect()
    };
    let after_lines: Vec<String> = if after_text.is_empty() {
        Vec::new()
    } else {
        after_text.split('\n').map(|s| s.to_string()).collect()
    };
    let range_str = args.range.clone();

    let result = edit_lines(&args.file, &opts, move |lines| {
        let (from, to) = parse_line_range(&range_str, lines.len())?;
        let mut out: Vec<String> = Vec::with_capacity(lines.len() + before_lines.len() + after_lines.len());
        for (i, l) in lines.into_iter().enumerate() {
            let lineno = i + 1;
            if lineno == from {
                for bl in &before_lines {
                    out.push(bl.clone());
                }
            }
            out.push(l);
            if lineno == to {
                for al in &after_lines {
                    out.push(al.clone());
                }
            }
        }
        Ok(out)
    })?;

    print_generic("Surrounded", &args.file, &result, args.dry_run);
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
