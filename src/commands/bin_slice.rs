use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::hex::parse_offset;

#[derive(Args)]
pub struct BinSliceArgs {
    file: PathBuf,

    /// Start offset (inclusive)
    start: String,

    /// End offset (exclusive)
    end: String,

    /// Output file (required — raw bytes)
    #[arg(short = 'o', long, required = true)]
    output: PathBuf,
}

pub fn run(args: BinSliceArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let data = std::fs::read(&args.file)?;
    let s = parse_offset(&args.start)? as usize;
    let e = parse_offset(&args.end)? as usize;
    if s >= data.len() { anyhow::bail!("Start {} past EOF {}", s, data.len()); }
    let e = e.min(data.len());
    if e <= s { anyhow::bail!("End ({}) must be > start ({})", e, s); }
    let slice = &data[s..e];
    std::fs::write(&args.output, slice)?;
    println!("{} {} ({} bytes: {:#010x}..{:#010x})",
        "Sliced:".green().bold(),
        args.output.display().to_string().cyan(),
        slice.len().to_string().yellow(),
        s, e);
    Ok(())
}
