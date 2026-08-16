use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::ai::config::load;
use crate::engine::ai::keys::Provider;
use crate::engine::ai::models::{load_cached, save_models};
use crate::engine::ai::providers::list_models;
use crate::engine::ai::runtime::build_runtime;

#[derive(Args)]
pub struct AiModelsArgs {
    provider: String,
    /// Force refresh from provider (ignore cache)
    #[arg(short = 'r', long)]
    refresh: bool,
    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: AiModelsArgs) -> Result<()> {
    let p = Provider::parse(&args.provider)?;
    let cfg = load()?;
    let models = if args.refresh {
        let rt = build_runtime()?;
        let m = rt.block_on(async move { list_models(p).await })?;
        save_models(&m)?;
        m
    } else {
        let cached = load_cached(p.as_str(), cfg.models_cache_ttl_secs)?;
        if cached.is_empty() {
            let rt = build_runtime()?;
            let m = rt.block_on(async move { list_models(p).await })?;
            save_models(&m)?;
            m
        } else { cached }
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&models)?);
        return Ok(());
    }

    println!("{} {} ({} models)", "Models for".cyan().bold(), p.as_str().yellow(), models.len().to_string().yellow());
    for m in &models {
        let ctx = m.context_window.map(|c| format!("{}k", c / 1024)).unwrap_or_else(|| "?".to_string());
        let in_c = m.input_cost_per_1m.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "?".to_string());
        let out_c = m.output_cost_per_1m.map(|c| format!("${:.2}", c)).unwrap_or_else(|| "?".to_string());
        let caps = m.capabilities.join(",");
        println!("  {:<45}  ctx {:<6}  in {:<7}  out {:<7}  {}",
            m.id.cyan(), ctx.dimmed(), in_c.yellow(), out_c.yellow(), caps.magenta());
    }
    Ok(())
}
