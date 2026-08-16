use anyhow::Result;
use clap::Args;
use colored::*;
use std::sync::mpsc::channel;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::keys::Provider;
use crate::engine::ai::prompts::get as get_prompt;
use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
use crate::engine::ai::router::route as ai_route;
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::usage::record;
use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct AiCommitMessageArgs {
    /// Analyze staged (default) vs all working tree changes
    #[arg(long)]
    unstaged: bool,

    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Actually commit (default: print only)
    #[arg(short = 'c', long)]
    commit: bool,

    #[arg(long)]
    no_stream: bool,

    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: AiCommitMessageArgs) -> Result<()> {
    ensure_repo()?;
    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, "commit-message", "commit-message")?;
        (decision.provider, decision.model)
    };

    let args_git: Vec<&str> = if args.unstaged { vec!["diff"] } else { vec!["diff", "--cached"] };
    let diff = git(&args_git)?;
    if diff.trim().is_empty() {
        eprintln!("{}", "No diff to describe.".yellow());
        return Ok(());
    }

    let system = get_prompt("commit-message").unwrap_or_else(|_| "Write a git commit message for this diff.".to_string());
    let user = format!("Diff:\n\n```diff\n{}\n```\n\nWrite the commit message now.", truncate(&diff, 40000));

    let renderer = if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || { while let Ok(ev) = rx.recv() { render(&ev, &renderer); } });

    let req = GenerateRequest {
        provider, model: model.clone(),
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: user },
        ],
        max_tokens: cfg.max_output_tokens.min(600),
        temperature: 0.2,
        stream: !args.no_stream && cfg.stream,
    };
    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let result = rt.block_on(async move { generate(req, Some(tx_clone)).await });
    let message = match result {
        Ok(res) => {
            let _ = tx.send(AiEvent::Complete { tokens_in: res.tokens_in, tokens_out: res.tokens_out, cost_usd: res.cost_usd, duration_ms: res.duration_ms });
            drop(tx);
            let _ = handle.join();
            let _ = record(provider.as_str(), &model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("commit-message"));
            res.text
        }
        Err(e) => {
            let _ = tx.send(AiEvent::Error { message: e.to_string() });
            drop(tx);
            let _ = handle.join();
            eprintln!("{}", e.to_string().red());
            std::process::exit(1);
        }
    };

    let cleaned = message.trim().trim_matches('"').trim_matches('`').to_string();

    if args.no_stream || !cfg.stream {
        println!("{}", cleaned);
    } else {
        println!();
    }

    if args.commit {
        git(&["commit", "-m", &cleaned])?;
        println!("{}", "Committed.".green().bold());
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let cut: String = s.chars().take(max).collect();
        format!("{}\n[…truncated…]", cut)
    } else { s.to_string() }
}
