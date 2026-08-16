use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct JsonFmtArgs {
    file: PathBuf,
    /// Compact (single line). Default: pretty.
    #[arg(short = 'c', long)]
    compact: bool,
    /// Sort keys alphabetically
    #[arg(short = 's', long)]
    sort_keys: bool,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    /// Write to a different file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: JsonFmtArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
    let value = if args.sort_keys { sort_keys(value) } else { value };
    let out = if args.compact { serde_json::to_string(&value)? } else { serde_json::to_string_pretty(&value)? };
    let target = args.output.clone().unwrap_or_else(|| args.file.clone());
    if target == args.file && !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    std::fs::write(&target, out)?;
    println!("{} {} ({})",
        "Formatted:".green().bold(),
        target.display().to_string().cyan(),
        if args.compact { "compact" } else { "pretty" });
    Ok(())
}

fn sort_keys(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => {
            let mut sorted: std::collections::BTreeMap<String, serde_json::Value> = std::collections::BTreeMap::new();
            for (k, val) in m { sorted.insert(k, sort_keys(val)); }
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(a) => serde_json::Value::Array(a.into_iter().map(sort_keys).collect()),
        _ => v,
    }
}
