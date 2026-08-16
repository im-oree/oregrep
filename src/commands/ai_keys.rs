use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;

use crate::engine::ai::keys::{redact, registered_providers, remove_key, set_key, Provider};

#[derive(Args)]
pub struct AiKeysArgs {
    #[command(subcommand)]
    pub action: KeysAction,
}

#[derive(Subcommand)]
pub enum KeysAction {
    /// Register (or overwrite) an API key for a provider
    Register { provider: String, key: String },
    /// Remove a stored key
    Unregister { provider: String, #[arg(short = 'y', long)] yes: bool },
    /// Show which providers have keys registered (env or stored)
    List,
    /// Quick liveness test: fetch model list from the provider
    Test { provider: String },
    /// Replace an existing key
    Rotate { provider: String, new_key: String },
}

pub fn run(args: AiKeysArgs) -> Result<()> {
    match args.action {
        KeysAction::Register { provider, key } => {
            let p = Provider::parse(&provider)?;
            set_key(p, &key)?;
            println!("{} {} → {}", "Registered:".green().bold(), p.as_str().cyan(), redact(&key).dimmed());
        }
        KeysAction::Unregister { provider, yes } => {
            let p = Provider::parse(&provider)?;
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Remove key for {}?", p.as_str()), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            let removed = remove_key(p)?;
            if removed { println!("{} {}", "Removed:".green(), p.as_str().cyan()); }
            else { println!("{} {} not registered", "!".yellow(), p.as_str().cyan()); }
        }
        KeysAction::List => {
            let rows = registered_providers()?;
            if rows.is_empty() { println!("{}", "(no providers configured)".dimmed()); return Ok(()); }
            for (p, source) in rows {
                let key = crate::engine::ai::keys::get_key(p).map(|k| redact(&k)).unwrap_or_default();
                let source_tag = format!("[{}]", source.label()).dimmed();
                println!("  {} {}  {}", p.as_str().cyan(), source_tag, key.yellow());
            }
        }
        KeysAction::Test { provider } => {
            let p = Provider::parse(&provider)?;
            let rt = crate::engine::ai::runtime::build_runtime()?;
            let result = rt.block_on(async move {
                crate::engine::ai::providers::list_models(p).await
            });
            match result {
                Ok(models) => println!("{} {} ({} models available)", "OK".green().bold(), p.as_str().cyan(), models.len().to_string().yellow()),
                Err(e) => { println!("{} {}: {}", "FAIL".red().bold(), p.as_str().cyan(), e); std::process::exit(1); }
            }
        }
        KeysAction::Rotate { provider, new_key } => {
            let p = Provider::parse(&provider)?;
            set_key(p, &new_key)?;
            println!("{} {} → {}", "Rotated:".green().bold(), p.as_str().cyan(), redact(&new_key).dimmed());
        }
    }
    Ok(())
}
