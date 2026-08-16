use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::extract_body;

#[derive(Args)]
pub struct SnippetArgs {
    file: PathBuf,
    symbol: String,

    /// Show line numbers
    #[arg(short = 'N', long)]
    number: bool,

    /// Print a header with file:line-range
    #[arg(short = 'L', long)]
    label: bool,

    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: SnippetArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let (start, _end, body) = extract_body(&content, &args.symbol)
        .ok_or_else(|| anyhow::anyhow!("Symbol '{}' not found in {}", args.symbol, args.file.display()))?;

    let line_start = content[..start].matches('\n').count() + 1;
    let line_end = line_start + body.matches('\n').count();

    let mut out = String::new();
    if args.label {
        out.push_str(&format!("=== {}:{}-{} ({}) ===\n", args.file.display(), line_start, line_end, args.symbol));
    }
    if args.number {
        for (i, l) in body.lines().enumerate() {
            out.push_str(&format!("{:>5} | {}\n", line_start + i, l));
        }
    } else {
        out.push_str(&body);
        if !out.ends_with('\n') { out.push('\n'); }
    }

    if let Some(p) = &args.output {
        std::fs::write(p, &out)?;
        println!("{} {} ({} bytes)", "Wrote:".green().bold(), p.display().to_string().cyan(), out.len().to_string().yellow());
    } else {
        print!("{}", out);
    }
    Ok(())
}
