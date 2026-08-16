use anyhow::Result;
use clap::Args;
use colored::*;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

/// Search for multiple patterns at once with per-pattern counts.
/// Shows which patterns match each file and total match counts.
#[derive(Args)]
pub struct FindMultiArgs {
    /// Patterns to search for (any number, space-separated)
    #[arg(required = true)]
    patterns: Vec<String>,

    /// Path to search (last arg if it looks like a directory)
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// Extension filter
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Exclude directories
    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Case-insensitive
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Treat patterns as literals
    #[arg(short = 'F', long)]
    literal: bool,

    /// Show counts per pattern (default: on)
    #[arg(short = 'c', long, default_value = "true")]
    count: bool,

    /// Files-only output
    #[arg(short = 'l', long)]
    files_only: bool,

    /// Verbose: show which patterns matched each file
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Include hidden files
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    no_ignore: bool,
}

pub fn run(args: FindMultiArgs) -> Result<()> {
    if args.patterns.is_empty() {
        anyhow::bail!("Provide at least one pattern");
    }

    // Handle "path as last positional" pattern: if the last "pattern" looks
    // like a directory, treat it as the path and remove it from patterns.
    let mut patterns = args.patterns.clone();
    let mut path = args.path.clone();
    if patterns.len() > 1 {
        let last = patterns.last().unwrap().clone();
        let p = PathBuf::from(&last);
        if p.is_dir() {
            path = p;
            patterns.pop();
        }
    }

    let regexes: Vec<regex::Regex> = patterns
        .iter()
        .map(|p| {
            let pat = if args.literal { regex::escape(p) } else { p.clone() };
            RegexBuilder::new(&pat)
                .case_insensitive(args.ignore_case)
                .build()
        })
        .collect::<Result<Vec<_>, _>>()?;

    let cfg = WalkConfig {
        root: path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        hidden: args.hidden,
        respect_gitignore: !args.no_ignore,
        include_binary: false,
        skip_backups: true,
    };
    let files = collect_files(&cfg)?;

    let mut per_pattern_count: Vec<usize> = vec![0; patterns.len()];
    let mut files_matched = 0usize;

    for f in &files {
        let content = match read_file_smart(f) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let counts: Vec<usize> = regexes
            .iter()
            .map(|r| r.find_iter(&content).count())
            .collect();

        let matched_any = counts.iter().any(|&c| c > 0);
        if !matched_any {
            continue;
        }

        files_matched += 1;
        for (i, c) in counts.iter().enumerate() {
            per_pattern_count[i] += c;
        }

        if args.files_only {
            println!("{}", f.display());
            continue;
        }

        if args.verbose {
            let details: Vec<String> = counts
                .iter()
                .enumerate()
                .filter(|(_, &c)| c > 0)
                .map(|(i, c)| format!("{}={}", patterns[i], c))
                .collect();
            println!(
                "{}  {}",
                f.display().to_string().cyan(),
                details.join(" ").dimmed()
            );
        } else if args.count {
            let total: usize = counts.iter().sum();
            println!(
                "{}: {}",
                f.display().to_string().cyan(),
                total.to_string().yellow()
            );
        } else {
            println!("{}", f.display().to_string().cyan());
        }
    }

    // Summary
    eprintln!();
    eprintln!(
        "{} {} files matched at least one pattern",
        "Total:".bold(),
        files_matched.to_string().yellow()
    );
    for (i, c) in per_pattern_count.iter().enumerate() {
        eprintln!(
            "  {} {} matches",
            patterns[i].cyan(),
            c.to_string().yellow()
        );
    }

    Ok(())
}
