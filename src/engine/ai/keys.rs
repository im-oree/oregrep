use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::engine::state::state_dir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Groq,
    OpenRouter,
    Google,
    Mistral,
    DeepSeek,
    Ollama,
    LmStudio,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::Groq => "groq",
            Provider::OpenRouter => "openrouter",
            Provider::Google => "google",
            Provider::Mistral => "mistral",
            Provider::DeepSeek => "deepseek",
            Provider::Ollama => "ollama",
            Provider::LmStudio => "lmstudio",
        }
    }
    pub fn all() -> &'static [Provider] {
        &[Provider::OpenAI, Provider::Anthropic, Provider::Groq, Provider::OpenRouter,
          Provider::Google, Provider::Mistral, Provider::DeepSeek, Provider::Ollama, Provider::LmStudio]
    }
    pub fn parse(s: &str) -> Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "openai" => Provider::OpenAI,
            "anthropic" | "claude" => Provider::Anthropic,
            "groq" => Provider::Groq,
            "openrouter" | "or" => Provider::OpenRouter,
            "google" | "gemini" => Provider::Google,
            "mistral" => Provider::Mistral,
            "deepseek" => Provider::DeepSeek,
            "ollama" => Provider::Ollama,
            "lmstudio" | "lm-studio" => Provider::LmStudio,
            other => bail!("Unknown provider: {} (valid: openai, anthropic, groq, openrouter, google, mistral, deepseek, ollama, lmstudio)", other),
        })
    }
    /// Env var checked first. None for local backends (Ollama/LM Studio).
    pub fn env_var(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("OPENAI_API_KEY"),
            Provider::Anthropic => Some("ANTHROPIC_API_KEY"),
            Provider::Groq => Some("GROQ_API_KEY"),
            Provider::OpenRouter => Some("OPENROUTER_API_KEY"),
            Provider::Google => Some("GOOGLE_API_KEY"),
            Provider::Mistral => Some("MISTRAL_API_KEY"),
            Provider::DeepSeek => Some("DEEPSEEK_API_KEY"),
            Provider::Ollama | Provider::LmStudio => None,
        }
    }
    pub fn needs_key(&self) -> bool {
        !matches!(self, Provider::Ollama | Provider::LmStudio)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyStore {
    /// provider name → key
    #[serde(default)]
    keys: BTreeMap<String, String>,
}

pub fn secrets_path() -> Result<PathBuf> {
    Ok(state_dir()?.join("secrets.toml"))
}

pub fn load_store() -> Result<KeyStore> {
    let p = secrets_path()?;
    if !p.exists() { return Ok(KeyStore::default()); }
    let text = std::fs::read_to_string(&p)?;
    Ok(toml::from_str(&text).unwrap_or_default())
}

pub fn save_store(store: &KeyStore) -> Result<()> {
    let p = secrets_path()?;
    let text = toml::to_string_pretty(store)?;
    std::fs::write(&p, text)?;
    restrict_perms(&p)?;
    Ok(())
}

/// Set restrictive perms on the secrets file so other users can't read it.
#[cfg(windows)]
fn restrict_perms(_path: &PathBuf) -> Result<()> {
    // Windows: %APPDATA% is already per-user. No further ACL work in v1.
    // Real hardening: icacls to remove Users:R, keep Owner:F.
    Ok(())
}
#[cfg(not(windows))]
fn restrict_perms(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

/// Get a key for a provider. Env var wins if set, else on-disk store.
pub fn get_key(provider: Provider) -> Option<String> {
    if let Some(env) = provider.env_var() {
        if let Ok(v) = std::env::var(env) {
            if !v.trim().is_empty() { return Some(v); }
        }
    }
    let store = load_store().ok()?;
    store.keys.get(provider.as_str()).cloned().filter(|s| !s.is_empty())
}

pub fn set_key(provider: Provider, key: &str) -> Result<()> {
    if !provider.needs_key() {
        bail!("Provider {} does not need an API key", provider.as_str());
    }
    let mut store = load_store()?;
    store.keys.insert(provider.as_str().to_string(), key.to_string());
    save_store(&store)?;
    Ok(())
}

pub fn remove_key(provider: Provider) -> Result<bool> {
    let mut store = load_store()?;
    let removed = store.keys.remove(provider.as_str()).is_some();
    save_store(&store)?;
    Ok(removed)
}

pub fn registered_providers() -> Result<Vec<(Provider, KeySource)>> {
    let store = load_store()?;
    let mut out = Vec::new();
    for p in Provider::all() {
        if !p.needs_key() {
            // Local backends always "available"
            out.push((*p, KeySource::Local));
            continue;
        }
        if let Some(env) = p.env_var() {
            if std::env::var(env).map(|v| !v.trim().is_empty()).unwrap_or(false) {
                out.push((*p, KeySource::Env));
                continue;
            }
        }
        if store.keys.contains_key(p.as_str()) {
            out.push((*p, KeySource::Stored));
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
pub enum KeySource {
    Env,
    Stored,
    Local,
}

impl KeySource {
    pub fn label(&self) -> &'static str {
        match self {
            KeySource::Env => "env",
            KeySource::Stored => "stored",
            KeySource::Local => "local",
        }
    }
}

pub fn redact(key: &str) -> String {
    if key.len() < 12 { return "***".to_string(); }
    format!("{}…{}", &key[..4], &key[key.len().saturating_sub(4)..])
}
