use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::keys::Provider;
use crate::engine::ai::prompts::get as get_prompt;
use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
use crate::engine::ai::router::route as ai_route;
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::usage::record;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct AiReviewArgs {
    file: PathBuf,

    #[arg(short = 'm', long)]
    model: Option<String>,

    #[arg(long)]
    no_stream: bool,

    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: AiReviewArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, "review", "review")?;
        (decision.provider, decision.model)
    };

    let content = read_file_smart(&args.file)?;
    // Auto-condense if the file is large enough to risk blowing the context budget.
    let budget_chars = (cfg.context_budget_tokens as usize).saturating_mul(4).saturating_sub(4000);
    let content = if content.len() > budget_chars.min(30_000) {
        eprintln!("{} file is large ({} chars) — condensing before send", "!".yellow(), content.len());
        crate::commands::condense::condense(&content, crate::commands::condense::Level::Medium)
    } else { content };
    let system = get_prompt("review").unwrap_or_else(|_| "You are a senior reviewer.".to_string());
    let user = format!("Review this file for issues.\n\nFile: `{}`\n\n```\n{}\n```", args.file.display(), content);

    let renderer = if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || { while let Ok(ev) = rx.recv() { render(&ev, &renderer); } });

    let req = GenerateRequest {
        provider, model: model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user },
        ],
        max_tokens: cfg.max_output_tokens,
        temperature: 0.2,
        stream: !args.no_stream && cfg.stream,
    };
    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let result = rt.block_on(async move { generate(req, Some(tx_clone)).await });
    match result {
        Ok(res) => {
            let _ = tx.send(AiEvent::Complete {
                tokens_in: res.tokens_in, tokens_out: res.tokens_out, cost_usd: res.cost_usd, duration_ms: res.duration_ms,
            });
            drop(tx);
            let _ = handle.join();
            let _ = record(provider.as_str(), &model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("review"));
            // Answer is delivered via the Token event; close with a newline.
            println!();
        }
        Err(e) => {
            let _ = tx.send(AiEvent::Error { message: e.to_string() });
            drop(tx);
            let _ = handle.join();
            eprintln!("{}", e.to_string().red());
            std::process::exit(1);
        }
    }
    Ok(())
}
