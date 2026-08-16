use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::engine::state::state_dir;

pub fn snippets_dir() -> Result<PathBuf> {
    let d = state_dir()?.join("snippets");
    if !d.exists() { std::fs::create_dir_all(&d)?; }
    Ok(d)
}

pub fn templates_dir() -> Result<PathBuf> {
    let d = state_dir()?.join("templates");
    if !d.exists() { std::fs::create_dir_all(&d)?; }
    Ok(d)
}

pub fn macros_dir() -> Result<PathBuf> {
    let d = state_dir()?.join("macros");
    if !d.exists() { std::fs::create_dir_all(&d)?; }
    Ok(d)
}

pub fn snippet_path(name: &str) -> Result<PathBuf> {
    Ok(snippets_dir()?.join(format!("{}.txt", sanitize(name))))
}

pub fn template_path(name: &str) -> Result<PathBuf> {
    Ok(templates_dir()?.join(format!("{}.tmpl", sanitize(name))))
}

pub fn macro_path(name: &str) -> Result<PathBuf> {
    Ok(macros_dir()?.join(format!("{}.macro", sanitize(name))))
}

pub fn sanitize(name: &str) -> String {
    name.chars().map(|c| {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' }
    }).collect()
}

/// Simple template interpolation: replace `{{var}}` with values.
pub fn interpolate(template: &str, vars: &std::collections::HashMap<String, String>) -> String {
    let re = regex::Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
    re.replace_all(template, |caps: &regex::Captures| {
        let key = &caps[1];
        vars.get(key).cloned().unwrap_or_else(|| format!("{{{{{}}}}}", key))
    }).into_owned()
}

/// Extract variable names from a template
pub fn extract_vars(template: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}").unwrap();
    let mut out = Vec::new();
    for cap in re.captures_iter(template) {
        let v = cap[1].to_string();
        if !out.contains(&v) { out.push(v); }
    }
    out
}

pub fn parse_kv_pairs(pairs: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let mut m = std::collections::HashMap::new();
    for p in pairs {
        let (k, v) = p.split_once('=')
            .with_context(|| format!("Bad key=value: {}", p))?;
        m.insert(k.trim().to_string(), v.to_string());
    }
    Ok(m)
}
