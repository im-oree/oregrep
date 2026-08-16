use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::state::{config_path, Config};

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// List all config values
    List,
    /// Get a specific value
    Get { key: String },
    /// Set a value
    Set { key: String, value: String },
    /// Remove a value
    Rm { key: String },
    /// Show the path to the config file
    Path,
    /// Reset config (delete file)
    Reset {
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

pub fn run(args: ConfigArgs) -> Result<()> {
    match args.action {
        ConfigAction::List => {
            let cfg = Config::load()?;
            if cfg.values.is_empty() { println!("{}", "(no config values set)".dimmed()); return Ok(()); }
            for (k, v) in &cfg.values {
                println!("  {} = {}", k.cyan(), v.yellow());
            }
        }
        ConfigAction::Get { key } => {
            let cfg = Config::load()?;
            match cfg.get(&key) {
                Some(v) => println!("{}", v),
                None => { eprintln!("{} key '{}' not set", "!".yellow(), key); std::process::exit(1); }
            }
        }
        ConfigAction::Set { key, value } => {
            let mut cfg = Config::load()?;
            cfg.set(&key, &value);
            cfg.save()?;
            println!("{} {} = {}", "Set:".green().bold(), key.cyan(), value.yellow());
        }
        ConfigAction::Rm { key } => {
            let mut cfg = Config::load()?;
            match cfg.remove(&key) {
                Some(v) => {
                    cfg.save()?;
                    println!("{} {} (was {})", "Removed:".green(), key.cyan(), v.dimmed());
                }
                None => { eprintln!("{} key '{}' not set", "!".yellow(), key); std::process::exit(1); }
            }
        }
        ConfigAction::Path => println!("{}", config_path()?.display()),
        ConfigAction::Reset { yes } => {
            let ok = crate::engine::confirm::confirm("Delete all config?", yes)?;
            if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            let p = config_path()?;
            if p.exists() { std::fs::remove_file(&p)?; }
            println!("{}", "Config reset.".green().bold());
        }
    }
    Ok(())
}
