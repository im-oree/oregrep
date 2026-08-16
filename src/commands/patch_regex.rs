use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::patch::{read_for_patch, unescape_arg, write_atomic};

#[derive(Args)]
pub struct PatchRegexArgs {
    /// File to patch
    file: PathBuf,

    /// Regex pattern to find (Rust regex syntax; supports capture groups)
    #[arg(short = 'f', long)]
    find: String,

    /// Replacement string (supports $1, $2, ${name} capture group refs)
    #[arg(short = 'r', long, default_value = "")]
    replace: String,

    /// Replace all matches (default: fail if not exactly 1)
    #[arg(short = 'a', long)]
    all: bool,

    /// Replace only the Nth match (1-indexed)
    #[arg(short = 'n', long)]
    nth: Option<usize>,

    /// Replace only the first match
    #[arg(long)]
    first: bool,

    /// Replace only the last match
    #[arg(long)]
    last: bool,

    /// Case-insensitive matching (or use (?i) inline in pattern)
    #[arg(short = 'i', long)]
    ignore_case: bool,

    /// Skip creating a backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label (default: timestamp)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry run: show match count, don't write
    #[arg(long)]
    dry_run: bool,

    /// Show unified diff preview before writing (implies --dry-run if combined)
    #[arg(long)]
    preview: bool,

    /// Context lines for --preview diff (default: 3)
    #[arg(short = 'C', long, default_value = "3")]
    context: usize,

    /// Literal mode: do not unescape \n \t \\ in replacement string
    #[arg(long)]
    literal: bool,
}

pub fn run(args: PatchRegexArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let (content, had_bom, newline) = read_for_patch(&args.file)?;

    // Build pattern — wrap with (?i) if ignore_case
    let pattern = if args.ignore_case {
        format!("(?i){}", args.find)
    } else {
        args.find.clone()
    };

    let re = Regex::new(&pattern)
        .map_err(|e| anyhow::anyhow!("Invalid regex: {}", e))?;

    // Unescape replacement (handles \n \t \\ in the replacement string) unless --literal
    let replace_unesc = if args.literal {
        args.replace.clone()
    } else {
        unescape_arg(&args.replace)
    };
    let replace_norm = replace_unesc.replace("\r\n", "\n").replace('\n', newline);

    // Collect all match positions (byte offsets)
    let match_positions: Vec<(usize, usize)> = re
        .find_iter(&content)
        .map(|m| (m.start(), m.end()))
        .collect();
    let match_count = match_positions.len();

    // Validate mode
    if args.all || args.first || args.last || args.nth.is_some() {
        // Multi-mode: we'll handle below
    } else {
        // Once mode: require exactly 1
        if match_count == 0 {
            anyhow::bail!("Regex pattern not found in content");
        }
        if match_count > 1 {
            anyhow::bail!(
                "Regex pattern matches {} times, expected exactly 1. Use --all or --nth N.",
                match_count
            );
        }
    }

    if match_count == 0 {
        anyhow::bail!("Regex pattern not found in content");
    }

    // Build new content based on mode
    let new_content: String = if args.all {
        re.replace_all(&content, replace_norm.as_str()).into_owned()
    } else if args.first {
        re.replacen(&content, 1, replace_norm.as_str()).into_owned()
    } else if args.last {
        // Replace last: rebuild manually
        let last = match_positions.last().unwrap();
        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..last.0]);
        // Use regex replace on just the last match span to honour capture refs
        let span = &content[last.0..last.1];
        let replaced = re.replace(span, replace_norm.as_str());
        result.push_str(&replaced);
        result.push_str(&content[last.1..]);
        result
    } else if let Some(n) = args.nth {
        if n == 0 {
            anyhow::bail!("--nth is 1-indexed, cannot be 0");
        }
        if n > match_count {
            anyhow::bail!(
                "Requested match #{} but only {} occurrences exist",
                n,
                match_count
            );
        }
        let target = match_positions[n - 1];
        let mut result = String::with_capacity(content.len());
        result.push_str(&content[..target.0]);
        let span = &content[target.0..target.1];
        let replaced = re.replace(span, replace_norm.as_str());
        result.push_str(&replaced);
        result.push_str(&content[target.1..]);
        result
    } else {
        // Once (already validated exactly 1 above)
        re.replace(&content, replace_norm.as_str()).into_owned()
    };

    let replacements_made = if args.all {
        match_count
    } else {
        1
    };

    // --dry-run: report only
    if args.dry_run && !args.preview {
        println!(
            "{} {}",
            "[DRY RUN]".yellow().bold(),
            args.file.display().to_string().cyan()
        );
        println!(
            "  {} matches found, {} would be replaced",
            match_count.to_string().yellow(),
            replacements_made.to_string().green()
        );
        return Ok(());
    }

    // --preview: show unified diff
    if args.preview {
        println!("{} {}", "---".red(), args.file.display().to_string().red());
        println!(
            "{} {} {}",
            "+++".green(),
            args.file.display().to_string().green(),
            "(after patch-regex)".dimmed()
        );
        println!(
            "{}",
            format!(
                "    {} match{}, {} would be replaced",
                match_count,
                if match_count == 1 { "" } else { "es" },
                replacements_made
            )
            .dimmed()
        );
        println!();

        let diff = TextDiff::from_lines(&content, &new_content);
        for (group_idx, group) in diff.grouped_ops(args.context).iter().enumerate() {
            if group_idx > 0 {
                println!("{}", "---".dimmed());
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let line_content: String = change
                        .iter_strings_lossy()
                        .map(|(_, s)| s.to_string())
                        .collect();
                    let display = line_content.trim_end_matches(['\n', '\r']);
                    match change.tag() {
                        ChangeTag::Delete => {
                            println!("{} {}", "-".red().bold(), display.red())
                        }
                        ChangeTag::Insert => {
                            println!("{} {}", "+".green().bold(), display.green())
                        }
                        ChangeTag::Equal => {
                            println!("  {}", display.dimmed())
                        }
                    }
                }
            }
        }
        println!();
        println!(
            "{}  run {} to apply",
            "Preview only —".dimmed(),
            format!("ore patch-regex {}", args.file.display()).cyan()
        );
        return Ok(());
    }

    // Write — backup first
    if !args.no_backup {
        let label = args
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let backup_path = create_backup(&args.file, &label)?;
        println!(
            "{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    write_atomic(&args.file, &new_content, had_bom)?;

    println!(
        "{} {} ({} replacement{})",
        "Patched:".green().bold(),
        args.file.display().to_string().cyan(),
        replacements_made.to_string().yellow(),
        if replacements_made == 1 { "" } else { "s" }
    );

    Ok(())
}
