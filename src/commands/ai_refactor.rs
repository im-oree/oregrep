use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
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
pub struct AiRefactorArgs {
    file: PathBuf,

    /// What you want changed (natural language)
    intent: String,

    #[arg(short = 'm', long)]
    model: Option<String>,

    #[arg(long)]
    auto: bool,

    #[arg(long = "max-iters", default_value = "10")]
    max_iterations: usize,

    #[arg(short = 'q', long)]
    quiet: bool,

    /// Session name for persistent memory
    #[arg(short = 's', long)]
    session: Option<String>,

    /// Continue the "default" session
    #[arg(long)]
    r#continue: bool,
}

pub fn run(args: AiRefactorArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let cfg = load_cfg()?;
    let (provider, model) = if let Some(m) = &args.model {
        let (p, mid) = m.split_once(':').ok_or_else(|| anyhow::anyhow!("Model must be 'provider:name'"))?;
        (Provider::parse(p)?, mid.to_string())
    } else {
        let decision = ai_route(&cfg, "refactor", "refactor")?;
        (decision.provider, decision.model)
    };

    // Resolve session
    let session_name: Option<String> = if args.r#continue {
        Some(args.session.clone().unwrap_or_else(|| "default".to_string()))
    } else {
        args.session.clone()
    };

    // Load session history for context
    let history = if let Some(ref sname) = session_name {
        load_session(sname, Some(20)).unwrap_or_default()
    } else {
        vec![]
    };
    let history_prefix = if history.is_empty() {
        String::new()
    } else {
        let hist_text: String = history.iter().map(|m| {
            format!("[{}]: {}", m.role, m.content.trim())
        }).collect::<Vec<_>>().join("\n");
        format!("## Prior context\n\n{}\n\n", hist_text)
    };

    let task = format!(
        "{}Refactor `{}` with this intent: {}\n\n\
        Follow this workflow:\n\
        1. Read the file with `ore-cat` and any related files with `ore-neighbors`.\n\
        2. Plan the refactor (list steps before executing).\n\
        3. Use `ore-backup` before every edit.\n\
        4. Apply edits with `ore-patch` or `ore-replace`.\n\
        5. Run `ore-verify` (or language-specific compile) after each step.\n\
        6. If verification fails, use `ore-restore` and revise.\n\
        7. Report the final state and any files touched.\n\n\
        You have {} iterations.",
        history_prefix, args.file.display(), args.intent, args.max_iterations
    );

    let renderer = if args.quiet { Renderer::Silent } else { Renderer::Cli };
    let (tx, rx) = channel::<AiEvent>();
    let handle = std::thread::spawn(move || { while let Ok(ev) = rx.recv() { render(&ev, &renderer); } });

    let _ = tx.send(AiEvent::RouterChose {
        provider: provider.as_str().to_string(),
        model: model.clone(),
        reason: if args.model.is_some() { "user override".to_string() } else { format!("heuristic:{}", cfg.cost_mode) },
        estimated_cost_usd: 0.0,
    });

    let system = get_prompt("refactor").unwrap_or_else(|_| "You are a precise refactor agent.".to_string());
    let tools = builtin_tools();
    let agent_cfg = AgentConfig {
        provider, model: model.clone(),
        max_iterations: args.max_iterations,
        auto_approve_destructive: args.auto,
        task_label: "refactor".to_string(),
    };

    let tx_clone = tx.clone();
    let result = run_agent(&system, &task, &tools, &cfg, &agent_cfg, Some(tx_clone));
    drop(tx);
    let _ = handle.join();

    match result {
        Ok((answer, _, _, _, _)) => {
            if let Some(ref sname) = session_name {
                let _ = append(sname, "user", &format!("refactor {} — {}", args.file.display(), args.intent));
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
