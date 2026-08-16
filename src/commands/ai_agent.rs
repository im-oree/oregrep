use anyhow::Result;
use clap::Args;
use std::sync::mpsc::channel;

use crate::engine::ai::agent::{run_agent, AgentConfig};
use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::{render, AiEvent, Renderer};
use crate::engine::ai::keys::Provider;
use crate::engine::ai::prompts::get as get_prompt;
use crate::engine::ai::router::route as ai_route;
use crate::engine::ai::session::{append, load as load_session};
use crate::engine::ai::tools::builtin_tools;

#[derive(Args)]
pub struct AiAgentArgs {
    task: String,

    /// Force a specific model as "provider:model"
    #[arg(short = 'm', long)]
    model: Option<String>,

    /// Auto-approve destructive tool calls
    #[arg(long)]
    auto: bool,

    /// Max iterations before giving up
    #[arg(short = 'i', long, default_value = "10")]
    max_iterations: usize,

    /// JSON events on stderr
    #[arg(long)]
    events_json: bool,

    #[arg(short = 'q', long)]
    quiet: bool,

    /// Session name for persistent memory
    #[arg(short = 's', long)]
    session: Option<String>,

    /// Continue the "default" session
    #[arg(long)]
    r#continue: bool,

    /// Path to an image file (attaches vision context to the task)
    #[arg(long)]
    vision: Option<std::path::PathBuf>,
}

pub fn run(args: AiAgentArgs) -> Result<()> {
    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, &args.task, "refactor")?;
        (decision.provider, decision.model)
    };

    // Resolve session
    let session_name: Option<String> = if args.r#continue {
        Some(args.session.clone().unwrap_or_else(|| "default".to_string()))
    } else {
        args.session.clone()
    };

    let renderer = if args.events_json { Renderer::Json } else if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || {
        while let Ok(ev) = rx.recv() { render(&ev, &renderer); }
    });

    let _ = tx.send(AiEvent::RouterChose {
        provider: provider.as_str().to_string(),
        model: model.clone(),
        reason: if args.model.is_some() { "user override".to_string() } else { format!("heuristic:{}", cfg.cost_mode) },
        estimated_cost_usd: 0.0,
    });

    // Build task prompt with optional session history
    let history = if let Some(ref sname) = session_name {
        load_session(sname, Some(20)).unwrap_or_default()
    } else {
        vec![]
    };
    let task_prompt = if history.is_empty() {
        args.task.clone()
    } else {
        let hist_text: String = history.iter().map(|m| {
            format!("[{}]: {}", m.role, m.content.trim())
        }).collect::<Vec<_>>().join("\n");
        format!("## Prior context\n\n{}\n\n## Task\n\n{}", hist_text, args.task)
    };

    // If --vision given, prepend image description to task prompt via one-shot vision call
    let task_prompt = if let Some(ref img_path) = args.vision {
        use crate::engine::ai::providers::{generate_with_vision, ensure_vision_model, ChatMessage as CM, GenerateRequest as GR};
        use crate::engine::ai::runtime::build_runtime;
        eprintln!("vision: encoding image and describing before agent run…");
        let vision_model = ensure_vision_model(provider.as_str(), &model).unwrap_or_else(|_| model.clone());
        let desc_req = GR {
            provider, model: vision_model,
            messages: vec![CM { role: "user".to_string(), content: "Describe this image in detail for a coding agent that will use it as context.".to_string() }],
            max_tokens: 800, temperature: 0.2, stream: false,
        };
        let rt2 = build_runtime()?;
        let img = img_path.clone();
        let desc = rt2.block_on(async move { generate_with_vision(desc_req, &img, None).await })
            .map(|r| r.text)
            .unwrap_or_else(|e| format!("[vision error: {}]", e));
        format!("## Image context\n\n{}\n\n## Task\n\n{}", desc, task_prompt)
    } else {
        task_prompt
    };

    let system = get_prompt("refactor").unwrap_or_else(|_| "You are an autonomous coding agent with tool access.".to_string());
    let tools = builtin_tools();
    let agent_cfg = AgentConfig {
        provider, model: model.clone(),
        max_iterations: args.max_iterations,
        auto_approve_destructive: args.auto,
        task_label: "agent".to_string(),
    };

    let tx_clone = tx.clone();
    let result = run_agent(&system, &task_prompt, &tools, &cfg, &agent_cfg, Some(tx_clone));
    drop(tx);
    let _ = handle.join();

    match result {
        Ok((answer, _, _, _, _)) => {
            if let Some(ref sname) = session_name {
                let _ = append(sname, "user", &args.task);
                let _ = append(sname, "assistant", &answer);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}
