use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct CollapseBlankLinesArgs {
    file: PathBuf,

    /// Max consecutive blank lines to keep (default 1)
    #[arg(short = 'm', long, default_value = "1")]
    max: usize,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: CollapseBlankLinesArgs) -> Result<()> {
    let opts = EditOptions { no_backup: args.no_backup, label: args.label.clone(), dry_run: args.dry_run };
    let max = args.max;

    let result = edit_lines(&args.file, &opts, move |lines| {
        let mut out = Vec::with_capacity(lines.len());
        let mut blank_run = 0;
        for l in lines {
            if l.trim().is_empty() {
                blank_run += 1;
                if blank_run <= max {
                    out.push(l);
                }
            } else {
                blank_run = 0;
                out.push(l);
            }
        }
        Ok(out)
    })?;

    print_generic("Collapsed blanks", &args.file, &result, args.dry_run);
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
