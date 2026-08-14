use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct TrimArgs {
    file: PathBuf,

    /// Trim only trailing whitespace on each line (default)
    #[arg(short = 't', long)]
    trailing: bool,

    /// Trim only leading whitespace on each line
    #[arg(short = 'L', long)]
    leading: bool,

    /// Trim both leading and trailing
    #[arg(short = 'b', long)]
    both: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: TrimArgs) -> Result<()> {
    let opts = EditOptions { no_backup: args.no_backup, label: args.label.clone(), dry_run: args.dry_run };
    let leading = args.leading || args.both;
    let trailing = args.trailing || args.both || (!args.leading && !args.both);

    let result = edit_lines(&args.file, &opts, move |lines| {
        Ok(lines.into_iter().map(|l| {
            let mut s = l.as_str();
            if leading { s = s.trim_start(); }
            if trailing { s = s.trim_end(); }
            s.to_string()
        }).collect())
    })?;

    print_generic("Trimmed", &args.file, &result, args.dry_run);
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
