use anyhow::Result;

use crate::engine::ai::config::AiConfig;
use crate::engine::ai::keys::{get_key, registered_providers, Provider};
use crate::engine::ai::models::{load_cached, ModelInfo};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub provider: Provider,
    pub model: String,
    pub reason: String,
    pub estimated_input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_cost_usd: f64,
}

/// Primary entry point. Tries LLM router if configured, falls back to heuristic.
pub fn route(cfg: &AiConfig, prompt: &str, task_class: &str) -> Result<RouteDecision> {
    if !cfg.router_provider.is_empty() {
        if let Some(key) = get_key(Provider::parse(&cfg.router_provider)?) {
            if !key.is_empty() {
                match llm_route(cfg, prompt, task_class) {
                    Ok(d) => return Ok(d),
                    Err(_) => {} // fall through to heuristic
                }
            }
        }
    }
    heuristic_route(cfg, prompt, task_class)
}

/// LLM-based routing: ask the cheapest configured model to pick the best model.
pub fn llm_route(cfg: &AiConfig, prompt: &str, task_class: &str) -> Result<RouteDecision> {
    use crate::engine::ai::providers::{generate, ChatMessage, GenerateRequest};
    use crate::engine::ai::runtime::build_runtime;
    use crate::engine::ai::prompts::get as get_prompt;

    let router_provider = Provider::parse(&cfg.router_provider)?;
    let router_model = if cfg.router_model.is_empty() {
        // default cheapest per provider
        match router_provider {
            Provider::Groq => "llama-3.1-8b-instant".to_string(),
            Provider::OpenAI => "gpt-4o-mini".to_string(),
            Provider::Anthropic => "claude-haiku-4-5".to_string(),
            Provider::Google => "gemini-2.5-flash".to_string(),
            _ => return heuristic_route(cfg, prompt, task_class),
        }
    } else {
        cfg.router_model.clone()
    };

    // Build model table for the router prompt
    let available: Vec<Provider> = registered_providers()?.into_iter().map(|(p, _)| p).collect();
    let mut model_table = String::new();
    for p in &available {
        let cached = load_cached(p.as_str(), cfg.models_cache_ttl_secs).unwrap_or_default();
        let models_to_show: Vec<_> = if cached.is_empty() {
            // show the default model we'd pick
            vec![ModelInfo {
                provider: p.as_str().to_string(),
                id: pick_model(p, &[], cfg, task_class),
                context_window: None,
                input_cost_per_1m: None,
                output_cost_per_1m: None,
                capabilities: vec!["chat".to_string()],
                cached_at: 0,
            }]
        } else {
            cached.into_iter().take(5).collect()
        };
        for m in models_to_show {
            let ctx = m.context_window.map(|c| format!("{}k", c / 1024)).unwrap_or_else(|| "?".to_string());
            let in_c = m.input_cost_per_1m.map(|c| format!("${:.3}", c)).unwrap_or_else(|| "?".to_string());
            let out_c = m.output_cost_per_1m.map(|c| format!("${:.3}", c)).unwrap_or_else(|| "?".to_string());
            let caps = m.capabilities.join(",");
            model_table.push_str(&format!("{}:{} → ctx={} in={} out={} caps={}\n",
                p.as_str(), m.id, ctx, in_c, out_c, caps));
        }
    }

    let prompt_snippet: String = prompt.chars().take(500).collect();
    let est_ctx = (prompt.len() / 4).max(50);

    let system_template = get_prompt("router").unwrap_or_else(|_| {
        "You are ore's model router. Respond in JSON: {\"provider\": \"...\", \"model\": \"...\", \"reason\": \"...\", \"estimated_input_tokens\": 0, \"estimated_output_tokens\": 0}".to_string()
    });
    let system = system_template
        .replace("{{task_class}}", task_class)
        .replace("{{prompt_snippet}}", &prompt_snippet)
        .replace("{{context_estimate}}", &est_ctx.to_string())
        .replace("{{model_table}}", &model_table)
        .replace("{{cost_mode}}", &cfg.cost_mode);

    let req = GenerateRequest {
        provider: router_provider,
        model: router_model,
        messages: vec![
            ChatMessage { role: "system".to_string(), content: system },
            ChatMessage { role: "user".to_string(), content: format!("Route this task: {}", task_class) },
        ],
        max_tokens: 200,
        temperature: 0.0,
        stream: false,
    };

    let rt = build_runtime()?;
    let result = rt.block_on(async move { generate(req, None).await })?;

    // Parse JSON response
    let text = result.text.trim().to_string();
    // Strip markdown fences if present
    let json_str = if let Some(s) = text.strip_prefix("```json") {
        s.trim_end_matches("```").trim()
    } else if let Some(s) = text.strip_prefix("```") {
        s.trim_end_matches("```").trim()
    } else {
        &text
    };

    let v: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| anyhow::anyhow!("LLM router returned invalid JSON: {} — raw: {}", e, json_str))?;

    let provider_str = v.get("provider").and_then(|x| x.as_str()).unwrap_or("");
    let model_str = v.get("model").and_then(|x| x.as_str()).unwrap_or("");
    let reason = v.get("reason").and_then(|x| x.as_str()).unwrap_or("llm-router").to_string();
    let est_in = v.get("estimated_input_tokens").and_then(|x| x.as_u64()).unwrap_or(500) as u32;
    let est_out = v.get("estimated_output_tokens").and_then(|x| x.as_u64()).unwrap_or(500) as u32;

    if provider_str.is_empty() || model_str.is_empty() {
        anyhow::bail!("LLM router returned empty provider/model");
    }

    let provider = Provider::parse(provider_str)?;
    let cached = load_cached(provider.as_str(), cfg.models_cache_ttl_secs).unwrap_or_default();
    let (in_c, out_c) = model_cost(&cached, model_str);
    let cost = (est_in as f64 / 1_000_000.0) * in_c.unwrap_or(0.5)
             + (est_out as f64 / 1_000_000.0) * out_c.unwrap_or(1.5);

    Ok(RouteDecision {
        provider,
        model: model_str.to_string(),
        reason: format!("llm-router: {}", reason),
        estimated_input_tokens: est_in,
        estimated_output_tokens: est_out,
        estimated_cost_usd: cost,
    })
}

