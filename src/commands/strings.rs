use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct StringsArgs {
    file: PathBuf,

    /// Minimum string length
    #[arg(short = 'n', long, default_value = "4")]
    min: usize,

    /// Show offset before each string
    #[arg(short = 'o', long)]
    offsets: bool,

    /// Also include UTF-16 LE strings
    #[arg(short = 'u', long)]
    utf16: bool,

    /// Max results (0 = all)
    #[arg(short = 'm', long, default_value = "0")]
    max: usize,
}

pub fn run(args: StringsArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let data = std::fs::read(&args.file)?;
    let mut results: Vec<(usize, String, &'static str)> = Vec::new();

    // ASCII pass
    let mut current: Vec<u8> = Vec::new();
    let mut current_start = 0usize;
    for (i, b) in data.iter().enumerate() {
        if b.is_ascii_graphic() || *b == b' ' || *b == b'\t' {
            if current.is_empty() { current_start = i; }
            current.push(*b);
        } else {
            if current.len() >= args.min {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    results.push((current_start, s, "ascii"));
                }
            }
            current.clear();
        }
    }
    if current.len() >= args.min {
        if let Ok(s) = String::from_utf8(current.clone()) {
            results.push((current_start, s, "ascii"));
        }
    }

    // UTF-16 LE pass (optional)
    if args.utf16 && data.len() >= 2 {
        let mut utf: Vec<u16> = Vec::new();
        let mut utf_start = 0usize;
        let mut i = 0usize;
        while i + 1 < data.len() {
            let lo = data[i] as u16;
            let hi = data[i + 1] as u16;
            let code = lo | (hi << 8);
            let c = char::from_u32(code as u32);
            let printable = c.map(|c| c.is_ascii_graphic() || c == ' ' || c == '\t').unwrap_or(false);
            if printable {
                if utf.is_empty() { utf_start = i; }
                utf.push(code);
                i += 2;
            } else {
                if utf.len() >= args.min {
                    if let Some(s) = String::from_utf16(&utf).ok() {
                        results.push((utf_start, s, "utf16"));
                    }
                }
                utf.clear();
                i += 1;
            }
        }
        if utf.len() >= args.min {
            if let Some(s) = String::from_utf16(&utf).ok() {
                results.push((utf_start, s, "utf16"));
            }
        }
    }

    results.sort_by_key(|r| r.0);
    let n = if args.max == 0 { results.len() } else { results.len().min(args.max) };
    for (offset, s, kind) in results.iter().take(n) {
        if args.offsets {
            let tag = if *kind == "utf16" { "u16".magenta() } else { "".normal() };
            println!("{:>10}  {}  {}", format!("{:#010x}", offset).dimmed(), tag, s);
        } else {
            println!("{}", s);
        }
    }
    eprintln!("\n{} {} strings extracted", "Total:".bold(), results.len().to_string().yellow());
    Ok(())
}
