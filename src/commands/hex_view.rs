use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::hex::{format_hex_dump, parse_offset};

#[derive(Args)]
pub struct HexViewArgs {
    file: PathBuf,

    /// Start offset (supports 0x, k, m, g suffixes)
    #[arg(short = 'o', long)]
    offset: Option<String>,

    /// Byte count to show (0 = to end)
    #[arg(short = 'l', long, default_value = "512")]
    length: usize,

    /// Bytes per line
    #[arg(short = 'w', long, default_value = "16")]
    width: usize,
}

pub fn run(args: HexViewArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let all = std::fs::read(&args.file)?;
    let start = args.offset.as_deref().map(parse_offset).transpose()?.unwrap_or(0) as usize;
    if start >= all.len() {
        println!("{} start offset {} >= file size {}", "!".yellow(), start, all.len());
        return Ok(());
    }
    let end = if args.length == 0 { all.len() } else { (start + args.length).min(all.len()) };
    let slice = &all[start..end];

    println!("{} {} ({} bytes total, showing {}..{})",
        "File:".dimmed(),
        args.file.display().to_string().cyan(),
        all.len().to_string().yellow(),
        start.to_string().yellow(),
        end.to_string().yellow()
    );
    print!("{}", format_hex_dump(slice, start, args.width));
    Ok(())
}
