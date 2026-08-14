use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::hex::{parse_hex_bytes, parse_offset};

#[derive(Args)]
pub struct HexPatchArgs {
    file: PathBuf,

    /// Offset (supports 0x, k, m, g)
    offset: String,

    /// Hex bytes to write at that offset
    bytes: String,

    /// Extend file if offset is past EOF (pad with zeros)
    #[arg(long)]
    extend: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: HexPatchArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let offset = parse_offset(&args.offset)? as usize;
    let new_bytes = parse_hex_bytes(&args.bytes)?;
    let mut data = std::fs::read(&args.file)?;
    let end = offset + new_bytes.len();

    if end > data.len() {
        if !args.extend {
            anyhow::bail!("Write of {} bytes at offset {} extends past EOF ({}). Use --extend to zero-pad.",
                new_bytes.len(), offset, data.len());
        }
        data.resize(end, 0);
    }

    if args.dry_run {
        println!("{} would write {} bytes at {:#010x}..{:#010x}",
            "[DRY]".yellow(),
            new_bytes.len().to_string().yellow(),
            offset, end);
        return Ok(());
    }

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }

    for (i, b) in new_bytes.iter().enumerate() {
        data[offset + i] = *b;
    }
    std::fs::write(&args.file, &data)?;
    println!("{} {} ({} bytes at {:#010x})",
        "Patched:".green().bold(),
        args.file.display().to_string().cyan(),
        new_bytes.len().to_string().yellow(),
        offset
    );
    Ok(())
}
