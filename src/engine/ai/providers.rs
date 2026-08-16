use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::engine::ai::config::load as load_cfg;
use crate::engine::ai::events::AiEvent;
use crate::engine::ai::keys::{get_key, Provider};
use crate::engine::ai::models::{augment_cost, ModelInfo};
use crate::engine::ai::usage::{add_process_cost, process_total_cost};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,   // "system" | "user" | "assistant" | "tool"
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub provider: Provider,
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stream: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GenerateResult {
    pub text: String,
    pub tokens_in: u32,
    pub tokens_out: u32,
    pub cost_usd: f64,
    pub duration_ms: u128,
}

static RATE_LIMIT_STATE: OnceLock<Mutex<HashMap<String, VecDeque<Instant>>>> = OnceLock::new();

fn rate_limit_state() -> &'static Mutex<HashMap<String, VecDeque<Instant>>> {
    RATE_LIMIT_STATE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub async fn list_models(provider: Provider) -> Result<Vec<ModelInfo>> {
    let client = build_client()?;
    let now = chrono::Local::now().timestamp();
    let name = provider.as_str().to_string();
    let raw_ids: Vec<String> = match provider {
        Provider::OpenAI => fetch_openai_like(&client, "https://api.openai.com/v1/models", &keyed(provider)?).await?,
        Provider::Groq => fetch_openai_like(&client, "https://api.groq.com/openai/v1/models", &keyed(provider)?).await?,
        Provider::OpenRouter => fetch_openai_like(&client, "https://openrouter.ai/api/v1/models", &keyed(provider)?).await?,
        Provider::DeepSeek => fetch_openai_like(&client, "https://api.deepseek.com/v1/models", &keyed(provider)?).await?,
        Provider::Mistral => fetch_openai_like(&client, "https://api.mistral.ai/v1/models", &keyed(provider)?).await?,
        Provider::Anthropic => fetch_anthropic(&client, &keyed(provider)?).await?,
        Provider::Google => fetch_google(&client, &keyed(provider)?).await?,
        Provider::Ollama => fetch_ollama(&client).await?,
        Provider::LmStudio => fetch_lmstudio(&client).await?,
    };
    let mut out: Vec<ModelInfo> = Vec::with_capacity(raw_ids.len());
    for id in raw_ids {
        let (in_c, out_c, ctx, caps) = augment_cost(&name, &id);
        out.push(ModelInfo {
            provider: name.clone(),
            id,
            context_window: ctx,
            input_cost_per_1m: in_c,
            output_cost_per_1m: out_c,
            capabilities: if caps.is_empty() { vec!["chat".to_string()] } else { caps },
            cached_at: now,
        });
    }
    Ok(out)
}

fn keyed(provider: Provider) -> Result<String> {
    get_key(provider).with_context(|| format!("No API key for {}. Register with `ore ai-keys register {} <key>` or set {}",
        provider.as_str(),
        provider.as_str(),
        provider.env_var().unwrap_or("(no env)")))
}

fn build_client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .build()?)
}

async fn throttle_provider(provider: Provider) -> Result<()> {
    let cfg = load_cfg().unwrap_or_default();
    let rpm = cfg.rate_limits.get(provider.as_str()).copied().unwrap_or_else(|| {
        if provider.as_str() == "groq" { 30 } else { 0 }
    });
    if rpm == 0 {
        return Ok(());
    }

    loop {
        let wait_for = {
            let now = Instant::now();
            let mut state = rate_limit_state()
                .lock()
                .map_err(|_| anyhow::anyhow!("rate limit mutex poisoned"))?;
            let queue = state.entry(provider.as_str().to_string()).or_default();

            while let Some(front) = queue.front() {
                if now.duration_since(*front) >= Duration::from_secs(60) {
                    queue.pop_front();
                } else {
                    break;
                }
            }

            if queue.len() < rpm as usize {
                queue.push_back(now);
                None
            } else {
                let oldest = *queue.front().unwrap();
                Some(Duration::from_secs(60).saturating_sub(now.duration_since(oldest)) + Duration::from_millis(50))
            }
        };

        if let Some(delay) = wait_for {
            tokio::time::sleep(delay).await;
        } else {
            return Ok(());
        }
    }
}

