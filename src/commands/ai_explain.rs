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
pub struct AiExplainArgs {
    /// File path to explain, OR a natural-language question about the repo
    file_or_question: String,

    #[arg(short = 'm', long)]
    model: Option<String>,

    #[arg(long)]
    no_stream: bool,

    #[arg(short = 'q', long)]
    quiet: bool,

    /// Path to an image file (attach visual context alongside the file/question)
    #[arg(long)]
    vision: Option<std::path::PathBuf>,
}

pub fn run(args: AiExplainArgs) -> Result<()> {
    let path = PathBuf::from(&args.file_or_question);
    let is_file = path.exists() && path.is_file();

    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, "explain", "explain")?;
        (decision.provider, decision.model)
    };

    let (system, user) = if is_file {
        // ---- File mode (existing behavior) ----
        let content = read_file_smart(&path)?;
        let budget_chars = (cfg.context_budget_tokens as usize).saturating_mul(4).saturating_sub(4000);
        let content = if content.len() > budget_chars.min(30_000) {
            eprintln!("{} file is large ({} chars) — condensing before send", "!".yellow(), content.len());
            crate::commands::condense::condense(&content, crate::commands::condense::Level::Medium)
        } else {
            content
        };
        let sys = get_prompt("explain").unwrap_or_else(|_| "Explain what this file does.".to_string());
        let usr = format!("File: `{}`\n\n```\n{}\n```", path.display(), content);
        (sys, usr)
    } else {
        // ---- Question mode: run `ore digest .` for context ----
        eprintln!("{} not a file path — treating as repo question, running digest…", "▸".cyan());
        // Spawn the current exe directly (no shell / PATH dependency), same pattern as
        // workspace-report / report-health. digest has no --no-imports flag; imports are
        // included and trimmed by the budget truncation below.
        let digest_output = match std::process::Command::new(std::env::current_exe()?)
            .args(["digest", "."])
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).into_owned(),
            Err(_) => "(digest unavailable)".to_string(),
        };
        // Truncate digest to fit budget
        let budget_chars = (cfg.context_budget_tokens as usize).saturating_mul(4).saturating_sub(8000);
        let digest_trimmed: String = digest_output.chars().take(budget_chars).collect();

        let sys = format!(
            "You are a codebase expert. The user asked a question about their repository.\n\
             Below is a structural digest of the codebase. Answer the question using this context.\n\
             Be specific and cite file names when relevant."
        );
        let usr = format!(
            "## Codebase Digest\n\n{}\n\n## Question\n\n{}",
            digest_trimmed, args.file_or_question
        );
        (sys, usr)
    };

    let renderer = if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &renderer); }
    });

    let messages = vec![
        ChatMessage { role: "system".to_string(), content: system },
        ChatMessage { role: "user".to_string(), content: user },
    ];
    let req = GenerateRequest {
        provider, model: model.clone(),
        messages,
        max_tokens: cfg.max_output_tokens,
        temperature: 0.3,
        stream: if args.vision.is_some() { false } else { !args.no_stream && cfg.stream },
    };
    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let result = if let Some(ref img_path) = args.vision {
        use crate::engine::ai::providers::generate_with_vision;
        let img = img_path.clone();
        rt.block_on(async move { generate_with_vision(req, &img, Some(tx_clone)).await })
    } else {
        rt.block_on(async move { generate(req, Some(tx_clone)).await })
    };
    match result {
        Ok(res) => {
            let _ = tx.send(AiEvent::Complete {
                tokens_in: res.tokens_in, tokens_out: res.tokens_out,
                cost_usd: res.cost_usd, duration_ms: res.duration_ms,
            });
            drop(tx);
            let _ = handle.join();
            let _ = record(provider.as_str(), &model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("explain"));
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
