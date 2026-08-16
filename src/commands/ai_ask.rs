use anyhow::Result;
use clap::Args;
use colored::*;
use std::sync::mpsc::channel;

use crate::engine::ai::agent::{run_agent, AgentConfig};
use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::keys::Provider;
use crate::engine::ai::prompts::get as get_prompt;
use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
use crate::engine::ai::router::route as ai_route;
use crate::engine::ai::runtime::build_runtime;
use crate::engine::ai::session::{append, load as load_session};
use crate::engine::ai::tools::{builtin_tools, ToolSpec};
use crate::engine::ai::usage::record;

#[derive(Args)]
pub struct AiAskArgs {
    /// The question. If omitted, reads from stdin.
    pub question: Option<String>,

    /// Force a specific model as "provider:model" (bypasses router)
    #[arg(short = 'm', long)]
    pub model: Option<String>,

    /// Disable streaming
    #[arg(long)]
    pub no_stream: bool,

    /// Emit events as JSON on stderr (for GUI / tooling)
    #[arg(long)]
    pub events_json: bool,

    /// Silent — result only, no event chatter
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Show router decision + timing at end
    #[arg(short = 'W', long)]
    pub why: bool,

    /// Disable agentic tool use (old stateless behavior)
    #[arg(long)]
    pub no_tools: bool,

    /// Also allow destructive tools (rare for ai-ask)
    #[arg(long)]
    pub auto: bool,

    /// Session name for persistent memory
    #[arg(short = 's', long)]
    pub session: Option<String>,

    /// Continue the "default" session (shorthand for --session default)
    #[arg(long)]
    pub r#continue: bool,

    /// Path to an image file (enables vision mode)
    #[arg(long)]
    pub vision: Option<std::path::PathBuf>,
}

/// Read-only tool subset for ai-ask (no destructive tools)
fn readonly_tools() -> Vec<ToolSpec> {
    builtin_tools()
        .into_iter()
        .filter(|t| !t.destructive)
        .collect()
}

