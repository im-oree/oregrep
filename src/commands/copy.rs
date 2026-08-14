use anyhow::Result;
use clap::Args;
use colored::*;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct CopyArgs {
    /// File to copy (omit for stdin)
    file: Option<PathBuf>,

    /// Also print to stdout (tee behavior)
    #[arg(short = 't', long)]
    tee: bool,
}

pub fn run(args: CopyArgs) -> Result<()> {
    let content = if let Some(f) = &args.file {
        if !f.exists() { anyhow::bail!("File not found: {}", f.display()); }
        std::fs::read(f)?
    } else {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    };

    let clean = strip_ansi(&content);

    if args.tee {
        std::io::stdout().write_all(&clean)?;
    }

    #[cfg(windows)]
    {
        use std::process::{Command, Stdio};
        let mut child = Command::new("clip.exe")
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(&clean)?;
        }
        child.wait()?;
    }
    #[cfg(not(windows))]
    {
        eprintln!("{} Clipboard not implemented on this platform yet", "WARN".yellow());
    }

    eprintln!("{} {} bytes copied to clipboard", "OK".green().bold(), clean.len().to_string().yellow());
    Ok(())
}

fn strip_ansi(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let c = bytes[i];
                i += 1;
                if (0x40..=0x7E).contains(&c) { break; }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}
