use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::formats::{nav_set, parse_cli_value};

#[derive(Args)]
pub struct YamlSetArgs {
    file: PathBuf,
    path: String,
    value: String,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: YamlSetArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let mut json: serde_json::Value = serde_json::to_value(yaml)?;
    let new_val = parse_cli_value(&args.value);
    nav_set(&mut json, &args.path, new_val)?;
    let back_yaml: serde_yaml::Value = serde_json::from_value(json)?;
    let out = serde_yaml::to_string(&back_yaml)?;

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
