use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::commands::checksum::{crc32_of, md5_of, sha256_of};

#[derive(Args)]
pub struct VerifyChecksumArgs {
    /// File to verify
    file: PathBuf,

    /// Expected hash
    expected: String,
}

pub fn run(args: VerifyChecksumArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }
    let expected = args.expected.to_lowercase();
    let actual = match expected.len() {
        64 => sha256_of(&args.file)?,
        32 => md5_of(&args.file)?,
        8  => crc32_of(&args.file)?,
        _  => anyhow::bail!("Cannot determine hash algo from length {}", expected.len()),
    };
    if actual == expected {
        println!("{} {}  ({})",
            "MATCH".green().bold(),
            args.file.display().to_string().cyan(),
            actual.dimmed()
        );
        Ok(())
    } else {
        println!("{} {}",
            "MISMATCH".red().bold(),
            args.file.display().to_string().cyan()
        );
        println!("  expected: {}", expected.yellow());
        println!("  actual:   {}", actual.red());
        std::process::exit(1);
    }
}
