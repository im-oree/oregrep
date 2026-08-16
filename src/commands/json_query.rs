use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct JsonQueryArgs {
    file: PathBuf,
    /// JSONPath expression, e.g. "$.foo.bar[?(@.age > 30)].name"
    path: String,
    #[arg(short = 'p', long, default_value = "true")]
    pretty: bool,
}

pub fn run(args: JsonQueryArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
    let matches = jsonpath_lib::select(&value, &args.path)
        .map_err(|e| anyhow::anyhow!("JSONPath error: {}", e))?;
    if matches.is_empty() {
        eprintln!("{} No matches for {}", "!".yellow(), args.path);
        return Ok(());
    }
    for m in &matches {
        if args.pretty { println!("{}", serde_json::to_string_pretty(m)?); }
        else { println!("{}", serde_json::to_string(m)?); }
    }
    eprintln!("\n{} {} matches", "Total:".dimmed(), matches.len().to_string().yellow());
    Ok(())
}
