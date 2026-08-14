use anyhow::Result;
use clap::Args;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct DiffSemanticArgs {
    file_a: PathBuf,
    file_b: PathBuf,

    /// Show identical files output too
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: DiffSemanticArgs) -> Result<()> {
    if !args.file_a.exists() { anyhow::bail!("File not found: {}", args.file_a.display()); }
    if !args.file_b.exists() { anyhow::bail!("File not found: {}", args.file_b.display()); }
    let a = read_file_smart(&args.file_a)?;
    let b = read_file_smart(&args.file_b)?;

    // Normalize: strip comments, collapse whitespace, remove blank lines
    let na = normalize(&a);
    let nb = normalize(&b);

    if na == nb {
        println!("{} {} and {} are semantically identical (differ only in formatting/comments).",
            "IDENTICAL".green().bold(),
            args.file_a.display().to_string().cyan(),
            args.file_b.display().to_string().cyan()
        );
        if !args.verbose { return Ok(()); }
    }

    let diff = TextDiff::from_lines(&na, &nb);
    println!("{} {}", "---".red(), args.file_a.display().to_string().cyan());
    println!("{} {}", "+++".green(), args.file_b.display().to_string().cyan());
    println!("{}", "(semantic diff — ignores whitespace and comments)".dimmed());
    println!();

    let mut added = 0usize;
    let mut removed = 0usize;
    for group in diff.grouped_ops(3) {
        println!("{}", "@@".magenta());
        for op in group {
            for change in diff.iter_changes(&op) {
                let text = change.value().trim_end_matches('\n');
                match change.tag() {
                    ChangeTag::Delete => { removed += 1; println!("- {}", text.red()); }
                    ChangeTag::Insert => { added += 1; println!("+ {}", text.green()); }
                    ChangeTag::Equal => { println!("  {}", text.dimmed()); }
                }
            }
        }
    }
    eprintln!("\n+{} -{} lines (semantic)", added.to_string().green(), removed.to_string().red());
    Ok(())
}

fn normalize(s: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in s.lines() {
        // Strip line comments (// and #)
        let mut trimmed = line.to_string();
        if let Some(idx) = find_comment_start(&trimmed) {
            trimmed.truncate(idx);
        }
        // Collapse whitespace runs to single space
        let collapsed: String = trimmed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if collapsed.is_empty() { continue; }
        out.push(collapsed);
    }
    out.join("\n")
}

fn find_comment_start(s: &str) -> Option<usize> {
    // Rough: skip inside strings, find // or # outside quotes
    let bytes = s.as_bytes();
    let mut in_dq = false;
    let mut in_sq = false;
    let mut in_bt = false;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b'"' if !in_sq && !in_bt => in_dq = !in_dq,
            b'\'' if !in_dq && !in_bt => in_sq = !in_sq,
            b'`' if !in_dq && !in_sq => in_bt = !in_bt,
            b'/' if !in_dq && !in_sq && !in_bt => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    return Some(i);
                }
            }
            b'#' if !in_dq && !in_sq && !in_bt => {
                return Some(i);
            }
            _ => {}
        }
        i += 1;
    }
    None
}
