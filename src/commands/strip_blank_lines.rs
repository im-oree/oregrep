use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct StripBlankLinesArgs {
    file: PathBuf,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: StripBlankLinesArgs) -> Result<()> {
    let opts = EditOptions { no_backup: args.no_backup, label: args.label.clone(), dry_run: args.dry_run };
    let result = edit_lines(&args.file, &opts, |lines| {
        Ok(lines.into_iter().filter(|l| !l.trim().is_empty()).collect())
    })?;
    print_generic("Stripped blanks", &args.file, &result, args.dry_run);
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
