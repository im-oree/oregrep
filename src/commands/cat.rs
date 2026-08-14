use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::{is_binary, read_file_smart};

#[derive(Args)]
pub struct CatArgs {
    /// File to print
    file: PathBuf,

    /// Show line numbers
    #[arg(short = 'n', long)]
    number: bool,

    /// Force print even if binary
    #[arg(long)]
    binary: bool,

    /// Show only lines matching pattern
    #[arg(short = 'g', long)]
    grep: Option<String>,

    /// Print raw bytes without decoding (for binary inspection)
    #[arg(long)]
    raw: bool,
}

pub fn run(args: CatArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    if args.raw {
        let bytes = std::fs::read(&args.file)?;
        use std::io::Write;
        std::io::stdout().write_all(&bytes)?;
        return Ok(());
    }

    if !args.binary {
        if is_binary(&args.file)? {
            anyhow::bail!(
                "{} appears to be binary. Use --binary to force or --raw for bytes.",
                args.file.display()
            );
        }
    }

    let content = read_file_smart(&args.file)?;
    let grep_re = if let Some(pat) = &args.grep {
        Some(regex::Regex::new(pat)?)
    } else {
        None
    };

    for (i, line) in content.lines().enumerate() {
        let lineno = i + 1;
        if let Some(re) = &grep_re {
            if !re.is_match(line) {
                continue;
            }
        }
        if args.number {
            println!("{:>6} | {}", lineno.to_string().green(), line);
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}
