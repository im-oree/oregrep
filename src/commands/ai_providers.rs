use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::keys::{registered_providers, Provider};

#[derive(Args)]
pub struct AiProvidersArgs {}

pub fn run(_args: AiProvidersArgs) -> Result<()> {
    let registered: std::collections::HashMap<String, String> = registered_providers()?
        .into_iter().map(|(p, s)| (p.as_str().to_string(), s.label().to_string())).collect();

    println!("{}", "Providers:".cyan().bold());
    for p in Provider::all() {
        let status = registered.get(p.as_str()).cloned().unwrap_or_else(|| "not configured".to_string());
        let color = if registered.contains_key(p.as_str()) { "green" } else { "red" };
        println!("  {:<12}  {}", p.as_str().cyan(), status.color(color));
    }
    Ok(())
}
