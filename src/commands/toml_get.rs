use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::formats::{nav_get, value_to_display};

#[derive(Args)]
pub struct TomlGetArgs {
    file: PathBuf,
    path: String,
    #[arg(short = 'p', long)]
    pretty: bool,
}

pub fn run(args: TomlGetArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let toml_val: toml::Value = toml::from_str(&content)?;
    let json: serde_json::Value = serde_json::to_value(toml_val)?;
    match nav_get(&json, &args.path) {
        Some(v) => println!("{}", value_to_display(v, args.pretty)),
        None => { eprintln!("Path not found: {}", args.path); std::process::exit(1); }
    }
    Ok(())
}
