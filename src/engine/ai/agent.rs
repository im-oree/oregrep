// Agent loop.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

use crate::engine::ai::config::AiConfig;
use crate::engine::ai::events::AiEvent;
use crate::engine::ai::keys::Provider;
use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
use crate::engine::ai::tools::{execute, find_tool, ToolSpec};

/// Simplified agent-side representation of a tool call the LLM wants us to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedCall {
    pub name: String,
    pub args: serde_json::Value,
}

/// A single agent turn's outcome. (Reserved for richer turn reporting; run_agent
/// currently returns an aggregated tuple.)
#[allow(dead_code)]
pub struct AgentTurn {
    pub assistant_text: String,
    pub proposed_calls: Vec<ProposedCall>,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub cost_usd: f64,
    pub duration_ms: u128,
}

pub struct AgentConfig {
    pub provider: Provider,
    pub model: String,
    pub max_iterations: usize,
    pub auto_approve_destructive: bool,
    pub task_label: String,
}

/// Run an agent loop:
///   system prompt (with tool descriptions injected) → user prompt → LLM →
///   parse tool calls from response → execute → feed results back → repeat until no calls or budget hit.
///
/// Tool call convention (v1, JSON in text): the model emits a fenced block like:
///   ```tool_call
///   { "name": "ore-find", "args": { "pattern": "TODO", "path": "src" } }
///   ```
/// Multiple calls can be emitted in one turn.
pub fn run_agent(
    system_prompt: &str,
    user_prompt: &str,
    tools: &[ToolSpec],
    ai_cfg: &AiConfig,
    agent_cfg: &AgentConfig,
    tx: Option<Sender<AiEvent>>,
) -> Result<(String, u32, u32, f64, u128)> {
    let tool_desc = describe_tools(tools);
    let full_system = format!(
        "{}\n\n---\nYou have access to the following tools. To call a tool, emit a fenced block:\n\n```tool_call\n{{\"name\": \"tool-name\", \"args\": {{...}}}}\n```\n\nYou may call multiple tools per turn (emit multiple blocks). After tools run, you'll get results and can call more tools or provide a final answer. When done, just answer without any tool_call blocks.\n\n{}",
        system_prompt, tool_desc
    );

    let mut messages: Vec<ChatMessage> = vec![
        ChatMessage { role: "system".to_string(), content: full_system },
        ChatMessage { role: "user".to_string(), content: user_prompt.to_string() },
    ];

    let mut total_tokens_in: u32 = 0;
    let mut total_tokens_out: u32 = 0;
    let mut total_cost: f64 = 0.0;
    let mut total_duration: u128 = 0;
    let mut total_tool_calls: usize = 0;

    let rt = crate::engine::ai::runtime::build_runtime()?;

    let mut final_text = String::new();
    for iteration in 1..=agent_cfg.max_iterations {
        if let Some(t) = &tx {
            let _ = t.send(AiEvent::AgentStep { iteration, description: format!("iteration {}", iteration) });
        }
        let req = GenerateRequest {
            provider: agent_cfg.provider,
            model: agent_cfg.model.clone(),
            messages: messages.clone(),
            max_tokens: ai_cfg.max_output_tokens,
            temperature: ai_cfg.temperature,
            stream: false, // Streaming tool-call parsing is trickier; do non-streaming per iteration
        };
        let tx_clone = tx.clone();
        let turn_result = rt.block_on(async move { generate(req, tx_clone).await })?;

        total_tokens_in += turn_result.tokens_in;
        total_tokens_out += turn_result.tokens_out;
        total_cost += turn_result.cost_usd;
        total_duration += turn_result.duration_ms;

        // Parse tool calls from the response
        let (visible_text, calls) = parse_tool_calls(&turn_result.text);
        // Do NOT emit visible text here. The non-streaming generate() already emits the
        // full response as one Token event (which is exactly the output we want per turn),
        // and the final answer prints via that same single Token event on the last turn.
        // Emitting visible_text again here was the source of the double-print bug.

        if calls.is_empty() {
            final_text = visible_text.clone();
            let _ = crate::engine::ai::usage::record(
                agent_cfg.provider.as_str(), &agent_cfg.model,
                total_tokens_in, total_tokens_out, total_cost, total_duration,
                Some(&agent_cfg.task_label),
            );
            if let Some(t) = &tx {
                let _ = t.send(AiEvent::AgentDone { iterations: iteration, tool_calls: total_tool_calls });
            }
            return Ok((final_text, total_tokens_in, total_tokens_out, total_cost, total_duration));
        }

        // Push assistant message (with the raw tool_call blocks) into history
        messages.push(ChatMessage { role: "assistant".to_string(), content: turn_result.text.clone() });

        // Execute each tool call
        let mut tool_output_message = String::new();
        for call in &calls {
            if let Some(t) = &tx {
                let _ = t.send(AiEvent::ToolCallRequested { name: call.name.clone(), args: call.args.clone() });
            }
            let tool = match find_tool(tools, &call.name) {
                Some(t) => t,
                None => {
                    let msg = format!("[tool `{}` not found]", call.name);
                    if let Some(tx) = &tx {
                        let _ = tx.send(AiEvent::Error { message: msg.clone() });
                    }
                    tool_output_message.push_str(&format!("### {} → ERROR\n{}\n\n", call.name, msg));
                    continue;
                }
            };
            // Destructive safety
            if tool.destructive && !agent_cfg.auto_approve_destructive {
                let msg = format!("[destructive tool `{}` skipped — rerun with --auto to allow]", call.name);
                tool_output_message.push_str(&format!("### {} → SKIPPED (destructive, not auto-approved)\n{}\n\n", call.name, msg));
                if let Some(tx) = &tx {
                    let _ = tx.send(AiEvent::ToolCallResult { name: call.name.clone(), ok: false, preview: "skipped: destructive".to_string() });
                }
                continue;
            }
            match execute(tool, &call.args, tx.as_ref()) {
                Ok(res) => {
                    total_tool_calls += 1;
                    let tag = if res.ok { "OK" } else { "FAIL" };
                    tool_output_message.push_str(&format!("### {} → {} ({}ms)\n```\n{}\n```\n\n", call.name, tag, res.duration_ms, res.output));
                }
                Err(e) => {
                    total_tool_calls += 1;
                    tool_output_message.push_str(&format!("### {} → EXCEPTION\n{}\n\n", call.name, e));
                }
            }
        }
        messages.push(ChatMessage { role: "user".to_string(), content: format!("Tool results:\n\n{}", tool_output_message) });
    }

    // Max iterations reached
    if let Some(t) = &tx {
        let _ = t.send(AiEvent::AgentDone { iterations: agent_cfg.max_iterations, tool_calls: total_tool_calls });
    }
    let _ = crate::engine::ai::usage::record(
        agent_cfg.provider.as_str(), &agent_cfg.model,
        total_tokens_in, total_tokens_out, total_cost, total_duration,
        Some(&agent_cfg.task_label),
    );
    Ok((final_text, total_tokens_in, total_tokens_out, total_cost, total_duration))
}

fn describe_tools(tools: &[ToolSpec]) -> String {
    let mut out = String::from("Available tools:\n\n");
    for t in tools {
        out.push_str(&format!("- **{}** ({}): {}\n  args schema: {}\n\n",
            t.name,
            if t.destructive { "destructive" } else { "read-only" },
            t.description,
            serde_json::to_string(&t.input_schema).unwrap_or_default()
        ));
    }
    out
}

/// Extract tool_call blocks. Returns (visible_text_stripped_of_calls, parsed_calls).
fn parse_tool_calls(text: &str) -> (String, Vec<ProposedCall>) {
    let re = regex::Regex::new(r"(?s)```tool_call\s*\n(.*?)\n```").unwrap();
    let mut calls = Vec::new();
    for cap in re.captures_iter(text) {
        let body = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(body.trim()) {
            let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let args = v.get("args").cloned().unwrap_or(serde_json::json!({}));
            if !name.is_empty() {
                calls.push(ProposedCall { name, args });
            }
        }
    }
    let stripped = re.replace_all(text, "").to_string();
    (stripped, calls)
}