/// Heuristic (no-LLM) routing. Used as fallback.
pub fn heuristic_route(cfg: &AiConfig, prompt: &str, task_class: &str) -> Result<RouteDecision> {
    let est_in = (prompt.len() / 4).max(50) as u32;
    let est_out: u32 = match task_class {
        "ask" => 400,
        "explain" => 600,
        "review" => 800,
        "fix" | "refactor" => 1500,
        "commit-message" => 200,
        _ => 500,
    };

    let available: Vec<Provider> = registered_providers()?.into_iter().map(|(p, _)| p).collect();
    if available.is_empty() {
        anyhow::bail!("No AI providers configured. Register a key with `ore ai-keys register <provider> <key>`.");
    }

    let preferred_order: &[Provider] = match cfg.cost_mode.as_str() {
        "cheap" => &[Provider::Groq, Provider::DeepSeek, Provider::Google, Provider::Ollama, Provider::Mistral, Provider::OpenAI, Provider::Anthropic, Provider::OpenRouter, Provider::LmStudio],
        "quality" => &[Provider::Anthropic, Provider::OpenAI, Provider::Google, Provider::OpenRouter, Provider::Mistral, Provider::Groq, Provider::DeepSeek, Provider::Ollama, Provider::LmStudio],
        _ => &[Provider::Groq, Provider::OpenAI, Provider::Anthropic, Provider::Google, Provider::DeepSeek, Provider::Mistral, Provider::OpenRouter, Provider::Ollama, Provider::LmStudio],
    };

    for candidate in preferred_order {
        if !available.contains(candidate) { continue; }
        let cached = load_cached(candidate.as_str(), cfg.models_cache_ttl_secs).unwrap_or_default();
        let model = pick_model(candidate, &cached, cfg, task_class);
        let (in_c, out_c) = model_cost(&cached, &model);
        let cost = (est_in as f64 / 1_000_000.0) * in_c.unwrap_or(0.5)
                 + (est_out as f64 / 1_000_000.0) * out_c.unwrap_or(1.5);
        return Ok(RouteDecision {
            provider: *candidate,
            model,
            reason: format!("heuristic: cost_mode={}, task={}", cfg.cost_mode, task_class),
            estimated_input_tokens: est_in,
            estimated_output_tokens: est_out,
            estimated_cost_usd: cost,
        });
    }
    anyhow::bail!("No suitable provider found for task '{}'.", task_class);
}

fn pick_model(provider: &Provider, cached: &[ModelInfo], cfg: &AiConfig, task_class: &str) -> String {
    if !cfg.default_model.is_empty() && cfg.default_provider == provider.as_str() {
        return cfg.default_model.clone();
    }
    match provider {
        Provider::Groq => match task_class {
            "ask" | "commit-message" => "llama-3.1-8b-instant".to_string(),
            _ => "llama-3.3-70b-versatile".to_string(),
        },
        Provider::OpenAI => match (task_class, cfg.cost_mode.as_str()) {
            (_, "cheap") => "gpt-4o-mini".to_string(),
            ("fix", _) | ("refactor", _) | ("review", _) => "gpt-4o".to_string(),
            _ => "gpt-4o-mini".to_string(),
        },
        Provider::Anthropic => match cfg.cost_mode.as_str() {
            "cheap" => "claude-haiku-4-5".to_string(),
            "quality" => "claude-opus-4-5".to_string(),
            _ => "claude-sonnet-4-5".to_string(),
        },
        Provider::Google => match cfg.cost_mode.as_str() {
            "cheap" => "gemini-2.5-flash".to_string(),
            _ => "gemini-2.5-pro".to_string(),
        },
        Provider::DeepSeek => match task_class {
            "fix" | "refactor" | "review" => "deepseek-reasoner".to_string(),
            _ => "deepseek-chat".to_string(),
        },
        Provider::Mistral => match cfg.cost_mode.as_str() {
            "cheap" => "mistral-small-latest".to_string(),
            _ => "mistral-large-latest".to_string(),
        },
        Provider::OpenRouter => "openrouter/auto".to_string(),
        Provider::Ollama => cached.first().map(|m| m.id.clone()).unwrap_or_else(|| "llama3.2".to_string()),
        Provider::LmStudio => cached.first().map(|m| m.id.clone()).unwrap_or_else(|| "local-model".to_string()),
    }
}

fn model_cost(cached: &[ModelInfo], model: &str) -> (Option<f64>, Option<f64>) {
    if let Some(m) = cached.iter().find(|m| m.id == model) {
        return (m.input_cost_per_1m, m.output_cost_per_1m);
    }
    (None, None)
}
