use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct YamlToJsonArgs {
    file: PathBuf,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(short = 'c', long)]
    compact: bool,
}

pub fn run(args: YamlToJsonArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let json: serde_json::Value = serde_json::to_value(yaml)?;
    let out = if args.compact { serde_json::to_string(&json)? } else { serde_json::to_string_pretty(&json)? };
    match args.output {
        Some(p) => {
            std::fs::write(&p, out)?;
            println!("{} {}", "Wrote:".green().bold(), p.display().to_string().cyan());
        }
        None => println!("{}", out),
    }
    Ok(())
}
