use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::ai::config::{as_pairs, load, save, set_field};

#[derive(Args)]
pub struct AiConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    List,
    Get { key: String },
    Set { key: String, value: String },
    Path,
    Reset { #[arg(short = 'y', long)] yes: bool },
}

pub fn run(args: AiConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::List => {
            let cfg = load()?;
            for (k, v) in as_pairs(&cfg) {
                println!("  {} = {}", k.cyan(), v.yellow());
            }
        }
        ConfigAction::Get { key } => {
            let cfg = load()?;
            for (k, v) in as_pairs(&cfg) {
                if k == key { println!("{}", v); return Ok(()); }
            }
            eprintln!("Unknown key: {}", key);
            std::process::exit(1);
        }
        ConfigAction::Set { key, value } => {
            let mut cfg = load()?;
            set_field(&mut cfg, &key, &value)?;
            save(&cfg)?;
            println!("{} {} = {}", "Set:".green().bold(), key.cyan(), value.yellow());
        }
        ConfigAction::Path => {
            println!("{}", crate::engine::ai::config::config_path()?.display());
        }
        ConfigAction::Reset { yes } => {
            if !yes {
                let ok = crate::engine::confirm::confirm("Reset AI config to defaults?", false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            let cfg = crate::engine::ai::config::AiConfig::default();
            save(&cfg)?;
            println!("{}", "AI config reset.".green().bold());
        }
    }
    Ok(())
}
