// Agent tool registry: ore command wrappers for the agent loop.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

use crate::engine::ai::events::AiEvent;
use crate::engine::proc::run_cmd;

/// A tool the LLM can invoke. Currently: shells out to ore subcommands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value, // JSON schema
    pub destructive: bool,
}

/// Ore command → tool spec. Keep this list conservative and well-described so LLMs use them well.
pub fn builtin_tools() -> Vec<ToolSpec> {
    vec![
        // ---- Read-only introspection ----
        tool("ore-find", "Search a pattern across files. Regex by default. Returns matched lines with file paths and line numbers.", json_props(&[
            ("pattern", "string", "Regex pattern to search for"),
            ("path", "string", "Path to search (defaults to '.')"),
            ("ext", "string", "Comma-separated extensions to include (e.g. 'ts,tsx')"),
        ], &["pattern"]), false),
        tool("ore-tree", "Print a directory tree.", json_props(&[
            ("path", "string", "Path (defaults to '.')"),
            ("depth", "integer", "Max depth"),
        ], &[]), false),
        tool("ore-cat", "Print a file with smart encoding detection.", json_props(&[
            ("file", "string", "File to print"),
            ("number", "boolean", "Show line numbers"),
        ], &["file"]), false),
        tool("ore-extract", "Extract line ranges from a file (multi-range, multi-file supported).", json_props(&[
            ("file", "string", "File to extract from"),
            ("ranges", "string", "Range spec like '10-30,50-70,100'"),
        ], &["file", "ranges"]), false),
        tool("ore-symbols", "List every named/exported symbol across a path (functions, classes, hooks, etc.).", json_props(&[
            ("path", "string", "Path to scan"),
            ("kind", "string", "Filter by kind: fn|class|hook|comp|type|iface|const|struct|trait|mod"),
            ("name", "string", "Filter by name substring"),
        ], &[]), false),
        tool("ore-outline", "Outline one file's structure with line numbers.", json_props(&[
            ("file", "string", "File to outline"),
        ], &["file"]), false),
        tool("ore-refs", "Find every reference to a symbol across a path.", json_props(&[
            ("symbol", "string", "Symbol name"),
            ("path", "string", "Path to search"),
        ], &["symbol"]), false),
        tool("ore-used-by", "List files that import from a given file.", json_props(&[
            ("file", "string", "Target file"),
            ("path", "string", "Root path to scan for importers"),
        ], &["file"]), false),
        tool("ore-imports-of", "Show what a file imports.", json_props(&[
            ("file", "string", "File to inspect"),
        ], &["file"]), false),
        tool("ore-neighbors", "Recursive dependency neighborhood around a file.", json_props(&[
            ("file", "string", "Starting file"),
            ("depth", "integer", "Recursion depth (default 2)"),
        ], &["file"]), false),
        tool("ore-digest", "Codebase digest (structural summary of a directory).", json_props(&[
            ("path", "string", "Root path"),
            ("ext", "string", "Extensions filter"),
        ], &[]), false),
        tool("ore-explain", "Heuristic English summary of what a file does.", json_props(&[
            ("file", "string", "File to explain"),
        ], &["file"]), false),
        tool("ore-health", "Codebase health report (score, todos, code smells).", json_props(&[
            ("path", "string", "Root path"),
        ], &[]), false),
        tool("ore-stats", "File/line/size stats across a path.", json_props(&[
            ("path", "string", "Root path"),
        ], &[]), false),
        tool("ore-git-log", "Recent git commits.", json_props(&[
            ("limit", "integer", "Max commits (default 20)"),
        ], &[]), false),
        tool("ore-git-status", "Working tree status.", json_props(&[], &[]), false),
        tool("ore-git-changed", "List changed files with filters.", json_props(&[], &[]), false),
        tool("ore-git-blame", "Blame a file (optionally a range).", json_props(&[
            ("file", "string", "File to blame"),
            ("range", "string", "Range like '10-20'"),
        ], &["file"]), false),

        // ---- Web ----
        tool("web-search", "Search the web via SearXNG with DuckDuckGo fallback. Returns titles + URLs + snippets.", json_props(&[
            ("query", "string", "Search query"),
        ], &["query"]), false),
        tool("web-fetch-clean", "Fetch a URL and strip to article text (nav/scripts removed).", json_props(&[
            ("url", "string", "URL to fetch"),
        ], &["url"]), false),

        // ---- Destructive (require confirmation unless --auto) ----
        tool("ore-patch", "Apply a literal find/replace to a file. Auto-backs up first.", json_props(&[
            ("file", "string", "File to patch"),
            ("find", "string", "Exact text to find (must match once by default)"),
            ("replace", "string", "Replacement text"),
        ], &["file", "find", "replace"]), true),
        tool("ore-replace", "Regex find/replace in a file.", json_props(&[
            ("pattern", "string", "Regex pattern"),
            ("replacement", "string", "Replacement (supports $1, $2)"),
            ("file", "string", "File to modify"),
        ], &["pattern", "replacement", "file"]), true),
        tool("ore-backup", "Create a labeled backup of a file.", json_props(&[
            ("file", "string", "File to back up"),
            ("label", "string", "Label suffix (e.g. AGENT-FIX)"),
        ], &["file"]), true),
        tool("ore-restore", "Restore a file from its most recent (or labeled) backup.", json_props(&[
            ("file", "string", "File to restore"),
            ("label", "string", "Backup label"),
        ], &["file"]), true),
        tool("ore-compile-rust", "Run cargo check on a Rust project.", json_props(&[
            ("path", "string", "Project root"),
        ], &[]), true),
        tool("ore-compile-ts", "Run tsc --noEmit on a TypeScript project.", json_props(&[
            ("path", "string", "Project root"),
        ], &[]), true),
        tool("ore-verify", "Run typecheck + lint + tests bundle.", json_props(&[
            ("path", "string", "Project root"),
        ], &[]), true),
    ]
}

