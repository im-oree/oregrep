use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{read_for_patch, unescape_arg, write_atomic};

/// Patch with fuzzy anchor matching — finds the closest block to `--find`
/// (using similarity threshold) instead of requiring exact match.
/// Useful when whitespace/comments have drifted since the anchor was captured.
#[derive(Args)]
pub struct PatchFuzzyArgs {
    /// File to patch
    file: PathBuf,

    /// Text to find (fuzzy — line-by-line similarity)
    #[arg(short = 'f', long)]
    find: String,

    /// Replacement text
    #[arg(short = 'r', long, default_value = "")]
    replace: String,

    /// Minimum similarity to accept (0-100)
    #[arg(short = 't', long, default_value = "85")]
    threshold: u32,

    /// Show closest matches and exit (no write)
    #[arg(long)]
    dry_run: bool,

    /// Auto-apply best match without confirmation
    #[arg(short = 'y', long)]
    yes: bool,

    /// Skip backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Case-insensitive matching
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Literal mode: skip unescape
    #[arg(long)]
    literal: bool,
}

pub fn run(args: PatchFuzzyArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let find_str = if args.literal { args.find.clone() } else { unescape_arg(&args.find) };
    let replace_str = if args.literal { args.replace.clone() } else { unescape_arg(&args.replace) };

    let (content, had_bom, newline) = read_for_patch(&args.file)?;
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let find_norm = find_str.replace("\r\n", "\n").replace('\r', "\n");
    let replace_norm = replace_str.replace("\r\n", "\n").replace('\r', "\n");

    let file_lines: Vec<&str> = normalized.lines().collect();
    let find_lines: Vec<&str> = find_norm.lines().collect();

    if find_lines.is_empty() {
        anyhow::bail!("--find is empty");
    }

    let window = find_lines.len();
    let mut best: Option<(usize, u32)> = None;

    for start in 0..=file_lines.len().saturating_sub(window) {
        let win: Vec<&str> = file_lines[start..start + window].to_vec();
        let sim = compute_similarity(&find_lines, &win, args.ignore_case);
        if sim >= args.threshold && best.map(|(_, b)| sim > b).unwrap_or(true) {
            best = Some((start, sim));
        }
    }

    let (start, sim) = match best {
        Some(b) => b,
        None => {
            eprintln!(
                "{} no match ≥ {}% similarity found in {}",
                "No match:".red().bold(),
                args.threshold,
                args.file.display()
            );
            eprintln!("{}", "Try `ore re-anchor` with a lower threshold to explore.".dimmed());
            std::process::exit(1);
        }
    };

    let end = start + window;
    let start_1i = start + 1;
    let end_1i = end;

    println!(
        "{} best match at lines {}-{} ({}% similar)",
        "Found:".green().bold(),
        start_1i.to_string().yellow(),
        end_1i.to_string().yellow(),
        sim.to_string().cyan()
    );

    // Show the diff preview
    println!("\n{}", "Original (in file):".dimmed());
    for i in start..end {
        println!("  {} {}", "-".red(), file_lines[i].red());
    }
    println!("\n{}", "Replacement:".dimmed());
    for line in replace_norm.lines() {
        println!("  {} {}", "+".green(), line.green());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN] no changes written".yellow().bold());
        return Ok(());
    }

    if !args.yes && sim < 95 {
        eprint!("\n{} Apply this patch? [y/N] ", "Confirm:".cyan().bold());
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("{}", "Aborted.".dimmed());
            return Ok(());
        }
    }

    // Backup
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(||
            chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
        );
        let bp = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bp.display().to_string().dimmed());
    }

    // Build new content — respect original line ending
    let mut new_lines: Vec<String> = Vec::with_capacity(file_lines.len() + window);
    for l in &file_lines[..start] {
        new_lines.push(l.to_string());
    }
    if !replace_norm.is_empty() {
        for l in replace_norm.lines() {
            new_lines.push(l.to_string());
        }
    }
    for l in &file_lines[end..] {
        new_lines.push(l.to_string());
    }

    // Preserve trailing newline
    let had_trailing = content.ends_with('\n') || content.ends_with("\r\n");
    let mut new_content = new_lines.join(newline);
    if had_trailing {
        new_content.push_str(newline);
    }

    write_atomic(&args.file, &new_content, had_bom)?;
    println!(
        "{} {} ({} → {} lines)",
        "Patched:".green().bold(),
        args.file.display().to_string().cyan(),
        file_lines.len().to_string().yellow(),
        new_lines.len().to_string().green()
    );

    Ok(())
}

fn compute_similarity(a: &[&str], b: &[&str], ignore_case: bool) -> u32 {
    if a.len() != b.len() { return 0; }
    let mut total: u64 = 0;
    let mut max: u64 = 0;
    for (la, lb) in a.iter().zip(b.iter()) {
        let (la_s, lb_s) = if ignore_case {
            (la.to_lowercase(), lb.to_lowercase())
        } else {
            (la.to_string(), lb.to_string())
        };
        let sim = if la_s == lb_s {
            100
        } else if la_s.trim() == lb_s.trim() {
            95
        } else {
            let dist = strsim::levenshtein(&la_s, &lb_s);
            let m = la_s.len().max(lb_s.len()).max(1);
            (100 - ((dist * 100) / m).min(100)) as u32
        };
        let w = la_s.len().max(1) as u64;
        total += (sim as u64) * w;
        max += 100 * w;
    }
    if max == 0 { return 0; }
    ((total * 100) / max) as u32
}
