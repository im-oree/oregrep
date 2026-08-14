use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::hex::{parse_hex_bytes, parse_offset};

#[derive(Args)]
pub struct HexInsertArgs {
    file: PathBuf,

    /// Offset to insert AT (existing bytes shift forward)
    offset: String,

    /// Hex bytes to insert
    bytes: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: HexInsertArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let offset = parse_offset(&args.offset)? as usize;
    let insert = parse_hex_bytes(&args.bytes)?;
    let mut data = std::fs::read(&args.file)?;
    if offset > data.len() { anyhow::bail!("Offset {} past EOF {}", offset, data.len()); }

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }

    let mut new_data = Vec::with_capacity(data.len() + insert.len());
    new_data.extend_from_slice(&data[..offset]);
    new_data.extend_from_slice(&insert);
    new_data.extend_from_slice(&data[offset..]);
    data = new_data;
    std::fs::write(&args.file, &data)?;

    println!("{} {} ({} bytes inserted at {:#010x}, new size {})",
        "Inserted:".green().bold(),
        args.file.display().to_string().cyan(),
        insert.len().to_string().yellow(),
        offset,
        data.len().to_string().yellow()
    );
    Ok(())
}
