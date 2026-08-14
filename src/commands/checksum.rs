use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use md5::{Md5, Digest as _};
use sha2::Sha256;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct ChecksumArgs {
    /// File(s) to checksum
    files: Vec<PathBuf>,

    /// Algorithm
    #[arg(short = 'a', long, default_value = "sha256")]
    algo: Algo,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Algo {
    Sha256,
    Md5,
    Crc32,
    All,
}

pub fn run(args: ChecksumArgs) -> Result<()> {
    if args.files.is_empty() {
        anyhow::bail!("At least one file required");
    }
    for f in &args.files {
        if !f.exists() {
            eprintln!("  {} {}", "MISSING".red(), f.display());
            continue;
        }
        match args.algo {
            Algo::Sha256 => println!("{}  {}", sha256_of(f)?.yellow(), f.display().to_string().cyan()),
            Algo::Md5 => println!("{}  {}", md5_of(f)?.yellow(), f.display().to_string().cyan()),
            Algo::Crc32 => println!("{}  {}", crc32_of(f)?.yellow(), f.display().to_string().cyan()),
            Algo::All => {
                println!("{}", f.display().to_string().cyan().bold());
                println!("  sha256: {}", sha256_of(f)?.yellow());
                println!("  md5:    {}", md5_of(f)?.yellow());
                println!("  crc32:  {}", crc32_of(f)?.yellow());
            }
        }
    }
    Ok(())
}

pub fn sha256_of(path: &PathBuf) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        sha2::Digest::update(&mut hasher, &buf[..n]);
    }
    Ok(format!("{:x}", sha2::Digest::finalize(hasher)))
}

pub fn md5_of(path: &PathBuf) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn crc32_of(path: &PathBuf) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = crc32fast::Hasher::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:08x}", hasher.finalize()))
}
