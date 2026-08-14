use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{apply_patch, read_for_patch, write_atomic, PatchMode};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct PatchProjectArgs {
    /// Literal text to find
    #[arg(short = 'f', long)]
    find: String,

    /// Replacement text
    #[arg(short = 'r', long)]
    replace: String,

    /// Root path
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    #[arg(short = 'H', long)]
    hidden: bool,
    #[arg(long)]
    no_ignore: bool,
    #[arg(long)]
    binary: bool,

    /// Replace all occurrences per file (default)
    #[arg(short = 'a', long, default_value = "true")]
    all: bool,

    /// Only replace files where exactly one match is found (safer)
    #[arg(long, conflicts_with = "all")]
    exact_one: bool,

    #[arg(long)]
    dry_run: bool,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    keep_going: bool,
}

pub fn run(args: PatchProjectArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: args.binary,
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
    };

    let files = collect_files(&cfg)?;
    println!("{} {} files to scan", "Scanning:".cyan(), files.len().to_string().yellow());

    let label = args
        .label
        .clone()
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());

    let mut files_matched = 0;
    let mut files_changed = 0;
    let mut total_replacements = 0;

    let mode = if args.exact_one { PatchMode::Once } else { PatchMode::All };

    for file in &files {
        let (content, had_bom, newline) = match read_for_patch(file) {
            Ok(x) => x,
            Err(e) => {
                if args.keep_going {
                    eprintln!("  {} {}: {}", "SKIP".yellow(), file.display(), e);
                    continue;
                }
                return Err(e);
            }
        };

        let find_norm = args.find.replace("\r\n", "\n").replace('\n', newline);
        let replace_norm = args.replace.replace("\r\n", "\n").replace('\n', newline);

        if !content.contains(&find_norm) {
            continue;
        }
        files_matched += 1;

        let (new_content, result) = match apply_patch(&content, &find_norm, &replace_norm, mode) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("  {} {}: {}", "SKIP".yellow(), file.display(), e);
                if !args.keep_going {
                    return Err(e);
                }
                continue;
            }
        };

        total_replacements += result.replacements_made;

        if args.dry_run {
            println!("  {} {}  ({} matches, {} would be replaced)",
                "[DRY]".yellow(),
                file.display().to_string().cyan(),
                result.matches_found.to_string().yellow(),
                result.replacements_made.to_string().green()
            );
            continue;
        }

        if !args.no_backup {
            create_backup(file, &label)?;
        }
        write_atomic(file, &new_content, had_bom)?;
        files_changed += 1;
        println!("  {} {}  ({} replacements)",
            "OK".green(),
            file.display().to_string().cyan(),
            result.replacements_made.to_string().yellow()
        );
    }

    println!("\n{}", "Summary:".bold());
    println!("  Files scanned:  {}", files.len().to_string().yellow());
    println!("  Files matched:  {}", files_matched.to_string().yellow());
    println!("  Files changed:  {}", files_changed.to_string().green());
    println!("  Total replacements: {}", total_replacements.to_string().green());
    if args.dry_run {
        println!("  {}", "[DRY RUN — nothing was written]".yellow().bold());
    }
    Ok(())
}
