use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct YamlFmtArgs {
    file: PathBuf,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: YamlFmtArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let out = serde_yaml::to_string(&yaml)?;
    let target = args.output.clone().unwrap_or_else(|| args.file.clone());
    if target == args.file && !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    std::fs::write(&target, out)?;
    println!("{} {}", "Formatted:".green().bold(), target.display().to_string().cyan());
    Ok(())
}
