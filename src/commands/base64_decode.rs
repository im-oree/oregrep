use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Args;
use colored::*;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct Base64DecodeArgs {
    /// Input file (omit for stdin)
    file: Option<PathBuf>,

    /// Output file (default: stdout as raw bytes)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'u', long)]
    url_safe: bool,
}

pub fn run(args: Base64DecodeArgs) -> Result<()> {
    let mut input = String::new();
    if let Some(f) = &args.file {
        input = std::fs::read_to_string(f)?;
    } else {
        std::io::stdin().read_to_string(&mut input)?;
    }
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    let decoded = if args.url_safe {
        base64::engine::general_purpose::URL_SAFE.decode(&cleaned)?
    } else {
        STANDARD.decode(&cleaned)?
    };

    if let Some(o) = args.output {
        std::fs::write(&o, &decoded)?;
        eprintln!("{} {} ({} b64 chars → {} bytes)",
            "Wrote:".green().bold(),
            o.display().to_string().cyan(),
            cleaned.len().to_string().yellow(),
            decoded.len().to_string().yellow());
    } else {
        std::io::stdout().write_all(&decoded)?;
    }
    Ok(())
}
