use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct TailArgs {
    /// File to read
    file: PathBuf,

    /// Number of lines (default 10)
    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,

    /// Show line numbers
    #[arg(short = 'N', long)]
    number: bool,
}

pub fn run(args: TailArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }
    let content = read_file_smart(&args.file)?;
    let all: Vec<&str> = content.lines().collect();
    let total = all.len();
    let start = total.saturating_sub(args.lines);
    for (i, line) in all.iter().enumerate().skip(start) {
        if args.number {
            println!("{:>6} | {}", (i + 1).to_string().green(), line);
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}
