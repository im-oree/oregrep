use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::Args;
use colored::*;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct Base64EncodeArgs {
    /// File to encode (omit for stdin)
    file: Option<PathBuf>,

    /// Write encoded output here (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// URL-safe base64 variant
    #[arg(short = 'u', long)]
    url_safe: bool,

    /// Wrap output at N chars (0 = single line)
    #[arg(short = 'w', long, default_value = "0")]
    wrap: usize,
}

pub fn run(args: Base64EncodeArgs) -> Result<()> {
    let bytes = if let Some(f) = &args.file {
        std::fs::read(f)?
    } else {
        let mut b = Vec::new();
        std::io::stdin().read_to_end(&mut b)?;
        b
    };

    let encoded = if args.url_safe {
        base64::engine::general_purpose::URL_SAFE.encode(&bytes)
    } else {
        STANDARD.encode(&bytes)
    };

    let wrapped = if args.wrap > 0 {
        encoded.as_bytes()
            .chunks(args.wrap)
            .map(|c| std::str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    } else { encoded };

    if let Some(o) = args.output {
        std::fs::write(&o, &wrapped)?;
        eprintln!("{} {} ({} bytes → {} b64 chars)",
            "Wrote:".green().bold(),
            o.display().to_string().cyan(),
            bytes.len().to_string().yellow(),
            wrapped.len().to_string().yellow());
    } else {
        println!("{}", wrapped);
    }
    Ok(())
}
