use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::hex::parse_offset;

#[derive(Args)]
pub struct HexExtractArgs {
    file: PathBuf,

    /// Start offset
    offset: String,

    /// Length in bytes
    length: String,

    /// Output file (omit for stdout as hex)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: HexExtractArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let data = std::fs::read(&args.file)?;
    let start = parse_offset(&args.offset)? as usize;
    let len = parse_offset(&args.length)? as usize;
    if start >= data.len() { anyhow::bail!("Offset {} past EOF {}", start, data.len()); }
    let end = (start + len).min(data.len());
    let slice = &data[start..end];

    if let Some(o) = args.output {
        std::fs::write(&o, slice)?;
        println!("{} {} ({} bytes from {:#010x}..{:#010x})",
            "Wrote:".green().bold(),
            o.display().to_string().cyan(),
            slice.len().to_string().yellow(),
            start, end);
    } else {
        use crate::engine::hex::format_hex_dump;
        print!("{}", format_hex_dump(slice, start, 16));
    }
    Ok(())
}
