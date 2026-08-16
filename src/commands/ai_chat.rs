use anyhow::Result;
use clap::Args;
use colored::*;
use std::io::{BufRead, Write};
use std::sync::mpsc::channel;

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::keys::Provider;
use crate::engine::ai::prompts::get as get_prompt;
use crate::engine::ai::router::route as ai_route;
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::session::{append, load as load_session};
use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
use crate::engine::ai::usage::record;

#[derive(Args)]
pub struct AiChatArgs {
    /// Session name (persists conversation). Default: 'default'
    #[arg(short = 's', long, default_value = "default")]
    session: String,

    /// Force a specific model as "provider:model"
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Disable streaming
    #[arg(long)]
    no_stream: bool,

    /// Quiet event stream
    #[arg(short = 'q', long)]
    quiet: bool,

    /// One-shot: send this message and exit (skip interactive REPL)
    #[arg(short = 'p', long)]
    prompt: Option<String>,
}

pub fn run(args: AiChatArgs) -> Result<()> {
    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, "chat", "ask")?;
        (decision.provider, decision.model)
    };

    println!("{} session: {}  provider: {}  model: {}",
        "▸ chat".cyan().bold(),
        args.session.yellow(),
        provider.as_str().cyan(),
        model.yellow());

    if let Some(p) = args.prompt {
        return one_shot(&args.session, provider, &model, &p, !args.no_stream && cfg.stream, args.quiet);
    }

    println!("{}", "Type '/exit' to leave, '/reset' to clear this session, '/history' to view.".dimmed());
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        print!("\n{} ", ">>>".magenta().bold());
        stdout.flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 { break; }
        let input = line.trim().to_string();
        if input.is_empty() { continue; }
        match input.as_str() {
            "/exit" | "/quit" => break,
            "/reset" => {
                let _ = crate::engine::ai::session::delete(&args.session);
                println!("{}", "(session cleared)".dimmed());
                continue;
            }
            "/history" => {
                let msgs = load_session(&args.session, Some(50))?;
                for m in msgs {
                    let role_c = match m.role.as_str() {
                        "user" => "user".magenta(),
                        "assistant" => "assistant".cyan(),
                        _ => m.role.as_str().dimmed(),
                    };
                    println!("[{}] {}", role_c, m.content.trim());
                }
                continue;
            }
            _ => {}
        }
        one_shot(&args.session, provider, &model, &input, !args.no_stream && cfg.stream, args.quiet)?;
    }
    Ok(())
}

fn one_shot(session: &str, provider: Provider, model: &str, user_text: &str, stream: bool, quiet: bool) -> Result<()> {
    let cfg = load_cfg()?;
    let history = load_session(session, Some(40))?;
    let system = get_prompt("chat-system").unwrap_or_else(|_| "You are ore, a codebase-aware assistant.".to_string());

    let mut messages = vec![ChatMessage { role: "system".to_string(), content: system }];
    for h in &history {
        messages.push(ChatMessage { role: h.role.clone(), content: h.content.clone() });
    }
    messages.push(ChatMessage { role: "user".to_string(), content: user_text.to_string() });

    let renderer = if quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &renderer); }
    });

    let req = GenerateRequest {
        provider, model: model.to_string(), messages,
        max_tokens: cfg.max_output_tokens, temperature: cfg.temperature, stream,
    };
    let rt = build_runtime()?;
    let tx_clone = tx.clone();
    let result = rt.block_on(async move { generate(req, Some(tx_clone)).await });
    match result {
        Ok(res) => {
            let _ = tx.send(AiEvent::Complete {
                tokens_in: res.tokens_in, tokens_out: res.tokens_out,
                cost_usd: res.cost_usd, duration_ms: res.duration_ms,
            });
            drop(tx);
            let _ = handle.join();
            append(session, "user", user_text)?;
            append(session, "assistant", &res.text)?;
            let _ = record(provider.as_str(), model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("chat"));
            if !stream { println!("{}", res.text); }
        }
        Err(e) => {
            let _ = tx.send(AiEvent::Error { message: e.to_string() });
            drop(tx);
            let _ = handle.join();
            eprintln!("{}", e.to_string().red());
        }
    }
    Ok(())
}
