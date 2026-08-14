use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::fs;
use std::path::PathBuf;

use crate::engine::backup::create_backup;

#[derive(Args)]
pub struct NewlinesArgs {
    /// File to inspect or convert
    file: PathBuf,

    /// Target newline style (omit for check-only)
    #[arg(short = 't', long)]
    to: Option<NewlineStyle>,

    /// Skip backup
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum NewlineStyle {
    Lf,
    Crlf,
    Cr,
}

pub fn run(args: NewlinesArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let bytes = fs::read(&args.file)?;
    let (crlf, lf_only, cr_only) = count_newlines(&bytes);

    if args.to.is_none() {
        println!("{} {}",
            "File:".dimmed(),
            args.file.display().to_string().cyan()
        );
        println!("  CRLF: {}", crlf.to_string().yellow());
        println!("  LF only: {}", lf_only.to_string().yellow());
        println!("  CR only: {}", cr_only.to_string().yellow());
        let style = if crlf > 0 && lf_only == 0 && cr_only == 0 {
            "CRLF (pure)".green()
        } else if lf_only > 0 && crlf == 0 && cr_only == 0 {
            "LF (pure)".green()
        } else if crlf == 0 && lf_only == 0 && cr_only == 0 {
            "no newlines".dimmed()
        } else {
            "MIXED".red()
        };
        println!("  Style: {}", style);
        return Ok(());
    }

    // Convert
    let text = String::from_utf8_lossy(&bytes).into_owned();
    // Normalize everything to LF first
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let converted = match args.to.unwrap() {
        NewlineStyle::Lf => normalized,
        NewlineStyle::Crlf => normalized.replace('\n', "\r\n"),
        NewlineStyle::Cr => normalized.replace('\n', "\r"),
    };

    if !args.no_backup {
        let label = args
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let backup_path = create_backup(&args.file, &label)?;
        println!("{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    fs::write(&args.file, converted.as_bytes())?;
    println!("{} {} -> {:?}",
        "Converted:".green().bold(),
        args.file.display().to_string().cyan(),
        args.to.unwrap()
    );

    Ok(())
}

fn count_newlines(bytes: &[u8]) -> (usize, usize, usize) {
    let mut crlf = 0;
    let mut lf_only = 0;
    let mut cr_only = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                crlf += 1;
                i += 2;
                continue;
            }
            cr_only += 1;
        } else if bytes[i] == b'\n' {
            lf_only += 1;
        }
        i += 1;
    }
    (crlf, lf_only, cr_only)
}
