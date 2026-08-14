use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{read_for_patch, write_atomic};

#[derive(Args)]
pub struct ReplaceArgs {
    /// Regex pattern to find
    pattern: String,

    /// Replacement string (supports $1, $2 capture groups)
    replacement: String,

    /// File to modify
    file: PathBuf,

    /// Treat pattern as literal string, not regex
    #[arg(short = 'F', long)]
    literal: bool,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Whole word match only
    #[arg(short = 'w', long)]
    word: bool,

    /// Multi-line mode (^ and $ match line boundaries)
    #[arg(short = 'm', long)]
    multiline: bool,

    /// Skip backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run: show matches, don't write
    #[arg(long)]
    dry_run: bool,

    /// Replace only the first N matches (0 = all)
    #[arg(short = 'n', long, default_value = "0")]
    max: usize,
}

pub fn run(args: ReplaceArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

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
        .build()
        .with_context(|| format!("Invalid regex: {}", pattern))?;

    let (content, had_bom, _newline) = read_for_patch(&args.file)?;

    let match_count = re.find_iter(&content).count();
    if match_count == 0 {
        println!("{} No matches for pattern in {}",
            "!".yellow(),
            args.file.display().to_string().cyan()
        );
        return Ok(());
    }

    let new_content = if args.max == 0 {
        re.replace_all(&content, args.replacement.as_str()).into_owned()
    } else {
        re.replacen(&content, args.max, args.replacement.as_str()).into_owned()
    };

    let replaced = if args.max == 0 {
        match_count
    } else {
        match_count.min(args.max)
    };

    if args.dry_run {
        println!("{} {}",
            "[DRY RUN]".yellow().bold(),
            args.file.display().to_string().cyan()
        );
        println!("  {} matches, {} would be replaced",
            match_count.to_string().yellow(),
            replaced.to_string().green()
        );
        return Ok(());
    }

    if !args.no_backup {
        let label = args
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let backup_path = create_backup(&args.file, &label)?;
        println!("{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    write_atomic(&args.file, &new_content, had_bom)?;

    println!("{} {} ({} replacement{})",
        "Replaced:".green().bold(),
        args.file.display().to_string().cyan(),
        replaced.to_string().yellow(),
        if replaced == 1 { "" } else { "s" }
    );

    Ok(())
}
