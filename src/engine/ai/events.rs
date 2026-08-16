use colored::*;
use serde::Serialize;
use std::io::Write;

// Tool-loop variants (ToolCall*, IterationComplete, Verifying, RollingBack) are constructed
// by the AI-2 agent batch; RouterThinking is used by the LLM router. Staged for now.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiEvent {
    RouterThinking { task: String },
    RouterChose { provider: String, model: String, reason: String, estimated_cost_usd: f64 },
    Sending { provider: String, model: String, prompt_chars: usize },
    Thinking,
    Token { text: String },
    ToolCallRequested { name: String, args: serde_json::Value },
    ToolCallExecuting { name: String },
    ToolCallResult { name: String, ok: bool, preview: String },
    IterationComplete { iteration: usize },
    Verifying { command: String },
    RollingBack { reason: String },
    Complete { tokens_in: u32, tokens_out: u32, cost_usd: f64, duration_ms: u128 },
    Error { message: String },
    SearchingWeb { query: String, instance: String },
    SearchFailed { instance: String, reason: String },
    SearchFallback { from: String, to: String },
    SearchFound { count: usize, sources: Vec<String> },
    FetchingUrl { url: String },
    FetchFailed { url: String, reason: String },
    AgentStep { iteration: usize, description: String },
    AgentDone { iterations: usize, tool_calls: usize },
}

/// How to render events for a human.
pub enum Renderer {
    /// Colored, live-updating terminal output
    Cli,
    /// One JSON event per line (for GUIs / scripting)
    Json,
    /// Suppress event chatter (result-only)
    Silent,
}

pub fn render(event: &AiEvent, renderer: &Renderer) {
    match renderer {
        // "Silent" means result-only: suppress chatter but still print the answer (Token).
        Renderer::Silent => match event {
            AiEvent::Token { text } => {
                let mut so = std::io::stdout().lock();
                let _ = write!(so, "{}", text);
                let _ = so.flush();
            }
            _ => {}
        },
        Renderer::Json => {
            let line = serde_json::to_string(event).unwrap_or_default();
            let mut out = std::io::stderr().lock();
            let _ = writeln!(out, "{}", line);
        }
        Renderer::Cli => render_cli(event),
    }
}

fn render_cli(event: &AiEvent) {
    let out = std::io::stderr();
    let mut out = out.lock();
    match event {
        AiEvent::RouterThinking { task } => {
            let _ = writeln!(out, "{} routing → {}", "▸".magenta(), task.dimmed());
        }
        AiEvent::RouterChose { provider, model, reason, estimated_cost_usd } => {
            let _ = writeln!(out, "{} {} {}  {}  {}",
                "▸".magenta(),
                provider.cyan(),
                model.yellow(),
                format!("~${:.4}", estimated_cost_usd).dimmed(),
                reason.dimmed());
        }
        AiEvent::Sending { provider, model, prompt_chars } => {
            let _ = writeln!(out, "{} sending to {}:{}  ({} chars)",
                "▸".cyan(),
                provider.cyan(),
                model.yellow(),
                prompt_chars.to_string().dimmed());
        }
        AiEvent::Thinking => {
            let _ = write!(out, "{} thinking… ", "▸".cyan().dimmed());
            let _ = out.flush();
        }
        AiEvent::Token { text } => {
            // Streamed tokens go to STDOUT so users can redirect the response cleanly
            let mut so = std::io::stdout().lock();
            let _ = write!(so, "{}", text);
            let _ = so.flush();
        }
        AiEvent::ToolCallRequested { name, args } => {
            let arg_short = serde_json::to_string(args).unwrap_or_default();
            let arg_short: String = arg_short.chars().take(120).collect();
            let _ = writeln!(out, "\n{} tool: {}  {}",
                "▸".yellow(),
                name.magenta(),
                arg_short.dimmed());
        }
        AiEvent::ToolCallExecuting { name } => {
            let _ = writeln!(out, "{} executing {}", "▸".yellow(), name.magenta());
        }
        AiEvent::ToolCallResult { name, ok, preview } => {
            let tag = if *ok { "OK".green().bold().to_string() } else { "FAIL".red().bold().to_string() };
            let prev: String = preview.chars().take(200).collect();
            let _ = writeln!(out, "{} {} {}  {}", "▸".yellow(), tag, name.magenta(), prev.dimmed());
        }
        AiEvent::IterationComplete { iteration } => {
            let _ = writeln!(out, "{} iteration {} complete", "▸".dimmed(), iteration.to_string().dimmed());
        }
        AiEvent::Verifying { command } => {
            let _ = writeln!(out, "{} verifying: {}", "▸".cyan(), command.dimmed());
        }
        AiEvent::RollingBack { reason } => {
            let _ = writeln!(out, "{} rolling back: {}", "↺".yellow(), reason.dimmed());
        }
        AiEvent::Complete { tokens_in, tokens_out, cost_usd, duration_ms } => {
            let _ = writeln!(out, "\n{} {}↑ {}↓  ${:.5}  {}ms",
                "▸".green().bold(),
                tokens_in.to_string().yellow(),
                tokens_out.to_string().yellow(),
                cost_usd,
                duration_ms.to_string().dimmed());
        }
        AiEvent::Error { message } => {
            let _ = writeln!(out, "{} {}", "✗".red().bold(), message.red());
        }
        AiEvent::SearchingWeb { query, instance } => {
            let _ = writeln!(out, "{} searching '{}'  {}", "▸".cyan(), query.yellow(), format!("via {}", instance).dimmed());
        }
        AiEvent::SearchFailed { instance, reason } => {
            let _ = writeln!(out, "{} search failed on {}: {}", "!".yellow(), instance.cyan(), reason.dimmed());
        }
        AiEvent::SearchFallback { from, to } => {
            let _ = writeln!(out, "{} falling back {} → {}", "↳".magenta(), from.dimmed(), to.yellow());
        }
        AiEvent::SearchFound { count, sources } => {
            let _ = writeln!(out, "{} {} results:", "▸".green(), count.to_string().yellow());
            for s in sources.iter().take(5) {
                let _ = writeln!(out, "    · {}", s.dimmed());
            }
            if sources.len() > 5 {
                let _ = writeln!(out, "    · … +{} more", sources.len() - 5);
            }
        }
        AiEvent::FetchingUrl { url } => {
            let _ = writeln!(out, "{} fetching {}", "▸".cyan(), url.dimmed());
        }
        AiEvent::FetchFailed { url, reason } => {
            let _ = writeln!(out, "{} fetch failed {}: {}", "!".yellow(), url.dimmed(), reason.dimmed());
        }
        AiEvent::AgentStep { iteration, description } => {
            let _ = writeln!(out, "{} step {} — {}", "▸".magenta(), iteration.to_string().yellow(), description);
        }
        AiEvent::AgentDone { iterations, tool_calls } => {
            let _ = writeln!(out, "{} agent complete: {} iterations, {} tool calls", "▸".green().bold(), iterations, tool_calls);
        }
    }
}
