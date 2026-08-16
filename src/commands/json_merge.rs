use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::formats::deep_merge;

#[derive(Args)]
pub struct JsonMergeArgs {
    /// Base file (also the output unless -o)
    base: PathBuf,
    /// One or more files to merge INTO base (later wins for scalars)
    overlays: Vec<PathBuf>,
    /// Replace arrays instead of concatenating
    #[arg(long)]
    replace_arrays: bool,
    #[arg(short = 'p', long, default_value = "true")]
    pretty: bool,
    /// Write output to this file instead of overwriting base
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: JsonMergeArgs) -> Result<()> {
    if args.overlays.is_empty() { anyhow::bail!("Provide at least one overlay file"); }
    let base_str = read_file_smart(&args.base)?;
    let mut base: serde_json::Value = serde_json::from_str(&base_str)?;
    for f in &args.overlays {
        let text = read_file_smart(f).with_context(|| format!("Reading {}", f.display()))?;
        let overlay: serde_json::Value = serde_json::from_str(&text)?;
        deep_merge(&mut base, overlay, args.replace_arrays);
    }
    let out = if args.pretty { serde_json::to_string_pretty(&base)? } else { serde_json::to_string(&base)? };
    let target = args.output.clone().unwrap_or_else(|| args.base.clone());
    if target == args.base && !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.base, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    std::fs::write(&target, out)?;
    println!("{} {} ({} overlays merged)",
        "Merged:".green().bold(),
        target.display().to_string().cyan(),
        args.overlays.len().to_string().yellow());
    Ok(())
}
