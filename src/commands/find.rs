use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use ignore::WalkBuilder;
use regex::{Regex, RegexBuilder};
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

    /// Skip matches inside line comments (// # -- and /* ... */ single-line)
    #[arg(long)]
    exclude_comments: bool,

    /// Skip matches inside string literals ("..." '...' `...`)
    #[arg(long)]
    exclude_strings: bool,

    /// Show N lines of context around each match (like cat-around, but repo-wide)
    /// Overrides -B and -A when set.
    #[arg(long, value_name = "N")]
    show: Option<usize>,
}

pub fn run(args: FindArgs) -> Result<()> {
    // --show N is a shorthand for -B N -A N (context both sides)
    let effective_before = args.show.unwrap_or(args.before);
    let effective_after = args.show.unwrap_or(args.after);

    // Build regex
    let mut pattern = if args.literal {
        regex::escape(&args.pattern)
    } else {
        // Convenience: accept grep-style \| as alternation (Rust regex uses bare |)
        // Users often paste patterns that came from bash-quoted contexts.
        // We only replace \| (not \\|) so escaped backslash-pipe still works.
        let cleaned = convert_grep_alternation(&args.pattern);
        cleaned
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

    // Handle single-file paths — WalkBuilder expects a directory. If user
    // passes a file, process only that file (skip directory traversal).
    if args.path.is_file() {
        return search_single_file(&args.path, &re, &args);
    }

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
            if !re.is_match(line) { continue; }

            // Skip line-comment-only lines when excluding comments
            if args.exclude_comments {
                let t = line.trim_start();
                if t.starts_with("//") || t.starts_with("#") || t.starts_with("--")
                    || t.starts_with("*") || t.starts_with("/*") {
                    continue;
                }
                // Also check: is the match inside a // comment mid-line?
                if let Some(m) = re.find(line) {
                    if let Some(comment_pos) = line.find("//") {
                        if m.start() > comment_pos { continue; }
                    }
                }
            }

            // Skip matches inside string literals (heuristic)
            if args.exclude_strings {
                if let Some(m) = re.find(line) {
                    if in_string_literal(line, m.start()) { continue; }
                }
            }

            matched_lines.push(idx);
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
            let start = m_idx.saturating_sub(effective_before);
            let end = (m_idx + effective_after + 1).min(lines.len());
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

/// Convert grep-style `\|` alternation to Rust regex `|`.
/// Preserves `\\|` (escaped backslash followed by literal pipe).
fn convert_grep_alternation(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // \\| → keep as \\| (escaped backslash + literal pipe)
        if i + 2 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b'\\' && bytes[i + 2] == b'|' {
            out.push_str("\\\\|");
            i += 3;
            continue;
        }
        // \| → | (grep-style alternation → Rust regex alternation)
        if i + 1 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b'|' {
            out.push('|');
            i += 2;
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Search a single file directly — bypasses WalkBuilder which expects a directory.
fn search_single_file(path: &std::path::Path, re: &Regex, args: &FindArgs) -> Result<()> {
    // Skip binary unless requested
    if !args.binary {
        if let Ok(true) = is_binary(path) {
            eprintln!("(binary file, use --binary to search)");
            return Ok(());
        }
    }

    let content = match read_file_smart(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read {}: {}", path.display(), e);
            return Ok(());
        }
    };

    let effective_before = args.show.unwrap_or(args.before);
    let effective_after = args.show.unwrap_or(args.after);

    let lines: Vec<(usize, &str)> = enumerate_lines(&content).collect();
    let mut matched_lines: Vec<usize> = Vec::new();
    for (idx, (_, line)) in lines.iter().enumerate() {
        if !re.is_match(line) { continue; }

        if args.exclude_comments {
            let t = line.trim_start();
            if t.starts_with("//") || t.starts_with("#") || t.starts_with("--")
                || t.starts_with("*") || t.starts_with("/*") {
                continue;
            }
            if let Some(m) = re.find(line) {
                if let Some(comment_pos) = line.find("//") {
                    if m.start() > comment_pos { continue; }
                }
            }
        }

        if args.exclude_strings {
            if let Some(m) = re.find(line) {
                if in_string_literal(line, m.start()) { continue; }
            }
        }

        matched_lines.push(idx);
    }

    if matched_lines.is_empty() {
        eprintln!("\n{} matches in 1 file", "0".dimmed());
        return Ok(());
    }

    if args.files_only {
        println!("{}", path.display().to_string().cyan());
        eprintln!("\n{} matches in 1 file", matched_lines.len().to_string().yellow());
        return Ok(());
    }
    if args.count_only {
        println!("{}: {}", path.display().to_string().cyan(), matched_lines.len().to_string().yellow());
        return Ok(());
    }

    println!("\n{}", path.display().to_string().cyan().bold());
    let mut printed: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &m_idx in &matched_lines {
        let start = m_idx.saturating_sub(effective_before);
        let end = (m_idx + effective_after + 1).min(lines.len());
        for i in start..end {
            if printed.contains(&i) { continue; }
            printed.insert(i);
            let (lineno, line) = lines[i];
            if i == m_idx {
                let highlighted = re.replace_all(line, |c: &regex::Captures| {
                    c[0].red().bold().to_string()
                });
                println!("  {}: {}", lineno.to_string().green(), highlighted);
            } else {
                println!("  {}| {}", lineno.to_string().dimmed(), line.dimmed());
            }
        }
    }

    eprintln!("\n{} matches in 1 file", matched_lines.len().to_string().yellow());
    Ok(())
}

/// Heuristic: is byte position `pos` inside a string literal on this line?
/// Counts unescaped quotes before pos — odd count = inside a string.
fn in_string_literal(line: &str, pos: usize) -> bool {
    let bytes = line.as_bytes();
    let end = pos.min(bytes.len());
    let mut in_dq = false;
    let mut in_sq = false;
    let mut in_bt = false;
    let mut i = 0;
    while i < end {
        let c = bytes[i];
        if c == b'\\' && i + 1 < end { i += 2; continue; }
        if !in_sq && !in_bt && c == b'"' { in_dq = !in_dq; }
        else if !in_dq && !in_bt && c == b'\'' { in_sq = !in_sq; }
        else if !in_dq && !in_sq && c == b'`' { in_bt = !in_bt; }
        i += 1;
    }
    in_dq || in_sq || in_bt
}
