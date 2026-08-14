use anyhow::Result;
use clap::Args;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct DiffIgnoreArgs {
    file_a: PathBuf,
    file_b: PathBuf,

    /// Ignore whitespace
    #[arg(short = 'w', long)]
    whitespace: bool,

    /// Ignore blank lines
    #[arg(short = 'b', long)]
    blank_lines: bool,

    /// Ignore case
    #[arg(short = 'i', long)]
    case: bool,

    /// Ignore comments (// and #)
    #[arg(short = 'c', long)]
    comments: bool,

    /// Context lines
    #[arg(short = 'C', long, default_value = "3")]
    context: usize,
}

pub fn run(args: DiffIgnoreArgs) -> Result<()> {
    if !args.file_a.exists() { anyhow::bail!("File not found: {}", args.file_a.display()); }
    if !args.file_b.exists() { anyhow::bail!("File not found: {}", args.file_b.display()); }
    let mut a = read_file_smart(&args.file_a)?;
    let mut b = read_file_smart(&args.file_b)?;

    let apply = |s: &str| -> String {
        let mut out: Vec<String> = Vec::new();
        for line in s.lines() {
            let mut t = line.to_string();
            if args.comments {
                if let Some(idx) = find_comment_start(&t) {
                    t.truncate(idx);
                }
            }
            if args.whitespace {
                t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            }
            if args.case {
                t = t.to_lowercase();
            }
            if args.blank_lines && t.trim().is_empty() {
                continue;
            }
            out.push(t);
        }
        out.join("\n")
    };
    a = apply(&a);
    b = apply(&b);

    let diff = TextDiff::from_lines(&a, &b);

    println!("{} {}", "---".red(), args.file_a.display().to_string().cyan());
    println!("{} {}", "+++".green(), args.file_b.display().to_string().cyan());
    let mut flags = Vec::new();
    if args.whitespace { flags.push("whitespace"); }
    if args.blank_lines { flags.push("blank-lines"); }
    if args.case { flags.push("case"); }
    if args.comments { flags.push("comments"); }
    if !flags.is_empty() {
        println!("{} {}", "(ignoring:".dimmed(), format!("{})", flags.join(", ")).dimmed());
    }
    println!();

    let mut added = 0usize;
    let mut removed = 0usize;
    for group in diff.grouped_ops(args.context) {
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
    eprintln!("\n+{} -{}", added.to_string().green(), removed.to_string().red());
    Ok(())
}

fn find_comment_start(s: &str) -> Option<usize> {
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
            b'#' if !in_dq && !in_sq && !in_bt => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}
