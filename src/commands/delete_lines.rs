use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, parse_line_range, EditOptions};

#[derive(Args)]
pub struct DeleteLinesArgs {
    /// File to modify
    file: PathBuf,

    /// Line or range: "42", "10:20", "10-20"
    range: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: DeleteLinesArgs) -> Result<()> {
    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };
    let range_str = args.range.clone();
    let result = edit_lines(&args.file, &opts, move |lines| {
        let (from, to) = parse_line_range(&range_str, lines.len())?;
        let mut out = Vec::with_capacity(lines.len() - (to - from + 1));
        for (i, l) in lines.into_iter().enumerate() {
            let lineno = i + 1;
            if lineno < from || lineno > to {
                out.push(l);
            }
        }
        Ok(out)
    })?;
    print_generic("Deleted", &args.file, &result, args.dry_run);
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
