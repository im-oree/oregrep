use anyhow::Result;
use clap::Args;
use colored::*;
use std::sync::mpsc::channel;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::search::search;

#[derive(Args)]
pub struct AiSearchTestArgs {
    query: String,
}

pub fn run(args: AiSearchTestArgs) -> Result<()> {
    let cfg = load_cfg()?;
    let (tx, rx) = channel::<AiEvent>();
    let renderer_thread = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &Renderer::Cli); }
    });

    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let result = rt.block_on(async move { search(&args.query, &cfg, Some(&tx_clone)).await });
    drop(tx);
    let _ = renderer_thread.join();

    let bundle = result?;
    println!("\n{}", "Retrieval preview (what the agent would see):".cyan().bold());
    println!("{} {}", "Source:".dimmed(), bundle.source);
    println!("{} {}", "Tried:".dimmed(), bundle.tried.join(", "));
    if !bundle.failures.is_empty() {
        println!("{}", "Failures:".yellow());
        for (inst, why) in &bundle.failures {
            println!("  · {} → {}", inst.dimmed(), why.dimmed());
        }
    }
    println!("\n{} {} results (truncated to config limits):", "Results:".cyan(), bundle.results.len().to_string().yellow());
    for (i, r) in bundle.results.iter().enumerate() {
        println!("\n  {}. {}", i + 1, r.title.yellow());
        println!("     {}", r.url.cyan());
        println!("     {}", r.snippet.dimmed());
    }
    Ok(())
}
