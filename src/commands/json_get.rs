use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::formats::{nav_get, value_to_display};

#[derive(Args)]
pub struct JsonGetArgs {
    file: PathBuf,
    /// Path like "foo.bar[0].baz" or "foo/bar/0/baz"
    path: String,
    /// Pretty print objects/arrays
    #[arg(short = 'p', long)]
    pretty: bool,
    /// Print as raw JSON always (even for scalars)
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: JsonGetArgs) -> Result<()> {
    let content = read_file_smart(&args.file).with_context(|| format!("Reading {}", args.file.display()))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Parsing {}", args.file.display()))?;
    match nav_get(&value, &args.path) {
        Some(v) => {
            if args.json { println!("{}", if args.pretty { serde_json::to_string_pretty(v)? } else { serde_json::to_string(v)? }); }
            else { println!("{}", value_to_display(v, args.pretty)); }
        }
        None => {
            eprintln!("{} Path not found: {}", "!".yellow(), args.path);
            std::process::exit(1);
        }
    }
    Ok(())
}
