use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct DedupLinesArgs {
    file: PathBuf,

    /// Only remove ADJACENT duplicates (like uniq)
    #[arg(short = 'a', long)]
    adjacent: bool,

    /// Ignore case when comparing
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Ignore leading/trailing whitespace when comparing
    #[arg(short = 't', long)]
    trim: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: DedupLinesArgs) -> Result<()> {
    let opts = EditOptions { no_backup: args.no_backup, label: args.label.clone(), dry_run: args.dry_run };
    let adjacent = args.adjacent;
    let ic = args.ignore_case;
    let trim_ws = args.trim;

    let result = edit_lines(&args.file, &opts, move |lines| {
        let key = |s: &str| -> String {
            let s = if trim_ws { s.trim().to_string() } else { s.to_string() };
            if ic { s.to_lowercase() } else { s }
        };
        let mut out = Vec::with_capacity(lines.len());
        if adjacent {
            let mut prev: Option<String> = None;
            for l in lines {
                let k = key(&l);
                if prev.as_ref() != Some(&k) {
                    out.push(l);
                    prev = Some(k);
                }
            }
        } else {
            let mut seen: HashSet<String> = HashSet::new();
            for l in lines {
                let k = key(&l);
                if seen.insert(k) {
                    out.push(l);
                }
            }
        }
        Ok(out)
    })?;

    print_generic("Deduplicated", &args.file, &result, args.dry_run);
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
