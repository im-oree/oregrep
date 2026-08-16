use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::engine::state::state_dir;

/// Cached compile-error record for `errors-last`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompileError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub severity: String, // "error" | "warning"
    pub code: String,     // e.g. "TS2304", "E0308"
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CompileReport {
    pub tool: String,     // "tsc" | "cargo" | "npm" etc.
    pub timestamp: String,
    pub exit_code: i32,
    pub errors: Vec<CompileError>,
    pub warnings: Vec<CompileError>,
    pub raw_output: String,
}

pub fn errors_cache_path() -> Result<PathBuf> {
    Ok(state_dir()?.join("last-errors.json"))
}

pub fn save_report(report: &CompileReport) -> Result<()> {
    let path = errors_cache_path()?;
    let json = serde_json::to_string_pretty(report)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_report() -> Result<Option<CompileReport>> {
    let path = errors_cache_path()?;
    if !path.exists() { return Ok(None); }
    let text = std::fs::read_to_string(&path)?;
    let report: CompileReport = serde_json::from_str(&text)?;
    Ok(Some(report))
}

/// Parse tsc --pretty=false output like:
///   src/App.tsx(10,5): error TS2304: Cannot find name 'foo'.
pub fn parse_tsc_output(output: &str) -> (Vec<CompileError>, Vec<CompileError>) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let re = regex::Regex::new(r"^(.+?)\((\d+),(\d+)\):\s+(error|warning)\s+(TS\d+):\s+(.+)$").unwrap();
    for line in output.lines() {
        if let Some(caps) = re.captures(line.trim()) {
            let entry = CompileError {
                file: caps[1].to_string(),
                line: caps[2].parse().unwrap_or(0),
                column: caps[3].parse().unwrap_or(0),
                severity: caps[4].to_string(),
                code: caps[5].to_string(),
                message: caps[6].to_string(),
            };
            if entry.severity == "error" { errors.push(entry); }
            else { warnings.push(entry); }
        }
    }
    (errors, warnings)
}

/// Parse cargo output like:
///   error[E0308]: mismatched types
///     --> src/main.rs:12:9
pub fn parse_cargo_output(output: &str) -> (Vec<CompileError>, Vec<CompileError>) {
    let mut errors: Vec<CompileError> = Vec::new();
    let mut warnings: Vec<CompileError> = Vec::new();

    let header_re = regex::Regex::new(r"^(error|warning)(?:\[([A-Z0-9]+)\])?:\s*(.+)$").unwrap();
    let loc_re = regex::Regex::new(r"^\s*-->\s*(.+?):(\d+):(\d+)\s*$").unwrap();

    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some(hcaps) = header_re.captures(lines[i]) {
            let severity = hcaps[1].to_string();
            let code = hcaps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            let msg = hcaps[3].to_string();
            // Look forward a few lines for the location
            let mut file = String::new();
            let mut ln = 0usize;
            let mut col = 0usize;
            for j in (i + 1)..(i + 6).min(lines.len()) {
                if let Some(lcaps) = loc_re.captures(lines[j]) {
                    file = lcaps[1].to_string();
                    ln = lcaps[2].parse().unwrap_or(0);
                    col = lcaps[3].parse().unwrap_or(0);
                    break;
                }
            }
            let entry = CompileError {
                file, line: ln, column: col,
                severity: severity.clone(), code, message: msg,
            };
            // Skip aggregation lines like `error: could not compile ...` that
            // carry no location — they duplicate the real diagnostics.
            if entry.file.is_empty() && entry.line == 0 {
                // dropped
            } else if severity == "error" {
                errors.push(entry);
            } else {
                warnings.push(entry);
            }
        }
        i += 1;
    }
    (errors, warnings)
}

pub fn locks_path() -> Result<PathBuf> {
    Ok(state_dir()?.join("locks.json"))
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct LockRegistry {
    pub locked: Vec<String>,
}

pub fn load_locks() -> Result<LockRegistry> {
    let path = locks_path()?;
    if !path.exists() { return Ok(LockRegistry::default()); }
    let s = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&s).unwrap_or_default())
}

pub fn save_locks(reg: &LockRegistry) -> Result<()> {
    let path = locks_path()?;
    let json = serde_json::to_string_pretty(reg)?;
    std::fs::write(path, json)?;
    Ok(())
}
