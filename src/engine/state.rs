use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Get the ore state directory, creating it if missing.
/// Windows: %APPDATA%\ore
/// Unix:    ~/.config/ore
pub fn state_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(not(windows))]
    let base = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|h| h.join(".config"))
        .unwrap_or_else(|| PathBuf::from("."));

    let dir = base.join("ore");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).with_context(|| format!("Creating state dir: {}", dir.display()))?;
    }
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf> { Ok(state_dir()?.join("config.toml")) }
pub fn aliases_path() -> Result<PathBuf> { Ok(state_dir()?.join("aliases.toml")) }
pub fn focus_path() -> Result<PathBuf> { Ok(state_dir()?.join("focus")) }
pub fn sessions_dir() -> Result<PathBuf> {
    let d = state_dir()?.join("sessions");
    if !d.exists() { std::fs::create_dir_all(&d)?; }
    Ok(d)
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(flatten)]
    pub values: HashMap<String, String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let p = config_path()?;
        if !p.exists() { return Ok(Self::default()); }
        let s = std::fs::read_to_string(&p)?;
        Ok(toml::from_str(&s).unwrap_or_default())
    }
    pub fn save(&self) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        std::fs::write(config_path()?, s)?;
        Ok(())
    }
    pub fn get(&self, key: &str) -> Option<&String> { self.values.get(key) }
    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
    pub fn remove(&mut self, key: &str) -> Option<String> { self.values.remove(key) }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Aliases {
    #[serde(flatten)]
    pub map: HashMap<String, String>,
}

impl Aliases {
    pub fn load() -> Result<Self> {
        let p = aliases_path()?;
        if !p.exists() { return Ok(Self::default()); }
        let s = std::fs::read_to_string(&p)?;
        Ok(toml::from_str(&s).unwrap_or_default())
    }
    pub fn save(&self) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        std::fs::write(aliases_path()?, s)?;
        Ok(())
    }
}

pub fn read_focus() -> Result<Option<PathBuf>> {
    let p = focus_path()?;
    if !p.exists() { return Ok(None); }
    let s = std::fs::read_to_string(&p)?;
    let trimmed = s.trim();
    if trimmed.is_empty() { return Ok(None); }
    Ok(Some(PathBuf::from(trimmed)))
}

pub fn write_focus(path: Option<&PathBuf>) -> Result<()> {
    let fp = focus_path()?;
    match path {
        Some(p) => std::fs::write(fp, p.to_string_lossy().as_ref())?,
        None => { if fp.exists() { std::fs::remove_file(fp)?; } }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionEvent {
    pub timestamp: String,
    pub kind: String,      // "backup" | "delete" | "note" | ...
    pub file: Option<String>,
    pub backup: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Session {
    pub name: String,
    pub started_at: String,
    pub events: Vec<SessionEvent>,
}

pub fn current_session_marker() -> Result<PathBuf> { Ok(state_dir()?.join("current-session")) }

pub fn current_session_name() -> Result<Option<String>> {
    let p = current_session_marker()?;
    if !p.exists() { return Ok(None); }
    let s = std::fs::read_to_string(&p)?;
    let trimmed = s.trim();
    if trimmed.is_empty() { Ok(None) } else { Ok(Some(trimmed.to_string())) }
}

pub fn session_path(name: &str) -> Result<PathBuf> {
    Ok(sessions_dir()?.join(format!("{}.toml", name)))
}

pub fn load_session(name: &str) -> Result<Session> {
    let p = session_path(name)?;
    if !p.exists() {
        return Ok(Session { name: name.to_string(), started_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), events: vec![] });
    }
    let s = std::fs::read_to_string(&p)?;
    Ok(toml::from_str(&s).unwrap_or_else(|_| Session { name: name.to_string(), started_at: String::new(), events: vec![] }))
}

pub fn save_session(session: &Session) -> Result<()> {
    let p = session_path(&session.name)?;
    let s = toml::to_string_pretty(session)?;
    std::fs::write(p, s)?;
    Ok(())
}

pub fn set_current_session(name: Option<&str>) -> Result<()> {
    let p = current_session_marker()?;
    match name {
        Some(n) => std::fs::write(p, n)?,
        None => { if p.exists() { std::fs::remove_file(p)?; } }
    }
    Ok(())
}
