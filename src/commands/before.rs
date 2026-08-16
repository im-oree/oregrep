use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::edit::{edit_lines, EditOptions};

#[derive(Args)]
pub struct BeforeArgs {
    /// File to modify
    file: PathBuf,

    /// Pattern to match (regex)
    pattern: String,

    /// Text to insert before matching line(s). Use \n for multi-line.
    #[arg(default_value = "")]
    text: String,

    /// Match only the first occurrence
    #[arg(long)]
    first: bool,

    /// Treat pattern as literal
    #[arg(short = 'F', long)]
    literal: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: BeforeArgs) -> Result<()> {
    let opts = EditOptions {
        no_backup: args.no_backup,
        label: args.label.clone(),
        dry_run: args.dry_run,
    };
    let mut pattern = if args.literal { regex::escape(&args.pattern) } else { args.pattern.clone() };
    if args.ignore_case { pattern = format!("(?i){}", pattern); }
    let re = Regex::new(&pattern)?;
    let insert_text = args.text.replace("\\n", "\n");
    let insert_lines: Vec<String> = if insert_text.is_empty() {
        Vec::new()
    } else {
        insert_text.split('\n').map(|s| s.to_string()).collect()
    };
    let first_only = args.first;

    let result = edit_lines(&args.file, &opts, move |lines| {
        let mut out: Vec<String> = Vec::with_capacity(lines.len() + insert_lines.len());
        let mut done = false;
        for l in lines {
            if !done && re.is_match(&l) {
                for il in &insert_lines {
                    out.push(il.clone());
                }
                if first_only {
                    done = true;
                }
            }
            out.push(l);
        }
        Ok(out)
    })?;

    print_generic("Inserted before", &args.file, &result, args.dry_run);
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
