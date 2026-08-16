use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::ai::config::{as_pairs, load, save, set_field};

#[derive(Args)]
pub struct WebSearchConfigArgs {
    #[command(subcommand)]
    pub action: SearchCfgAction,
}

#[derive(Subcommand)]
pub enum SearchCfgAction {
    List,
    Get { key: String },
    Set { key: String, value: String },
    /// Reset all search-* fields to defaults
    Reset { #[arg(short = 'y', long)] yes: bool },
}

pub fn run(args: WebSearchConfigArgs) -> Result<()> {
    match args.action {
        SearchCfgAction::List => {
            let cfg = load()?;
            for (k, v) in as_pairs(&cfg) {
                if k.starts_with("search_") {
                    println!("  {} = {}", k.cyan(), v.yellow());
                }
            }
        }
        SearchCfgAction::Get { key } => {
            let cfg = load()?;
            for (k, v) in as_pairs(&cfg) {
                if k == key { println!("{}", v); return Ok(()); }
            }
            eprintln!("Unknown key: {}", key);
            std::process::exit(1);
        }
        SearchCfgAction::Set { key, value } => {
            let mut cfg = load()?;
            set_field(&mut cfg, &key, &value)?;
            save(&cfg)?;
            println!("{} {} = {}", "Set:".green().bold(), key.cyan(), value.yellow());
        }
        SearchCfgAction::Reset { yes } => {
            if !yes {
                let ok = crate::engine::confirm::confirm("Reset all search_* config to defaults?", false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            let mut cfg = load()?;
            let defaults = crate::engine::ai::config::AiConfig::default();
            cfg.search_searxng_url = defaults.search_searxng_url;
            cfg.search_timeout_secs = defaults.search_timeout_secs;
            cfg.search_max_results = defaults.search_max_results;
            cfg.search_max_chars_per_result = defaults.search_max_chars_per_result;
            cfg.search_fetch_max_chars = defaults.search_fetch_max_chars;
            cfg.search_fallback_instances = defaults.search_fallback_instances;
            save(&cfg)?;
            println!("{}", "Search config reset.".green().bold());
        }
    }
    Ok(())
}
