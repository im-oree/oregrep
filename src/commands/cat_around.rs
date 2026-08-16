use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct CatAroundArgs {
    /// File to search
    pub file: PathBuf,

    /// Pattern to search for (substring or regex with --regex)
    pub pattern: String,

    /// Lines of context before and after each match (default: 5)
    #[arg(short = 'C', long, default_value = "5")]
    pub context: usize,

    /// Show line numbers
    #[arg(short = 'n', long)]
    pub line_numbers: bool,

    /// Case-insensitive matching
    #[arg(short = 'i', long)]
    pub ignore_case: bool,

    /// Treat pattern as a regular expression
    #[arg(short = 'x', long)]
    pub regex: bool,
}

pub fn run(args: CatAroundArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let content = read_file_smart(&args.file)?;
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    if total == 0 {
        println!("{}", "(empty file)".dimmed());
        return Ok(());
    }

    // Auto-detect grep-style alternation \|  — if present, force regex mode
    let has_grep_alt = args.pattern.contains(r"\|");
    let force_regex = args.regex || has_grep_alt;

    // Build matcher
    let match_indices: Vec<usize> = if force_regex {
        // Convert grep-style \| to Rust regex |
        let converted = convert_grep_alternation(&args.pattern);
        let pattern = if args.ignore_case {
            format!("(?i){}", converted)
        } else {
            converted
        };
        let re = Regex::new(&pattern)
            .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| re.is_match(line))
            .map(|(i, _)| i)
            .collect()
    } else if args.ignore_case {
        let pat_lower = args.pattern.to_lowercase();
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains(&pat_lower))
            .map(|(i, _)| i)
            .collect()
    } else {
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(&args.pattern))
            .map(|(i, _)| i)
            .collect()
    };

    if match_indices.is_empty() {
        eprintln!(
            "{} pattern {:?} not found in {}",
            "No matches:".yellow(),
            args.pattern,
            args.file.display()
        );
        std::process::exit(1);
    }

    // Merge overlapping context windows into contiguous blocks
    let mut blocks: Vec<(usize, usize, Vec<usize>)> = Vec::new(); // (from, to, match_lines)

    for &idx in &match_indices {
        let from = idx.saturating_sub(args.context);
        let to = (idx + args.context).min(total - 1);

        if let Some(last) = blocks.last_mut() {
            if from <= last.1 + 1 {
                // Overlaps or adjacent — extend
                last.1 = last.1.max(to);
                last.2.push(idx);
                continue;
            }
        }
        blocks.push((from, to, vec![idx]));
    }

    // Print
    let file_label = args.file.display().to_string();
    let match_count = match_indices.len();
    println!(
        "{} {} ({} match{})",
        "→".cyan(),
        file_label.cyan().bold(),
        match_count.to_string().yellow(),
        if match_count == 1 { "" } else { "es" }
    );

    for (block_idx, (from, to, match_lines)) in blocks.iter().enumerate() {
        if block_idx > 0 {
            println!("{}", "---".dimmed());
        }

        // If block doesn't start at line 1, show ellipsis
        if *from > 0 {
            println!("{}", "  ...".dimmed());
        }

        for line_idx in *from..=*to {
            let line = lines[line_idx];
            let line_num = line_idx + 1; // 1-indexed for display
            let is_match = match_lines.contains(&line_idx);

            let prefix = if args.line_numbers {
                format!("{:>5} │ ", line_num)
            } else {
                String::new()
            };

            if is_match {
                if args.line_numbers {
                    println!(
                        "{}{}",
                        prefix.yellow().bold(),
                        line.yellow().bold()
                    );
                } else {
                    println!("  {}", line.yellow().bold());
                }
            } else {
                if args.line_numbers {
                    println!("{}{}", prefix.dimmed(), line);
                } else {
                    println!("  {}", line);
                }
            }
        }

        // If block doesn't end at last line, show ellipsis
        if *to < total - 1 {
            println!("{}", "  ...".dimmed());
        }
    }

    Ok(())
}

/// Convert grep-style `\|` alternation to Rust regex `|`.
/// Preserves `\\|` (escaped backslash followed by literal pipe).
fn convert_grep_alternation(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b'\\' && bytes[i + 2] == b'|' {
            out.push_str("\\\\|");
            i += 3;
            continue;
        }
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
