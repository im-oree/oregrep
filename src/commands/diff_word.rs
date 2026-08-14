use anyhow::Result;
use clap::Args;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct DiffWordArgs {
    file_a: PathBuf,
    file_b: PathBuf,

    /// Character-level instead of word
    #[arg(short = 'c', long)]
    chars: bool,
}

pub fn run(args: DiffWordArgs) -> Result<()> {
    if !args.file_a.exists() { anyhow::bail!("File not found: {}", args.file_a.display()); }
    if !args.file_b.exists() { anyhow::bail!("File not found: {}", args.file_b.display()); }
    let a = read_file_smart(&args.file_a)?;
    let b = read_file_smart(&args.file_b)?;

    let diff = if args.chars {
        TextDiff::from_chars(&a, &b)
    } else {
        TextDiff::from_words(&a, &b)
    };

    println!("{} {}", "---".red(), args.file_a.display().to_string().cyan());
    println!("{} {}", "+++".green(), args.file_b.display().to_string().cyan());
    println!();

    let mut added = 0usize;
    let mut removed = 0usize;
    for change in diff.iter_all_changes() {
        let text = change.value();
        match change.tag() {
            ChangeTag::Delete => { removed += 1; print!("{}", text.red().to_string()); }
            ChangeTag::Insert => { added += 1; print!("{}", text.green().to_string()); }
            ChangeTag::Equal => { print!("{}", text.dimmed().to_string()); }
        }
    }
    println!();
    eprintln!("\n+{} -{} tokens", added.to_string().green(), removed.to_string().red());
    Ok(())
}
