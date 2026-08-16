use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::extract_symbols;

#[derive(Args)]
pub struct OutlineArgs {
    file: PathBuf,

    #[arg(short = 'E', long)]
    exported_only: bool,

    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: OutlineArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let mut syms = extract_symbols(&content, &args.file);
    if args.exported_only { syms.retain(|s| s.exported); }

    if args.json { println!("{}", serde_json::to_string_pretty(&syms)?); return Ok(()); }

    println!("{} {}  ({} symbols)", "Outline:".cyan().bold(), args.file.display().to_string().yellow(), syms.len().to_string().dimmed());
    for s in &syms {
        let star = if s.exported { "*".green().to_string() } else { " ".to_string() };
        println!("  {} {:<7} {:>5}  {}", star, s.kind.short().magenta(), s.line.to_string().dimmed(), s.name.yellow());
    }
    Ok(())
}
