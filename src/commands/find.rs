use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use std::path::PathBuf;

use crate::engine::encoding::{is_binary, read_file_smart};
use crate::engine::text::enumerate_lines;

#[derive(Args)]
pub struct FindArgs {
    /// Pattern to search for (regex by default)
    pattern: String,

    /// Path to search (file or directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Treat pattern as literal string, not regex
    #[arg(short = 'F', long)]
    literal: bool,

    /// Case-insensitive search
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Whole word match only
    #[arg(short = 'w', long)]
    word: bool,

    /// Include hidden files
    #[arg(short = 'H', long)]
    hidden: bool,

    /// Don't respect .gitignore
    #[arg(long)]
    no_ignore: bool,

    /// Search binary files too
    #[arg(long)]
    binary: bool,

    /// Show only file names with matches
    #[arg(short = 'l', long)]
    files_only: bool,

    /// Show only match count per file
    #[arg(short = 'c', long)]
    count_only: bool,

    /// Show N lines before match
    #[arg(short = 'B', long, default_value = "0")]
    before: usize,

    /// Show N lines after match
    #[arg(short = 'A', long, default_value = "0")]
    after: usize,

    /// File extension filter (e.g. "ts,tsx,rs")
    #[arg(short = 'e', long)]
    ext: Option<String>,
}

pub fn run(args: FindArgs) -> Result<()> {
    // Build regex
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
        .multi_line(true)
        .build()
        .with_context(|| format!("Invalid regex: {}", pattern))?;

    // Extension filter
    let ext_filter: Option<Vec<String>> = args.ext.as_ref().map(|s| {
        s.split(',')
            .map(|e| e.trim().trim_start_matches('.').to_lowercase())
            .collect()
    });

    // Walker
    let walker = WalkBuilder::new(&args.path)
        .hidden(!args.hidden)
        .git_ignore(!args.no_ignore)
        .git_global(!args.no_ignore)
        .git_exclude(!args.no_ignore)
        .build();

    let mut total_matches: usize = 0;
    let mut files_with_matches: usize = 0;

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Extension filter
        if let Some(filters) = &ext_filter {
            let matches_ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| filters.iter().any(|f| f == &e.to_lowercase()))
                .unwrap_or(false);
            if !matches_ext {
                continue;
            }
        }

        // Skip binary unless requested
        if !args.binary {
            match is_binary(path) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(_) => continue,
            }
        }

        // Read + decode file
        let content = match read_file_smart(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Find matches
        let lines: Vec<(usize, &str)> = enumerate_lines(&content).collect();
        let mut matched_lines: Vec<usize> = Vec::new();
        for (idx, (_, line)) in lines.iter().enumerate() {
            if re.is_match(line) {
                matched_lines.push(idx);
            }
        }

        if matched_lines.is_empty() {
            continue;
        }

        files_with_matches += 1;
        total_matches += matched_lines.len();

        // Output modes
        if args.files_only {
            println!("{}", path.display().to_string().cyan());
            continue;
        }
        if args.count_only {
            println!(
                "{}: {}",
                path.display().to_string().cyan(),
                matched_lines.len().to_string().yellow()
            );
            continue;
        }

        // Full output with context
        println!("\n{}", path.display().to_string().cyan().bold());
        let mut printed: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for &m_idx in &matched_lines {
            let start = m_idx.saturating_sub(args.before);
            let end = (m_idx + args.after + 1).min(lines.len());
            for i in start..end {
                if printed.contains(&i) {
                    continue;
                }
                printed.insert(i);
                let (lineno, line) = lines[i];
                if i == m_idx {
                    let highlighted = re.replace_all(line, |c: &regex::Captures| {
                        c[0].red().bold().to_string()
                    });
                    println!(
                        "  {}: {}",
                        lineno.to_string().green(),
                        highlighted
                    );
                } else {
                    println!(
                        "  {}| {}",
                        lineno.to_string().dimmed(),
                        line.dimmed()
                    );
                }
            }
        }
    }

    eprintln!(
        "\n{} matches in {} files",
        total_matches.to_string().yellow(),
        files_with_matches.to_string().yellow()
    );

    Ok(())
}
