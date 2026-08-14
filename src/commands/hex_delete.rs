use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::hex::parse_offset;

#[derive(Args)]
pub struct HexDeleteArgs {
    file: PathBuf,

    /// Offset to delete FROM
    offset: String,

    /// Number of bytes to delete
    length: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: HexDeleteArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let offset = parse_offset(&args.offset)? as usize;
    let len = parse_offset(&args.length)? as usize;
    let mut data = std::fs::read(&args.file)?;
    if offset >= data.len() { anyhow::bail!("Offset {} past EOF {}", offset, data.len()); }
    let end = (offset + len).min(data.len());

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }

    let removed = end - offset;
    data.drain(offset..end);
    std::fs::write(&args.file, &data)?;
    println!("{} {} ({} bytes removed from {:#010x}, new size {})",
        "Deleted:".green().bold(),
        args.file.display().to_string().cyan(),
        removed.to_string().yellow(),
        offset,
        data.len().to_string().yellow()
    );
    Ok(())
}
