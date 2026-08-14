use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct HeadArgs {
    /// File to read
    file: PathBuf,

    /// Number of lines (default 10)
    #[arg(short = 'n', long, default_value = "10")]
    lines: usize,

    /// Show line numbers
    #[arg(short = 'N', long)]
    number: bool,
}

pub fn run(args: HeadArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }
    let content = read_file_smart(&args.file)?;
    for (i, line) in content.lines().take(args.lines).enumerate() {
        if args.number {
            println!("{:>6} | {}", (i + 1).to_string().green(), line);
        } else {
            println!("{}", line);
        }
    }
    Ok(())
}
