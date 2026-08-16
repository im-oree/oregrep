use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::formats::{nav_set, parse_cli_value};

#[derive(Args)]
pub struct JsonSetArgs {
    file: PathBuf,
    path: String,
    /// Value (JSON literal like 42 / true / "str" / [1,2], or bare string)
    value: String,
    #[arg(short = 'p', long, default_value = "true")]
    pretty: bool,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: JsonSetArgs) -> Result<()> {
    let content = read_file_smart(&args.file).with_context(|| format!("Reading {}", args.file.display()))?;
    let mut value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Parsing {}", args.file.display()))?;
    let new_val = parse_cli_value(&args.value);
    nav_set(&mut value, &args.path, new_val)?;
    let out = if args.pretty { serde_json::to_string_pretty(&value)? } else { serde_json::to_string(&value)? };

    if args.dry_run {
        println!("{} would set {} = {}",
            "[DRY]".yellow(), args.path.cyan(), args.value.yellow());
        return Ok(());
    }
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    std::fs::write(&args.file, out)?;
    println!("{} {} {} = {}",
        "Set:".green().bold(),
        args.file.display().to_string().cyan(),
        args.path.cyan(),
        args.value.yellow());
    Ok(())
}