pub fn run(args: AiAskArgs) -> Result<()> {
    let cfg = load_cfg()?;

    let question = if let Some(q) = args.question {
        q
    } else {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s.trim().to_string()
    };
    if question.trim().is_empty() {
        anyhow::bail!("Empty question.");
    }

    // Resolve session name
    let session_name: Option<String> = if args.r#continue {
        Some(args.session.clone().unwrap_or_else(|| "default".to_string()))
    } else {
        args.session.clone()
    };

    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, &question, "ask")?;
        (decision.provider, decision.model)
    };

    let renderer = if args.events_json { Renderer::Json }
        else if args.quiet { Renderer::Silent }
        else { Renderer::Cli };

    let (tx, rx) = channel::<AiEvent>();
    let renderer_thread = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &renderer); }
    });

    let _ = tx.send(AiEvent::RouterChose {
        provider: provider.as_str().to_string(),
        model: model.clone(),
        reason: if args.model.is_some() { "user override".to_string() } else { format!("heuristic:{}", cfg.cost_mode) },
        estimated_cost_usd: 0.0,
    });

    // Load session history if requested
    let history: Vec<crate::engine::ai::session::SessionMessage> = if let Some(ref sname) = session_name {
        load_session(sname, Some(40)).unwrap_or_default()
    } else {
        vec![]
    };

    let system_prompt = get_prompt("ask").unwrap_or_else(|_| "You are a helpful assistant.".to_string());

    // ---- Vision one-shot (bypasses agent loop) ----
    if let Some(ref img_path) = args.vision {
        use crate::engine::ai::providers::generate_with_vision;
        let system_prompt = get_prompt("ask").unwrap_or_else(|_| "You are a helpful assistant.".to_string());
        let mut messages = vec![ChatMessage { role: "system".to_string(), content: system_prompt }];
        for h in &history {
            messages.push(ChatMessage { role: h.role.clone(), content: h.content.clone() });
        }
        messages.push(ChatMessage { role: "user".to_string(), content: question.clone() });
        let req = GenerateRequest {
            provider, model: model.clone(), messages,
            max_tokens: cfg.max_output_tokens, temperature: cfg.temperature, stream: false,
        };
        let rt = build_runtime()?;
        let tx_clone = tx.clone();
        let img = img_path.clone();
        let result = rt.block_on(async move { generate_with_vision(req, &img, Some(tx_clone)).await });
        match result {
            Ok(res) => {
                let _ = tx.send(AiEvent::Complete {
                    tokens_in: res.tokens_in, tokens_out: res.tokens_out,
                    cost_usd: res.cost_usd, duration_ms: res.duration_ms,
                });
                drop(tx);
                let _ = renderer_thread.join();
                let _ = record(provider.as_str(), &model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("ask"));
                if let Some(ref sname) = session_name {
                    let _ = append(sname, "user", &question);
                    let _ = append(sname, "assistant", &res.text);
                }
                println!();
            }
            Err(e) => {
                let _ = tx.send(AiEvent::Error { message: e.to_string() });
                drop(tx);
                let _ = renderer_thread.join();
                eprintln!("{}", e.to_string().red());
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    if args.no_tools {
        // ---- Stateless non-agentic path (old behavior) ----
        let mut messages = vec![
            ChatMessage { role: "system".to_string(), content: system_prompt },
        ];
        for h in &history {
            messages.push(ChatMessage { role: h.role.clone(), content: h.content.clone() });
        }
        messages.push(ChatMessage { role: "user".to_string(), content: question.clone() });

        let req = GenerateRequest {
            provider,
            model: model.clone(),
            messages,
            max_tokens: cfg.max_output_tokens,
            temperature: cfg.temperature,
            stream: !args.no_stream && cfg.stream,
        };

        let rt = build_runtime()?;
        let tx_clone = tx.clone();
        let result = rt.block_on(async move { generate(req, Some(tx_clone)).await });

        match result {
            Ok(res) => {
                let _ = tx.send(AiEvent::Complete {
                    tokens_in: res.tokens_in,
                    tokens_out: res.tokens_out,
                    cost_usd: res.cost_usd,
                    duration_ms: res.duration_ms,
                });
                drop(tx);
                let _ = renderer_thread.join();
                let _ = record(provider.as_str(), &model, res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms, Some("ask"));
                if let Some(ref sname) = session_name {
                    let _ = append(sname, "user", &question);
                    let _ = append(sname, "assistant", &res.text);
                }
                if args.why {
                    eprintln!("\n{} {}:{}  {}↑ {}↓  ${:.5}  {}ms",
                        "why".cyan(), provider.as_str().cyan(), model.yellow(),
                        res.tokens_in, res.tokens_out, res.cost_usd, res.duration_ms);
                }
                println!();
            }
            Err(e) => {
                let _ = tx.send(AiEvent::Error { message: e.to_string() });
                drop(tx);
                let _ = renderer_thread.join();
                eprintln!("{}", e.to_string().red());
                std::process::exit(1);
            }
        }
    } else {
        // ---- Agentic path with read-only tools ----
        let tools = readonly_tools();

        // Build a user prompt that includes session history as context
        let user_prompt = if history.is_empty() {
            question.clone()
        } else {
            let hist_text: String = history.iter().map(|m| {
                format!("[{}]: {}", m.role, m.content.trim())
            }).collect::<Vec<_>>().join("\n");
            format!("## Prior conversation\n\n{}\n\n## Current question\n\n{}", hist_text, question)
        };

        let agent_cfg = AgentConfig {
            provider,
            model: model.clone(),
            max_iterations: 4, // ai-ask stays lean: max 4 tool calls before answering
            auto_approve_destructive: args.auto,
            task_label: "ask".to_string(),
        };

        let tx_clone = tx.clone();
        let result = run_agent(&system_prompt, &user_prompt, &tools, &cfg, &agent_cfg, Some(tx_clone));

        drop(tx);
        let _ = renderer_thread.join();

        match result {
            Ok((answer, tokens_in, tokens_out, cost_usd, duration_ms)) => {
                if let Some(ref sname) = session_name {
                    let _ = append(sname, "user", &question);
                    let _ = append(sname, "assistant", &answer);
                }
                if args.why {
                    eprintln!("\n{} {}:{}  {}↑ {}↓  ${:.5}  {}ms",
                        "why".cyan(), provider.as_str().cyan(), model.yellow(),
                        tokens_in, tokens_out, cost_usd, duration_ms);
                }
                println!();
            }
            Err(e) => {
                eprintln!("{}", e.to_string().red());
                std::process::exit(1);
            }
        }
    }
    Ok(())
}