fn tool(name: &str, desc: &str, schema: serde_json::Value, destructive: bool) -> ToolSpec {
    ToolSpec {
        name: name.to_string(),
        description: desc.to_string(),
        input_schema: schema,
        destructive,
    }
}

fn json_props(fields: &[(&str, &str, &str)], required: &[&str]) -> serde_json::Value {
    let mut props = serde_json::Map::new();
    for (name, ty, desc) in fields {
        props.insert(name.to_string(), serde_json::json!({
            "type": ty,
            "description": desc,
        }));
    }
    serde_json::json!({
        "type": "object",
        "properties": props,
        "required": required,
    })
}

pub fn find_tool<'a>(tools: &'a [ToolSpec], name: &str) -> Option<&'a ToolSpec> {
    tools.iter().find(|t| t.name == name)
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub ok: bool,
    pub output: String,
    pub duration_ms: u128,
}

/// Execute a tool by name. `args` is the JSON object from the LLM's tool call.
pub fn execute(tool: &ToolSpec, args: &serde_json::Value, tx: Option<&Sender<AiEvent>>) -> Result<ToolResult> {
    if let Some(t) = tx {
        let _ = t.send(AiEvent::ToolCallExecuting { name: tool.name.clone() });
    }
    let start = std::time::Instant::now();
    // Map tool name → ore CLI invocation
    let cmd_line = build_command(&tool.name, args)?;
    let result = run_cmd(&cmd_line, false, true)?;
    let success = result.success();
    let combined = if result.stderr.is_empty() {
        result.stdout
    } else {
        format!("{}\n[stderr]\n{}", result.stdout, result.stderr)
    };
    let out = ToolResult {
        ok: success,
        output: truncate(&combined, 8000),
        duration_ms: start.elapsed().as_millis(),
    };
    if let Some(t) = tx {
        let preview = truncate(&out.output, 200);
        let _ = t.send(AiEvent::ToolCallResult {
            name: tool.name.clone(),
            ok: out.ok,
            preview,
        });
    }
    Ok(out)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        let cut: String = s.chars().take(max).collect();
        format!("{}\n[…truncated…]", cut)
    } else {
        s.to_string()
    }
}

