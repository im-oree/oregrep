use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::hex::{format_hex_dump, parse_offset};

#[derive(Args)]
pub struct XxdArgs {
    file: PathBuf,

    #[arg(short = 'o', long)]
    offset: Option<String>,

    #[arg(short = 'l', long, default_value = "0")]
    length: usize,

    #[arg(short = 'w', long, default_value = "16")]
    width: usize,
}

pub fn run(args: XxdArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let all = std::fs::read(&args.file)?;
    let start = args.offset.as_deref().map(parse_offset).transpose()?.unwrap_or(0) as usize;
    if start >= all.len() { return Ok(()); }
    let end = if args.length == 0 { all.len() } else { (start + args.length).min(all.len()) };
    print!("{}", format_hex_dump(&all[start..end], start, args.width));
    Ok(())
}
