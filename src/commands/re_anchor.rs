use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

/// Show closest matches to a failed anchor. Uses line-by-line similarity
/// scoring so you can find where the actual code lives even if whitespace/
/// comments have drifted.
#[derive(Args)]
pub struct ReAnchorArgs {
    /// File to search
    file: PathBuf,

    /// The anchor text you tried and failed to match
    #[arg(short = 'f', long)]
    find: String,

    /// How many closest matches to show
    #[arg(short = 'n', long, default_value = "5")]
    top: usize,

    /// Minimum similarity threshold (0-100)
    #[arg(short = 't', long, default_value = "40")]
    threshold: u32,

    /// Context lines around each match
    #[arg(short = 'C', long, default_value = "2")]
    context: usize,

    /// Case-insensitive comparison
    #[arg(short = 'i', long)]
    ignore_case: bool,
}

pub fn run(args: ReAnchorArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let find_unesc = crate::engine::patch::unescape_arg(&args.find);
    let content = read_file_smart(&args.file)?;
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let find_norm = find_unesc.replace("\r\n", "\n").replace('\r', "\n");

    let file_lines: Vec<&str> = normalized.lines().collect();
    let find_lines: Vec<&str> = find_norm.lines().collect();

    if find_lines.is_empty() {
        anyhow::bail!("Anchor text is empty");
    }

    let window_size = find_lines.len();

    // Slide a window of window_size over the file, score each position
    let mut scored: Vec<(usize, u32)> = Vec::new(); // (start_line, similarity 0-100)

    for start in 0..=file_lines.len().saturating_sub(window_size) {
        let window: Vec<&str> = file_lines[start..start + window_size].to_vec();
        let similarity = compute_similarity(&find_lines, &window, args.ignore_case);
        if similarity >= args.threshold {
            scored.push((start, similarity));
        }
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(args.top);

    if scored.is_empty() {
        println!(
            "{} No matches ≥ {}% similarity in {}",
            "No results:".yellow().bold(),
            args.threshold,
            args.file.display()
        );
        std::process::exit(1);
    }

    println!(
        "{} '{}' in {} (top {})",
        "Closest matches to:".cyan().bold(),
        first_line_of(&args.find).dimmed(),
        args.file.display().to_string().cyan(),
        scored.len().to_string().yellow()
    );
    println!();

    for (idx, (start, sim)) in scored.iter().enumerate() {
        let end = (start + window_size).min(file_lines.len());
        let start_1indexed = start + 1;
        let end_1indexed = end;

        println!(
            "{} match — lines {}-{} — {}% similar",
            format!("#{}", idx + 1).yellow().bold(),
            start_1indexed.to_string().cyan(),
            end_1indexed.to_string().cyan(),
            sim.to_string().green()
        );

        let ctx_start = start.saturating_sub(args.context);
        let ctx_end = (end + args.context).min(file_lines.len());

        for i in ctx_start..ctx_end {
            let line_num = i + 1;
            let is_match = i >= *start && i < end;
            let line = file_lines.get(i).unwrap_or(&"");
            if is_match {
                println!(
                    "  {:>5} │ {}",
                    line_num.to_string().yellow().bold(),
                    line.yellow()
                );
            } else {
                println!(
                    "  {:>5} │ {}",
                    line_num.to_string().dimmed(),
                    line.dimmed()
                );
            }
        }
        println!();
    }

    Ok(())
}

fn compute_similarity(a: &[&str], b: &[&str], ignore_case: bool) -> u32 {
    if a.len() != b.len() {
        return 0;
    }
    let mut total_score: u64 = 0;
    let mut max_score: u64 = 0;

    for (la, lb) in a.iter().zip(b.iter()) {
        let (la_s, lb_s) = if ignore_case {
            (la.to_lowercase(), lb.to_lowercase())
        } else {
            (la.to_string(), lb.to_string())
        };
        let sim = line_similarity(&la_s, &lb_s);
        let weight = la_s.len().max(1) as u64;
        total_score += (sim as u64) * weight;
        max_score += 100 * weight;
    }

    if max_score == 0 {
        return 0;
    }
    ((total_score * 100) / max_score) as u32
}

fn line_similarity(a: &str, b: &str) -> u32 {
    if a == b {
        return 100;
    }
    if a.is_empty() && b.is_empty() {
        return 100;
    }
    if a.trim() == b.trim() {
        return 95;
    }
    let dist = strsim::levenshtein(a, b);
    let max_len = a.len().max(b.len()).max(1);
    let sim = 100 - ((dist * 100) / max_len).min(100);
    sim as u32
}

fn first_line_of(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(60).collect()
}
