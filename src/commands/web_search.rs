use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::search::search;

#[derive(Args)]
pub struct WebSearchArgs {
    query: String,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,

    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Suppress progress events (result-only)
    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: WebSearchArgs) -> Result<()> {
    let cfg = load_cfg()?;
    let renderer = if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let renderer_thread = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &renderer); }
    });

    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let bundle = rt.block_on(async move { search(&args.query, &cfg, Some(&tx_clone)).await });
    drop(tx);
    let _ = renderer_thread.join();

    let bundle = bundle?;
    let text = if args.json {
        serde_json::to_string_pretty(&bundle)?
    } else {
        let mut out = String::new();
        out.push_str(&format!("Results from {} ({} tried):\n\n", bundle.source, bundle.tried.len()));
        for (i, r) in bundle.results.iter().enumerate() {
            out.push_str(&format!("{}. {}\n   {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
        }
        out
    };

    match args.output {
        Some(p) => {
            std::fs::write(&p, &text)?;
            eprintln!("Wrote: {}", p.display().to_string().cyan());
        }
        None => print!("{}", text),
    }
    Ok(())
}
