use anyhow::Result;
use clap::Args;
use colored::*;
use std::io::Write;
use std::path::PathBuf;

#[derive(Args)]
pub struct BinCatArgs {
    /// Files to concatenate (in order)
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Output file (required — raw bytes)
    #[arg(short = 'o', long, required = true)]
    output: PathBuf,
}

pub fn run(args: BinCatArgs) -> Result<()> {
    let mut out = std::fs::File::create(&args.output)?;
    let mut total = 0u64;
    for f in &args.files {
        if !f.exists() { anyhow::bail!("File not found: {}", f.display()); }
        let bytes = std::fs::read(f)?;
        out.write_all(&bytes)?;
        total += bytes.len() as u64;
        println!("  {} {} ({} bytes)", "+".cyan(), f.display().to_string().dimmed(), bytes.len().to_string().yellow());
    }
    println!("{} {} ({} bytes total from {} files)",
        "Wrote:".green().bold(),
        args.output.display().to_string().cyan(),
        total.to_string().yellow(),
        args.files.len().to_string().yellow()
    );
    Ok(())
}
