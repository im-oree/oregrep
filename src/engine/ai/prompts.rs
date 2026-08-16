use anyhow::Result;
use std::path::PathBuf;

use crate::engine::state::state_dir;

pub fn prompts_dir() -> Result<PathBuf> {
    let d = state_dir()?.join("prompts");
    if !d.exists() {
        std::fs::create_dir_all(&d)?;
        seed_defaults(&d)?;
    }
    Ok(d)
}

pub fn seed_defaults(dir: &PathBuf) -> Result<()> {
    for (name, content) in DEFAULTS {
        let p = dir.join(name);
        if !p.exists() { std::fs::write(&p, content)?; }
    }
    Ok(())
}

pub fn list() -> Result<Vec<String>> {
    let d = prompts_dir()?;
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&d)? {
        let e = entry?;
        if e.path().extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(name) = e.path().file_stem().and_then(|s| s.to_str()) {
                out.push(name.to_string());
            }
        }
    }
    out.sort();
    Ok(out)
}

pub fn get(name: &str) -> Result<String> {
    let d = prompts_dir()?;
    let p = d.join(format!("{}.md", name));
    if !p.exists() { anyhow::bail!("Prompt not found: {}", name); }
    Ok(std::fs::read_to_string(p)?)
}

pub fn set(name: &str, content: &str) -> Result<()> {
    let d = prompts_dir()?;
    std::fs::write(d.join(format!("{}.md", name)), content)?;
    Ok(())
}

pub fn reset(name: &str) -> Result<()> {
    for (n, content) in DEFAULTS {
        if n.trim_end_matches(".md") == name {
            set(name, content)?;
            return Ok(());
        }
    }
    anyhow::bail!("No bundled default for prompt: {}", name);
}

pub fn default_for(name: &str) -> Option<&'static str> {
    for (n, content) in DEFAULTS {
        if n.trim_end_matches(".md") == name { return Some(content); }
    }
    None
}

const DEFAULTS: &[(&str, &str)] = &[
    ("router.md",
"You are ore's model router. Given a user task, choose the cheapest model that can complete it well.

Task class: {{task_class}}
User prompt (first 500 chars): {{prompt_snippet}}
Estimated context tokens: {{context_estimate}}

Available models (provider:id → context_window, input$/1M, output$/1M, capabilities):
{{model_table}}

Cost mode: {{cost_mode}} (cheap | balanced | quality)

Respond in this exact JSON shape (no prose):
{\"provider\": \"...\", \"model\": \"...\", \"reason\": \"...\", \"estimated_input_tokens\": 0, \"estimated_output_tokens\": 0}
"),
    ("ask.md",
"You are a precise, concise coding assistant embedded in a CLI tool called ore.
When answering, prefer runnable ore commands the user can copy. Use markdown code fences with `bash`.
If the question is about a codebase, reason about what tool call would find the answer, not what the answer is."),
    ("explain.md",
"Explain what this file does. Focus on: purpose, key exports, how it fits in a larger system, gotchas.
Skip line-by-line commentary. Use short paragraphs and bullet lists. Do not restate the code."),
    ("review.md",
"You are a senior code reviewer. For each issue, output a bullet with SEVERITY (blocker/major/minor/nit),
FILE:LINE, and a one-sentence recommendation. Group issues by file. Be terse and specific."),
    ("fix.md",
"You will produce a single unified diff (git-style) that fixes the described issue.
Only output the diff, no prose, no code fences. Include full file paths."),
    ("refactor.md",
"You will produce a plan and then execute it via tool calls.
Plan format: numbered steps, each step is 'action: <ore command or edit>'.
After each step, verify with `ore verify` or the appropriate check.
If verification fails, roll back and revise. Do not skip verification."),
    ("commit-message.md",
"Write a git commit message for the provided diff. Use conventional-commits style if the repo uses it,
else use plain English. Subject line <= 72 chars, imperative mood.
Body: bullet list of concrete changes. No fluff."),
    ("chat-system.md",
"You are ore, a codebase-aware assistant. You have access to hundreds of tools via the ore CLI.
Prefer calling tools to answer questions rather than guessing. Be terse."),
];
