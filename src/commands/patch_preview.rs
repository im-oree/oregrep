use anyhow::Result;
use clap::Args;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::patch::{
    apply_patch, read_for_patch, unescape_arg, PatchMode,
};

#[derive(Args)]
pub struct PatchPreviewArgs {
    /// File to preview the patch on
    file: PathBuf,

    /// Text to find (supports \n for multiline)
    #[arg(short = 'f', long)]
    find: String,

    /// Replacement text (supports \n for multiline)
    #[arg(short = 'r', long, default_value = "")]
    replace: String,

    /// Replace all occurrences (default: exactly 1)
    #[arg(short = 'a', long)]
    all: bool,

    /// Replace only the Nth occurrence (1-indexed)
    #[arg(short = 'n', long)]
    nth: Option<usize>,

    /// Replace only the first occurrence
    #[arg(long)]
    first: bool,

    /// Replace only the last occurrence
    #[arg(long)]
    last: bool,

    /// Disable color output (for piping)
    #[arg(long)]
    no_color: bool,

    /// Number of context lines in the diff (default: 3)
    #[arg(short = 'C', long, default_value = "3")]
    context: usize,

    /// Literal mode: do not unescape \n \t \\ in find/replace
    #[arg(long)]
    literal: bool,
}

pub fn run(args: PatchPreviewArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let mode = if args.all {
        PatchMode::All
    } else if let Some(n) = args.nth {
        PatchMode::Nth(n)
    } else if args.first {
        PatchMode::First
    } else if args.last {
        PatchMode::Last
    } else {
        PatchMode::Once
    };

    let (content, _, newline) = read_for_patch(&args.file)?;

    // Unescape then normalize to file's newline style — same pipeline as patch
    let find_unesc = if args.literal {
        args.find.clone()
    } else {
        unescape_arg(&args.find)
    };
    let replace_unesc = if args.literal {
        args.replace.clone()
    } else {
        unescape_arg(&args.replace)
    };
    let find_norm = find_unesc.replace("\r\n", "\n").replace('\n', newline);
    let replace_norm = replace_unesc.replace("\r\n", "\n").replace('\n', newline);

    let (new_content, result) = apply_patch(&content, &find_norm, &replace_norm, mode)?;

    // Header
    let file_str = args.file.display().to_string();
    if args.no_color {
        println!("--- {}", file_str);
        println!("+++ {} (after patch)", file_str);
    } else {
        println!("{} {}", "---".red(), file_str.red());
        println!("{} {} {}", "+++".green(), file_str.green(), "(after patch)".dimmed());
    }

    println!(
        "{}",
        format!(
            "    {} match{} found, {} would be replaced",
            result.matches_found,
            if result.matches_found == 1 { "" } else { "es" },
            result.replacements_made
        )
        .dimmed()
    );
    println!();

    // Unified diff via `similar`
    let diff = TextDiff::from_lines(&content, &new_content);

    let mut last_was_context = false;
    let ops = diff.grouped_ops(args.context);

    for (group_idx, group) in ops.iter().enumerate() {
        if group_idx > 0 {
            println!("{}", "---".dimmed());
        }

        for op in group {
            for change in diff.iter_inline_changes(op) {
                let tag = change.tag();
                let line_content: String = change.iter_strings_lossy()
                    .map(|(_, s)| s.to_string())
                    .collect();

                // Trim trailing newline for display (we add our own newline via println)
                let display = line_content.trim_end_matches(['\n', '\r']);

                match tag {
                    ChangeTag::Delete => {
                        last_was_context = false;
                        if args.no_color {
                            println!("- {}", display);
                        } else {
                            println!("{} {}", "-".red().bold(), display.red());
                        }
                    }
                    ChangeTag::Insert => {
                        last_was_context = false;
                        if args.no_color {
                            println!("+ {}", display);
                        } else {
                            println!("{} {}", "+".green().bold(), display.green());
                        }
                    }
                    ChangeTag::Equal => {
                        last_was_context = true;
                        if args.no_color {
                            println!("  {}", display);
                        } else {
                            println!("  {}", display.dimmed());
                        }
                    }
                }
            }
        }
        let _ = last_was_context;
    }

    // Summary line
    println!();
    if args.no_color {
        println!(
            "Preview only — run 'ore patch {}' to apply",
            args.file.display()
        );
    } else {
        println!(
            "{}  run {} to apply",
            "Preview only —".dimmed(),
            format!("ore patch {}", args.file.display()).cyan()
        );
    }

    Ok(())
}