fn build_command(tool_name: &str, args: &serde_json::Value) -> Result<String> {
    // Extract args as string map
    let obj = args.as_object().cloned().unwrap_or_default();
    let get = |k: &str| -> Option<String> {
        obj.get(k).map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    };
    let quote = |s: &str| -> String {
        if s.contains(' ') || s.contains('"') { format!("\"{}\"", s.replace('"', "\\\"")) } else { s.to_string() }
    };
    // Map each tool
    let cmd = match tool_name {
        "ore-find" => {
            let mut parts = vec!["ore find".to_string(), quote(&get("pattern").unwrap_or_default())];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            if let Some(e) = get("ext") { parts.push(format!("-e {}", quote(&e))); }
            parts.push("-c".to_string()); // count mode is compact
            parts.join(" ")
        }
        "ore-tree" => {
            let mut parts = vec!["ore tree".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            if let Some(d) = get("depth") { parts.push(format!("-d {}", d)); }
            parts.join(" ")
        }
        "ore-cat" => {
            let mut parts = vec!["ore cat".to_string(), quote(&get("file").unwrap_or_default())];
            if get("number").map(|v| v == "true").unwrap_or(false) { parts.push("-n".to_string()); }
            parts.join(" ")
        }
        "ore-extract" => {
            format!("ore extract {} {}", quote(&get("file").unwrap_or_default()), quote(&get("ranges").unwrap_or_default()))
        }
        "ore-symbols" => {
            let mut parts = vec!["ore symbols".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            if let Some(k) = get("kind") { parts.push(format!("-k {}", quote(&k))); }
            if let Some(n) = get("name") { parts.push(format!("-n {}", quote(&n))); }
            parts.join(" ")
        }
        "ore-outline" => format!("ore outline {}", quote(&get("file").unwrap_or_default())),
        "ore-refs" => {
            let mut parts = vec!["ore refs".to_string(), quote(&get("symbol").unwrap_or_default())];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        "ore-used-by" => {
            let mut parts = vec!["ore used-by".to_string(), quote(&get("file").unwrap_or_default())];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        "ore-imports-of" => format!("ore imports-of {}", quote(&get("file").unwrap_or_default())),
        "ore-neighbors" => {
            let mut parts = vec!["ore neighbors".to_string(), quote(&get("file").unwrap_or_default())];
            if let Some(d) = get("depth") { parts.push(format!("-d {}", d)); }
            parts.join(" ")
        }
        "ore-digest" => {
            let mut parts = vec!["ore digest".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            if let Some(e) = get("ext") { parts.push(format!("-e {}", quote(&e))); }
            parts.join(" ")
        }
        "ore-explain" => format!("ore explain {}", quote(&get("file").unwrap_or_default())),
        "ore-health" => {
            let mut parts = vec!["ore health".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        "ore-stats" => {
            let mut parts = vec!["ore stats".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        "ore-git-log" => {
            let mut parts = vec!["ore git-log".to_string()];
            if let Some(n) = get("limit") { parts.push(format!("-n {}", n)); }
            parts.join(" ")
        }
        "ore-git-status" => "ore git-status".to_string(),
        "ore-git-changed" => "ore git-changed".to_string(),
        "ore-git-blame" => {
            let mut parts = vec!["ore git-blame".to_string(), quote(&get("file").unwrap_or_default())];
            if let Some(r) = get("range") { parts.push(format!("-L {}", quote(&r))); }
            parts.join(" ")
        }
        "web-search" => format!("ore web-search {}", quote(&get("query").unwrap_or_default())),
        "web-fetch-clean" => format!("ore web-fetch-clean {}", quote(&get("url").unwrap_or_default())),
        "ore-patch" => {
            format!("ore patch {} -f {} -r {}",
                quote(&get("file").unwrap_or_default()),
                quote(&get("find").unwrap_or_default()),
                quote(&get("replace").unwrap_or_default()))
        }
        "ore-replace" => {
            format!("ore replace {} {} {}",
                quote(&get("pattern").unwrap_or_default()),
                quote(&get("replacement").unwrap_or_default()),
                quote(&get("file").unwrap_or_default()))
        }
        "ore-backup" => {
            let mut parts = vec!["ore backup".to_string(), quote(&get("file").unwrap_or_default())];
            if let Some(l) = get("label") { parts.push(format!("-l {}", quote(&l))); }
            parts.join(" ")
        }
        "ore-restore" => {
            let mut parts = vec!["ore restore".to_string(), quote(&get("file").unwrap_or_default())];
            if let Some(l) = get("label") { parts.push(format!("-l {}", quote(&l))); }
            parts.join(" ")
        }
        "ore-compile-rust" => {
            let mut parts = vec!["ore compile-rust".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.push("-c".to_string());
            parts.join(" ")
        }
        "ore-compile-ts" => {
            let mut parts = vec!["ore compile-ts".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        "ore-verify" => {
            let mut parts = vec!["ore verify".to_string()];
            if let Some(p) = get("path") { parts.push(quote(&p)); }
            parts.join(" ")
        }
        other => anyhow::bail!("Unknown tool: {}", other),
    };
    Ok(cmd)
}
