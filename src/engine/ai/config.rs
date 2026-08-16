use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::engine::state::state_dir;

pub fn config_path() -> Result<PathBuf> {
    Ok(state_dir()?.join("ai.toml"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Default provider used when no --model override
    pub default_provider: String,
    /// Default model within that provider (empty = router picks)
    pub default_model: String,
    /// Cost mode: cheap | balanced | quality
    pub cost_mode: String,
    /// Max tokens per generation
    pub max_output_tokens: u32,
    /// Total budget cap in USD per session (0 = unlimited)
    pub session_budget_usd: f64,
    /// Total budget cap in USD per single call (0 = unlimited)
    pub call_budget_usd: f64,
    /// Router provider (falls back to heuristic if unset)
    pub router_provider: String,
    pub router_model: String,
    /// Cache TTL for /models responses (seconds)
    pub models_cache_ttl_secs: u64,
    /// Auto-run tool calls without confirmation on read-only tools
    pub auto_readonly_tools: bool,
    /// Stream tokens by default
    pub stream: bool,
    /// Temperature (0.0..=2.0)
    pub temperature: f32,
    /// Context budget in tokens (auto-truncate to fit)
    pub context_budget_tokens: u32,
    /// Primary SearXNG instance URL
    pub search_searxng_url: String,
    /// Search request timeout in seconds
    pub search_timeout_secs: u64,
    /// Max results returned to the LLM
    pub search_max_results: usize,
    /// Max chars kept per result snippet (post-clean)
    pub search_max_chars_per_result: usize,
    /// Fetch + clean full page text when following a result URL (max chars)
    pub search_fetch_max_chars: usize,
    /// Comma-separated fallback instance URLs
    pub search_fallback_instances: String,
    /// Provider RPM caps. 0 or missing = uncapped.
    pub rate_limits: BTreeMap<String, u32>,
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut rate_limits = BTreeMap::new();
        rate_limits.insert("groq".to_string(), 30);

        AiConfig {
            default_provider: "groq".to_string(),
            default_model: String::new(),
            cost_mode: "balanced".to_string(),
            max_output_tokens: 2048,
            session_budget_usd: 0.0,
            call_budget_usd: 0.0,
            router_provider: "groq".to_string(),
            router_model: String::new(),
            models_cache_ttl_secs: 24 * 3600,
            auto_readonly_tools: true,
            stream: true,
            temperature: 0.7,
            context_budget_tokens: 100_000,
            search_searxng_url: "https://searx.be".to_string(),
            search_timeout_secs: 6,
            search_max_results: 8,
            search_max_chars_per_result: 400,
            search_fetch_max_chars: 8000,
            search_fallback_instances: "https://priv.au,https://searx.tiekoetter.com,https://search.hbubli.cc,https://baresearch.org,https://search.rhscz.eu".to_string(),
            rate_limits,
        }
    }
}

pub fn load() -> Result<AiConfig> {
    let p = config_path()?;
    if !p.exists() { return Ok(AiConfig::default()); }
    let text = std::fs::read_to_string(&p)?;
    Ok(toml::from_str(&text).unwrap_or_default())
}

pub fn save(cfg: &AiConfig) -> Result<()> {
    let p = config_path()?;
    std::fs::write(&p, toml::to_string_pretty(cfg)?)?;
    Ok(())
}

pub fn set_field(cfg: &mut AiConfig, key: &str, value: &str) -> Result<()> {
    match key {
        "default_provider" | "default-provider" => cfg.default_provider = value.to_string(),
        "default_model" | "default-model" => cfg.default_model = value.to_string(),
        "cost_mode" | "cost-mode" => cfg.cost_mode = value.to_string(),
        "max_output_tokens" | "max-output-tokens" => cfg.max_output_tokens = value.parse()?,
        "session_budget_usd" | "session-budget-usd" => cfg.session_budget_usd = value.parse()?,
        "call_budget_usd" | "call-budget-usd" => cfg.call_budget_usd = value.parse()?,
        "router_provider" | "router-provider" => cfg.router_provider = value.to_string(),
        "router_model" | "router-model" => cfg.router_model = value.to_string(),
        "models_cache_ttl_secs" | "models-cache-ttl-secs" => cfg.models_cache_ttl_secs = value.parse()?,
        "auto_readonly_tools" | "auto-readonly-tools" => cfg.auto_readonly_tools = value.parse()?,
        "stream" => cfg.stream = value.parse()?,
        "temperature" => cfg.temperature = value.parse()?,
        "context_budget_tokens" | "context-budget-tokens" => cfg.context_budget_tokens = value.parse()?,
        "search_searxng_url" | "search-searxng-url" => cfg.search_searxng_url = value.to_string(),
        "search_timeout_secs" | "search-timeout-secs" => cfg.search_timeout_secs = value.parse()?,
        "search_max_results" | "search-max-results" => cfg.search_max_results = value.parse()?,
        "search_max_chars_per_result" | "search-max-chars-per-result" => cfg.search_max_chars_per_result = value.parse()?,
        "search_fetch_max_chars" | "search-fetch-max-chars" => cfg.search_fetch_max_chars = value.parse()?,
        "search_fallback_instances" | "search-fallback-instances" => cfg.search_fallback_instances = value.to_string(),
        "rate_limits" | "rate-limits" => cfg.rate_limits = parse_rate_limits(value)?,
        other => anyhow::bail!("Unknown config key: {} (see `ore ai config list`)", other),
    }
    Ok(())
}

pub fn as_pairs(cfg: &AiConfig) -> Vec<(&'static str, String)> {
    vec![
        ("default_provider", cfg.default_provider.clone()),
        ("default_model", cfg.default_model.clone()),
        ("cost_mode", cfg.cost_mode.clone()),
        ("max_output_tokens", cfg.max_output_tokens.to_string()),
        ("session_budget_usd", format!("{}", cfg.session_budget_usd)),
        ("call_budget_usd", format!("{}", cfg.call_budget_usd)),
        ("router_provider", cfg.router_provider.clone()),
        ("router_model", cfg.router_model.clone()),
        ("models_cache_ttl_secs", cfg.models_cache_ttl_secs.to_string()),
        ("auto_readonly_tools", cfg.auto_readonly_tools.to_string()),
        ("stream", cfg.stream.to_string()),
        ("temperature", format!("{}", cfg.temperature)),
        ("context_budget_tokens", cfg.context_budget_tokens.to_string()),
        ("search_searxng_url", cfg.search_searxng_url.clone()),
        ("search_timeout_secs", cfg.search_timeout_secs.to_string()),
        ("search_max_results", cfg.search_max_results.to_string()),
        ("search_max_chars_per_result", cfg.search_max_chars_per_result.to_string()),
        ("search_fetch_max_chars", cfg.search_fetch_max_chars.to_string()),
        ("search_fallback_instances", cfg.search_fallback_instances.clone()),
        ("rate_limits", serde_json::to_string(&cfg.rate_limits).unwrap_or_else(|_| "{}".to_string())),
    ]
}

fn parse_rate_limits(value: &str) -> Result<BTreeMap<String, u32>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(BTreeMap::new());
    }

    if trimmed.starts_with('{') {
        let parsed: BTreeMap<String, u32> = serde_json::from_str(trimmed)?;
        return Ok(parsed);
    }

    let mut out = BTreeMap::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        let (name, rpm) = part
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("rate_limits must be JSON or comma form like `groq=30,openai=60`"))?;
        out.insert(name.trim().to_string(), rpm.trim().parse()?);
    }
    Ok(out)
}