async fn fetch_openai_like(client: &reqwest::Client, url: &str, key: &str) -> Result<Vec<String>> {
    let resp = client.get(url).bearer_auth(key).send().await?.error_for_status()?;
    let json: serde_json::Value = resp.json().await?;
    let arr = json.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(arr.iter().filter_map(|item| item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
}

async fn fetch_anthropic(client: &reqwest::Client, key: &str) -> Result<Vec<String>> {
    let resp = client.get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send().await?.error_for_status()?;
    let json: serde_json::Value = resp.json().await?;
    let arr = json.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(arr.iter().filter_map(|i| i.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
}

async fn fetch_google(client: &reqwest::Client, key: &str) -> Result<Vec<String>> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", key);
    let resp = client.get(&url).send().await?.error_for_status()?;
    let json: serde_json::Value = resp.json().await?;
    let arr = json.get("models").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(arr.iter().filter_map(|i| {
        i.get("name").and_then(|v| v.as_str()).map(|s| s.trim_start_matches("models/").to_string())
    }).collect())
}

async fn fetch_ollama(client: &reqwest::Client) -> Result<Vec<String>> {
    let resp = client.get("http://localhost:11434/api/tags").send().await;
    let resp = match resp {
        Ok(r) => r,
        Err(_) => return Ok(vec![]),
    };
    let json: serde_json::Value = resp.json().await.unwrap_or_default();
    Ok(json.get("models").and_then(|v| v.as_array()).cloned().unwrap_or_default()
        .iter().filter_map(|m| m.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
}

async fn fetch_lmstudio(client: &reqwest::Client) -> Result<Vec<String>> {
    let resp = client.get("http://localhost:1234/v1/models").send().await;
    let resp = match resp {
        Ok(r) => r,
        Err(_) => return Ok(vec![]),
    };
    let json: serde_json::Value = resp.json().await.unwrap_or_default();
    let arr = json.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    Ok(arr.iter().filter_map(|i| i.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())).collect())
}

/// Non-streaming generation. All providers unified into the OpenAI-like schema
/// via provider-specific adapters.
pub async fn generate(req: GenerateRequest, tx: Option<Sender<AiEvent>>) -> Result<GenerateResult> {
    let start = std::time::Instant::now();
    let client = build_client()?;

    let cfg = load_cfg().unwrap_or_default();
    let prompt_chars: usize = req.messages.iter().map(|m| m.content.len()).sum();
    let est_in_tokens = (prompt_chars / 4).max(1) as u32;
    let est_out_tokens = req.max_tokens;
    let (in_cost, out_cost, _, _) = augment_cost(req.provider.as_str(), &req.model);
    let estimated_call_cost = ((est_in_tokens as f64) / 1_000_000.0) * in_cost.unwrap_or(0.0)
        + ((est_out_tokens as f64) / 1_000_000.0) * out_cost.unwrap_or(0.0);

    let spent_so_far = process_total_cost();
    if cfg.session_budget_usd > 0.0 {
        if spent_so_far >= cfg.session_budget_usd {
            anyhow::bail!(
                "Session budget exceeded: spent ${:.5} / cap ${:.5} in this process.",
                spent_so_far,
                cfg.session_budget_usd
            );
        }
        if spent_so_far + estimated_call_cost > cfg.session_budget_usd {
            anyhow::bail!(
                "This call would exceed the session budget: spent ${:.5} + estimated ${:.5} > cap ${:.5}.",
                spent_so_far,
                estimated_call_cost,
                cfg.session_budget_usd
            );
        }
    }
    if cfg.call_budget_usd > 0.0 && estimated_call_cost > cfg.call_budget_usd {
        anyhow::bail!(
            "Estimated call cost ${:.5} exceeds call budget cap ${:.5}. Choose a cheaper model or reduce input size.",
            estimated_call_cost,
            cfg.call_budget_usd
        );
    }

    throttle_provider(req.provider).await?;

    if let Some(t) = &tx {
        let _ = t.send(AiEvent::Sending {
            provider: req.provider.as_str().to_string(),
            model: req.model.clone(),
            prompt_chars,
        });
    }

    let result = match req.provider {
        Provider::OpenAI => openai_generate(&client, "https://api.openai.com/v1/chat/completions", &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::Groq => openai_generate(&client, "https://api.groq.com/openai/v1/chat/completions", &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::OpenRouter => openai_generate(&client, "https://openrouter.ai/api/v1/chat/completions", &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::DeepSeek => openai_generate(&client, "https://api.deepseek.com/v1/chat/completions", &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::Mistral => openai_generate(&client, "https://api.mistral.ai/v1/chat/completions", &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::LmStudio => openai_generate(&client, "http://localhost:1234/v1/chat/completions", "not-needed", &req, tx.as_ref()).await?,
        Provider::Ollama => ollama_generate(&client, &req, tx.as_ref()).await?,
        Provider::Anthropic => anthropic_generate(&client, &keyed(req.provider)?, &req, tx.as_ref()).await?,
        Provider::Google => google_generate(&client, &keyed(req.provider)?, &req, tx.as_ref()).await?,
    };

    let duration_ms = start.elapsed().as_millis();
    let (in_cost, out_cost, _, _) = augment_cost(req.provider.as_str(), &req.model);
    let cost = ((result.tokens_in as f64) / 1_000_000.0) * in_cost.unwrap_or(0.0)
             + ((result.tokens_out as f64) / 1_000_000.0) * out_cost.unwrap_or(0.0);

    let final_result = GenerateResult { duration_ms, cost_usd: cost, ..result };
    add_process_cost(final_result.cost_usd);
    Ok(final_result)
}

/// Like \`generate\` but injects an image into the last user message.
/// Automatically upgrades the model to a vision-capable sibling when needed.
pub async fn generate_with_vision(
    mut req: GenerateRequest,
    image_path: &std::path::Path,
    tx: Option<Sender<AiEvent>>,
) -> Result<GenerateResult> {
    let vision_model = ensure_vision_model(req.provider.as_str(), &req.model)?;
    if vision_model != req.model {
        if let Some(t) = &tx {
            let _ = t.send(AiEvent::Error {
                message: format!(
                    "model '{}' doesn't support vision — upgrading to '{}'",
                    req.model, vision_model
                ),
            });
        }
        req.model = vision_model;
    }

    let (mime, b64) = encode_image_to_base64(image_path)?;

    let start = std::time::Instant::now();
    let client = build_client()?;

    let cfg = load_cfg().unwrap_or_default();
    let prompt_chars: usize = req.messages.iter().map(|m| m.content.len()).sum();
    let est_in_tokens = (prompt_chars / 4).max(1) as u32;
    let (in_cost, out_cost, _, _) = augment_cost(req.provider.as_str(), &req.model);
    let estimated_call_cost = ((est_in_tokens as f64) / 1_000_000.0) * in_cost.unwrap_or(0.0)
        + ((req.max_tokens as f64) / 1_000_000.0) * out_cost.unwrap_or(0.0);

    let spent_so_far = process_total_cost();
    if cfg.session_budget_usd > 0.0 && spent_so_far + estimated_call_cost > cfg.session_budget_usd {
        anyhow::bail!("This call would exceed the session budget.");
    }
    if cfg.call_budget_usd > 0.0 && estimated_call_cost > cfg.call_budget_usd {
        anyhow::bail!("Estimated call cost exceeds call budget cap.");
    }

    throttle_provider(req.provider).await?;

    if let Some(t) = &tx {
        let _ = t.send(AiEvent::Sending {
            provider: req.provider.as_str().to_string(),
            model: req.model.clone(),
            prompt_chars,
        });
    }

    let result = match req.provider {
        Provider::OpenAI | Provider::Groq | Provider::OpenRouter | Provider::DeepSeek | Provider::Mistral | Provider::LmStudio => {
            let url = match req.provider {
                Provider::OpenAI => "https://api.openai.com/v1/chat/completions",
                Provider::Groq => "https://api.groq.com/openai/v1/chat/completions",
                Provider::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
                Provider::DeepSeek => "https://api.deepseek.com/v1/chat/completions",
                Provider::Mistral => "https://api.mistral.ai/v1/chat/completions",
                _ => "http://localhost:1234/v1/chat/completions",
            };
            let key = keyed(req.provider).unwrap_or_else(|_| "not-needed".to_string());
            let vision_messages = inject_vision_openai(&req.messages, &mime, &b64);
            let body = serde_json::json!({
                "model": req.model,
                "messages": vision_messages,
                "max_tokens": req.max_tokens,
                "temperature": req.temperature,
                "stream": false,
            });
            let mut b = client.post(url).json(&body);
            if !key.is_empty() && key != "not-needed" { b = b.bearer_auth(&key); }
            let resp = b.send().await?.error_for_status()?;
            let json: serde_json::Value = resp.json().await?;
            let text = json.pointer("/choices/0/message/content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tokens_in = json.pointer("/usage/prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let tokens_out = json.pointer("/usage/completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if let Some(t) = &tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
            GenerateResult { text, tokens_in, tokens_out, ..Default::default() }
        }
        Provider::Anthropic => {
            let key = keyed(req.provider)?;
            let (system_msgs, _): (Vec<_>, Vec<_>) = req.messages.iter().cloned().partition(|m| m.role == "system");
            let system = system_msgs.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n\n");
            let vision_messages = inject_vision_anthropic(&req.messages, &mime, &b64);
            let body = serde_json::json!({
                "model": req.model,
                "max_tokens": req.max_tokens,
                "temperature": req.temperature,
                "system": system,
                "messages": vision_messages,
                "stream": false,
            });
            let resp = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send().await?.error_for_status()?;
            let json: serde_json::Value = resp.json().await?;
            let text = json.pointer("/content/0/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tokens_in = json.pointer("/usage/input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let tokens_out = json.pointer("/usage/output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if let Some(t) = &tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
            GenerateResult { text, tokens_in, tokens_out, ..Default::default() }
        }
        Provider::Google => {
            let key = keyed(req.provider)?;
            let vision_contents = inject_vision_google(&req.messages, &mime, &b64);
            let system_instruction = req.messages.iter().find(|m| m.role == "system").map(|m| {
                serde_json::json!({ "parts": [{ "text": m.content }]})
            });
            let mut body = serde_json::json!({
                "contents": vision_contents,
                "generationConfig": {
                    "maxOutputTokens": req.max_tokens,
                    "temperature": req.temperature,
                }
            });
            if let Some(si) = system_instruction { body["systemInstruction"] = si; }
            let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", req.model, key);
            let resp = client.post(&url).json(&body).send().await?.error_for_status()?;
            let json: serde_json::Value = resp.json().await?;
            let text = json.pointer("/candidates/0/content/parts/0/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tokens_in = json.pointer("/usageMetadata/promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let tokens_out = json.pointer("/usageMetadata/candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            if let Some(t) = &tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
            GenerateResult { text, tokens_in, tokens_out, ..Default::default() }
        }
        Provider::Ollama => {
            anyhow::bail!("Vision is not supported for Ollama in this build.");
        }
    };

    let duration_ms = start.elapsed().as_millis();
    let (in_c, out_c, _, _) = augment_cost(req.provider.as_str(), &req.model);
    let cost = ((result.tokens_in as f64) / 1_000_000.0) * in_c.unwrap_or(0.0)
             + ((result.tokens_out as f64) / 1_000_000.0) * out_c.unwrap_or(0.0);
    let final_result = GenerateResult { duration_ms, cost_usd: cost, ..result };
    add_process_cost(final_result.cost_usd);
    Ok(final_result)
}

// -------- OpenAI-compatible (OpenAI, Groq, OpenRouter, DeepSeek, Mistral, LM Studio) --------
async fn openai_generate(client: &reqwest::Client, url: &str, key: &str, req: &GenerateRequest, tx: Option<&Sender<AiEvent>>) -> Result<GenerateResult> {
    let body = serde_json::json!({
        "model": req.model,
        "messages": req.messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
        "stream": req.stream,
    });

    // Retry loop with exponential backoff on 429/5xx
    let mut delay_ms: u64 = 500;
    let max_attempts = 4;
    for attempt in 1..=max_attempts {
        let mut b = client.post(url).json(&body);
        if !key.is_empty() && key != "not-needed" { b = b.bearer_auth(key); }

        if !req.stream {
            match b.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status == 429 || status.is_server_error() {
                        if attempt < max_attempts {
                            if let Some(t) = tx {
                                let _ = t.send(AiEvent::Error {
                                    message: format!("HTTP {} — retrying in {}ms (attempt {}/{})", status, delay_ms, attempt, max_attempts)
                                });
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            delay_ms = (delay_ms * 2).min(8000);
                            continue;
                        }
                    }
                    if status == 413 {
                        anyhow::bail!("Payload too large (413). Reduce input size, use `ore condense` on the file, or switch to a larger-context model.");
                    }
                    let resp = resp.error_for_status()?;
                    let json: serde_json::Value = resp.json().await?;
                    let text = json.pointer("/choices/0/message/content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let tokens_in = json.pointer("/usage/prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let tokens_out = json.pointer("/usage/completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
                    return Ok(GenerateResult { text, tokens_in, tokens_out, ..Default::default() });
                }
                Err(e) if attempt < max_attempts => {
                    if let Some(t) = tx {
                        let _ = t.send(AiEvent::Error {
                            message: format!("network error — retrying in {}ms: {}", delay_ms, e)
                        });
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(8000);
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        } else {
            // Streaming path
            match b.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status == 429 || status.is_server_error() {
                        if attempt < max_attempts {
                            if let Some(t) = tx {
                                let _ = t.send(AiEvent::Error {
                                    message: format!("HTTP {} — retrying in {}ms (attempt {}/{})", status, delay_ms, attempt, max_attempts)
                                });
                            }
                            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            delay_ms = (delay_ms * 2).min(8000);
                            continue;
                        }
                    }
                    if status == 413 {
                        anyhow::bail!("Payload too large (413). Reduce input size, use `ore condense` on the file, or switch to a larger-context model.");
                    }
                    let resp = resp.error_for_status()?;
                    let mut stream = resp.bytes_stream().eventsource();
                    let mut collected = String::new();
                    let mut tokens_out: u32 = 0;
                    let mut tokens_in: u32 = 0;
                    if let Some(t) = tx { let _ = t.send(AiEvent::Thinking); }
                    while let Some(event) = stream.next().await {
                        let ev = match event { Ok(e) => e, Err(_) => break };
                        let data = ev.data;
                        if data.trim() == "[DONE]" { break; }
                        let json: serde_json::Value = match serde_json::from_str(&data) { Ok(v) => v, Err(_) => continue };
                        if let Some(text) = json.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
                            if !text.is_empty() {
                                collected.push_str(text);
                                if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.to_string() }); }
                            }
                        }
                        if let Some(u_in) = json.pointer("/usage/prompt_tokens").and_then(|v| v.as_u64()) { tokens_in = u_in as u32; }
                        if let Some(u_out) = json.pointer("/usage/completion_tokens").and_then(|v| v.as_u64()) { tokens_out = u_out as u32; }
                    }
                    if tokens_out == 0 { tokens_out = (collected.len() / 4) as u32; }
                    if tokens_in == 0 {
                        let prompt_chars: usize = req.messages.iter().map(|m| m.content.len()).sum();
                        tokens_in = (prompt_chars / 4) as u32;
                    }
                    return Ok(GenerateResult { text: collected, tokens_in, tokens_out, ..Default::default() });
                }
                Err(e) if attempt < max_attempts => {
                    if let Some(t) = tx {
                        let _ = t.send(AiEvent::Error {
                            message: format!("network error — retrying in {}ms: {}", delay_ms, e)
                        });
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                    delay_ms = (delay_ms * 2).min(8000);
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
    anyhow::bail!("exhausted retries")
}

// -------- Anthropic --------
async fn anthropic_generate(client: &reqwest::Client, key: &str, req: &GenerateRequest, tx: Option<&Sender<AiEvent>>) -> Result<GenerateResult> {
    // Split system message; Anthropic wants it separate
    let (system_msgs, chat_msgs): (Vec<_>, Vec<_>) = req.messages.iter().cloned().partition(|m| m.role == "system");
    let system = system_msgs.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n\n");
    let messages_json = chat_msgs.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>();

    let body = serde_json::json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
        "system": system,
        "messages": messages_json,
        "stream": req.stream,
    });
    let b = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&body);

    if !req.stream {
        let resp = b.send().await?.error_for_status()?;
        let json: serde_json::Value = resp.json().await?;
        let text = json.pointer("/content/0/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tokens_in = json.pointer("/usage/input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let tokens_out = json.pointer("/usage/output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
        return Ok(GenerateResult { text, tokens_in, tokens_out, ..Default::default() });
    }

    let resp = b.send().await?.error_for_status()?;
    let mut stream = resp.bytes_stream().eventsource();
    let mut collected = String::new();
    let mut tokens_in: u32 = 0;
    let mut tokens_out: u32 = 0;
    if let Some(t) = tx { let _ = t.send(AiEvent::Thinking); }
    while let Some(event) = stream.next().await {
        let ev = match event { Ok(e) => e, Err(_) => break };
        let data = ev.data;
        if data.trim().is_empty() { continue; }
        let json: serde_json::Value = match serde_json::from_str(&data) { Ok(v) => v, Err(_) => continue };
        if let Some(t_type) = json.get("type").and_then(|v| v.as_str()) {
            match t_type {
                "content_block_delta" => {
                    if let Some(text) = json.pointer("/delta/text").and_then(|v| v.as_str()) {
                        collected.push_str(text);
                        if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.to_string() }); }
                    }
                }
                "message_start" => {
                    tokens_in = json.pointer("/message/usage/input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                }
                "message_delta" => {
                    tokens_out = json.pointer("/usage/output_tokens").and_then(|v| v.as_u64()).unwrap_or(tokens_out as u64) as u32;
                }
                _ => {}
            }
        }
    }
    if tokens_out == 0 { tokens_out = (collected.len() / 4) as u32; }
    Ok(GenerateResult { text: collected, tokens_in, tokens_out, ..Default::default() })
}

// -------- Google --------
async fn google_generate(client: &reqwest::Client, key: &str, req: &GenerateRequest, tx: Option<&Sender<AiEvent>>) -> Result<GenerateResult> {
    let contents: Vec<serde_json::Value> = req.messages.iter().filter(|m| m.role != "system").map(|m| {
        let role = if m.role == "assistant" { "model" } else { "user" };
        serde_json::json!({ "role": role, "parts": [{ "text": m.content }]})
    }).collect();
    let system_instruction = req.messages.iter().find(|m| m.role == "system").map(|m| {
        serde_json::json!({ "parts": [{ "text": m.content }]})
    });

    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": req.max_tokens,
            "temperature": req.temperature,
        }
    });
    if let Some(si) = system_instruction {
        body["systemInstruction"] = si;
    }

    // Google doesn't do proper SSE for genai in the same way; use non-streaming for v1
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", req.model, key);
    let resp = client.post(&url).json(&body).send().await?.error_for_status()?;
    let json: serde_json::Value = resp.json().await?;
    let text = json.pointer("/candidates/0/content/parts/0/text").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tokens_in = json.pointer("/usageMetadata/promptTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let tokens_out = json.pointer("/usageMetadata/candidatesTokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
    Ok(GenerateResult { text, tokens_in, tokens_out, ..Default::default() })
}

// -------- Ollama --------
async fn ollama_generate(client: &reqwest::Client, req: &GenerateRequest, tx: Option<&Sender<AiEvent>>) -> Result<GenerateResult> {
    let body = serde_json::json!({
        "model": req.model,
        "messages": req.messages.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
        "stream": req.stream,
        "options": { "temperature": req.temperature, "num_predict": req.max_tokens },
    });
    if !req.stream {
        let resp = client.post("http://localhost:11434/api/chat").json(&body).send().await?.error_for_status()?;
        let json: serde_json::Value = resp.json().await?;
        let text = json.pointer("/message/content").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tokens_in = json.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let tokens_out = json.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: text.clone() }); }
        return Ok(GenerateResult { text, tokens_in, tokens_out, ..Default::default() });
    }
    // Streaming JSON-per-line (not SSE)
    let mut resp = client.post("http://localhost:11434/api/chat").json(&body).send().await?.error_for_status()?;
    let mut collected = String::new();
    let mut tokens_in: u32 = 0;
    let mut tokens_out: u32 = 0;
    if let Some(t) = tx { let _ = t.send(AiEvent::Thinking); }
    while let Some(chunk) = resp.chunk().await? {
        let bytes = chunk.to_vec();
        let text = String::from_utf8_lossy(&bytes).to_string();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let json: serde_json::Value = match serde_json::from_str(line) { Ok(v) => v, Err(_) => continue };
            if let Some(part) = json.pointer("/message/content").and_then(|v| v.as_str()) {
                if !part.is_empty() {
                    collected.push_str(part);
                    if let Some(t) = tx { let _ = t.send(AiEvent::Token { text: part.to_string() }); }
                }
            }
            if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                tokens_in = json.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                tokens_out = json.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                break;
            }
        }
    }
    if tokens_out == 0 { tokens_out = (collected.len() / 4) as u32; }
    Ok(GenerateResult { text: collected, tokens_in, tokens_out, ..Default::default() })
}

use eventsource_stream::Eventsource;

// -------- Vision helpers --------

/// Known vision-capable models per provider.
pub fn is_vision_model(provider: &str, model: &str) -> bool {
    match provider {
        "openai" => matches!(model, "gpt-4o" | "gpt-4o-mini" | "gpt-4-turbo"),
        "anthropic" => model.contains("claude-sonnet") || model.contains("claude-haiku") || model.contains("claude-opus"),
        "google" => model.contains("gemini"),
        "groq" => model.contains("llama-4") || model.contains("scout") || model.contains("maverick"),
        _ => false,
    }
}

/// Upgrade a model to a vision-capable sibling if it doesn't support vision.
/// Returns Err if no vision model is available for this provider.
pub fn ensure_vision_model(provider: &str, model: &str) -> Result<String> {
    if is_vision_model(provider, model) {
        return Ok(model.to_string());
    }
    let fallback = match provider {
        "openai" => Some("gpt-4o"),
        "anthropic" => Some("claude-sonnet-4-5"),
        "google" => Some("gemini-2.5-flash"),
        "groq" => Some("meta-llama/llama-4-scout-17b-16e-instruct"),
        _ => None,
    };
    fallback
        .map(|m| m.to_string())
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' has no known vision-capable model.", provider))
}

/// Encode an image file to a base64 data URL.
pub fn encode_image_to_base64(path: &std::path::Path) -> Result<(String, String)> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading image: {}", path.display()))?;
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok((mime.to_string(), b64))
}

/// Build a messages array that injects an image into the last user message.
/// Handles OpenAI-compatible format (OpenAI, Groq) and Anthropic separately.
pub fn inject_vision_openai(
    messages: &[ChatMessage],
    mime: &str,
    b64: &str,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let last_user_idx = messages.iter().rposition(|m| m.role == "user");
    for (i, m) in messages.iter().enumerate() {
        if Some(i) == last_user_idx {
            out.push(serde_json::json!({
                "role": "user",
                "content": [
                    {
                        "type": "image_url",
                        "image_url": {
                            "url": format!("data:{};base64,{}", mime, b64)
                        }
                    },
                    {
                        "type": "text",
                        "text": m.content
                    }
                ]
            }));
        } else {
            out.push(serde_json::json!({"role": m.role, "content": m.content}));
        }
    }
    out
}

pub fn inject_vision_anthropic(
    messages: &[ChatMessage],
    mime: &str,
    b64: &str,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let last_user_idx = messages.iter().rposition(|m| m.role == "user" || m.role == "assistant");
    for (i, m) in messages.iter().enumerate() {
        if m.role == "system" { continue; } // system handled separately in anthropic_generate
        if Some(i) == last_user_idx && m.role == "user" {
            out.push(serde_json::json!({
                "role": "user",
                "content": [
                    {
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": mime,
                            "data": b64
                        }
                    },
                    {
                        "type": "text",
                        "text": m.content
                    }
                ]
            }));
        } else {
            out.push(serde_json::json!({"role": m.role, "content": m.content}));
        }
    }
    out
}

pub fn inject_vision_google(
    messages: &[ChatMessage],
    mime: &str,
    b64: &str,
) -> Vec<serde_json::Value> {
    let mut out: Vec<serde_json::Value> = Vec::new();
    let last_user_idx = messages.iter().rposition(|m| m.role == "user");
    for (i, m) in messages.iter().enumerate() {
        if m.role == "system" { continue; }
        let role = if m.role == "assistant" { "model" } else { "user" };
        if Some(i) == last_user_idx && m.role == "user" {
            out.push(serde_json::json!({
                "role": role,
                "parts": [
                    {
                        "inline_data": {
                            "mime_type": mime,
                            "data": b64
                        }
                    },
                    { "text": m.content }
                ]
            }));
        } else {
            out.push(serde_json::json!({"role": role, "parts": [{"text": m.content}]}));
        }
    }
    out
}

