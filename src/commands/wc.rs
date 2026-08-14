use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct WcArgs {
    /// File(s) to count
    files: Vec<PathBuf>,

    /// Show only line counts
    #[arg(short = 'l', long)]
    lines_only: bool,

    /// Show only word counts
    #[arg(short = 'w', long)]
    words_only: bool,

    /// Show only byte counts
    #[arg(short = 'c', long)]
    bytes_only: bool,

    /// Show only character counts
    #[arg(short = 'm', long)]
    chars_only: bool,
}

pub fn run(args: WcArgs) -> Result<()> {
    if args.files.is_empty() {
        anyhow::bail!("At least one file required");
    }
    let mut tl = 0usize;
    let mut tw = 0usize;
    let mut tc = 0usize;
    let mut tb = 0usize;

    println!("{:>10} {:>10} {:>10} {:>10}  {}",
        "lines".dimmed(), "words".dimmed(), "chars".dimmed(), "bytes".dimmed(), "file".dimmed());

    for f in &args.files {
        if !f.exists() {
            eprintln!("{} {}", "MISSING:".red(), f.display());
            continue;
        }
        let bytes = std::fs::metadata(f).map(|m| m.len() as usize).unwrap_or(0);
        let content = read_file_smart(f)?;
        let lines = content.lines().count();
        let words = content.split_whitespace().count();
        let chars = content.chars().count();
        tl += lines; tw += words; tc += chars; tb += bytes;

        let show = |c: usize, on: bool| -> String {
            if on || !(args.lines_only || args.words_only || args.chars_only || args.bytes_only) {
                format!("{:>10}", c.to_string().yellow())
            } else { format!("{:>10}", "") }
        };

        println!("{} {} {} {}  {}",
            show(lines, args.lines_only),
            show(words, args.words_only),
            show(chars, args.chars_only),
            show(bytes, args.bytes_only),
            f.display().to_string().cyan()
        );
    }
    if args.files.len() > 1 {
        println!("{:>10} {:>10} {:>10} {:>10}  {}",
            tl.to_string().green(),
            tw.to_string().green(),
            tc.to_string().green(),
            tb.to_string().green(),
            "TOTAL".bold()
        );
    }
    Ok(())
}
