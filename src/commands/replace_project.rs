use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{read_for_patch, write_atomic};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct ReplaceProjectArgs {
    /// Regex pattern to find
    pub pattern: String,

    /// Replacement string (supports $1, $2 capture groups)
    pub replacement: String,

    /// Root path (default: current dir)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Extensions to include (comma-separated, e.g. "ts,tsx,rs")
    #[arg(short = 'e', long)]
    pub ext: Option<String>,

    /// Exclude substrings (comma-separated, e.g. "test,mock")
    #[arg(short = 'x', long)]
    pub exclude: Option<String>,

    /// Treat pattern as literal
    #[arg(short = 'F', long)]
    pub literal: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Whole word only
    #[arg(short = 'w', long)]
    pub word: bool,

    /// Multi-line mode (^ and $ match line boundaries)
    #[arg(short = 'm', long)]
    pub multiline: bool,

    /// Include hidden files
    #[arg(short = 'H', long)]
    pub hidden: bool,

    /// Do NOT respect .gitignore
    #[arg(long)]
    pub no_ignore: bool,

    /// Include binary files
    #[arg(long)]
    pub binary: bool,

    /// Dry run — show matches per file, don't write
    #[arg(long)]
    pub dry_run: bool,

    /// Skip backups (dangerous)
    #[arg(long)]
    pub no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    pub label: Option<String>,

    /// Continue on error (don't stop on first failure)
    #[arg(long)]
    pub keep_going: bool,
}

pub fn run(args: ReplaceProjectArgs) -> Result<()> {
    let mut pattern = if args.literal {
        regex::escape(&args.pattern)
    } else {
        args.pattern.clone()
    };
    if args.word {
        pattern = format!(r"\b{}\b", pattern);
    }
    let re = RegexBuilder::new(&pattern)
        .case_insensitive(args.ignore_case)
        .multi_line(args.multiline)
        .build()?;

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
    let mut errors = 0;

    for file in &files {
        let (content, had_bom, _) = match read_for_patch(file) {
            Ok(x) => x,
            Err(e) => {
                if args.keep_going {
                    eprintln!("  {} {}: {}", "SKIP".yellow(), file.display(), e);
                    continue;
                }
                return Err(e);
            }
        };

        let count = re.find_iter(&content).count();
        if count == 0 {
            continue;
        }
        files_matched += 1;

        let new_content = re.replace_all(&content, args.replacement.as_str()).into_owned();
        total_replacements += count;

        if args.dry_run {
            println!("  {} {}  ({} matches)",
                "[DRY]".yellow(),
                file.display().to_string().cyan(),
                count.to_string().yellow()
            );
            continue;
        }

        if !args.no_backup {
            if let Err(e) = create_backup(file, &label) {
                errors += 1;
                eprintln!("  {} backup failed for {}: {}", "ERR".red(), file.display(), e);
                if !args.keep_going {
                    return Err(e);
                }
                continue;
            }
        }

        match write_atomic(file, &new_content, had_bom) {
            Ok(_) => {
                files_changed += 1;
                println!("  {} {}  ({} replacements)",
                    "OK".green(),
                    file.display().to_string().cyan(),
                    count.to_string().yellow()
                );
            }
            Err(e) => {
                errors += 1;
                eprintln!("  {} write failed for {}: {}", "ERR".red(), file.display(), e);
                if !args.keep_going {
                    return Err(e);
                }
            }
        }
    }

    println!("\n{}", "Summary:".bold());
    println!("  Files scanned:  {}", files.len().to_string().yellow());
    println!("  Files matched:  {}", files_matched.to_string().yellow());
    println!("  Files changed:  {}", files_changed.to_string().green());
    println!("  Total replacements: {}", total_replacements.to_string().green());
    if errors > 0 {
        println!("  Errors: {}", errors.to_string().red());
    }
    if args.dry_run {
        println!("  {}", "[DRY RUN — nothing was written]".yellow().bold());
    }

    Ok(())
}
