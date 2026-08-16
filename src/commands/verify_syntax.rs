use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct VerifySyntaxArgs {
    files: Vec<PathBuf>,
}

pub fn run(args: VerifySyntaxArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("At least one file required"); }
    let mut ok = 0usize;
    let mut fail = 0usize;
    for f in &args.files {
        if !f.exists() { println!("  {} {}", "MISSING".red(), f.display()); fail += 1; continue; }
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let content = read_file_smart(f)?;
        let result: Result<(), String> = match ext.as_str() {
            "json" | "jsonc" => {
                // Try strict, then JSON5-lenient (comments + trailing commas)
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        let cleaned = strip_json5(&content);
                        serde_json::from_str::<serde_json::Value>(&cleaned).map(|_| ()).map_err(|e| e.to_string())
                    }
                }
            }
            "toml" => toml::from_str::<toml::Value>(&content).map(|_| ()).map_err(|e| e.to_string()),
            "yaml" | "yml" => Err("YAML validation not yet integrated".to_string()),
            _ => {
                // Basic brace-balance check for code files
                brace_balance(&content)
            }
        };
        match result {
            Ok(_) => { ok += 1; println!("  {} {} ({})", "OK".green().bold(), f.display().to_string().cyan(), ext.dimmed()); }
            Err(e) => { fail += 1; println!("  {} {}  {}", "FAIL".red().bold(), f.display().to_string().cyan(), e.dimmed()); }
        }
    }
    println!("\n{} {} ok, {} invalid", "Summary:".bold(), ok.to_string().green(), fail.to_string().red());
    if fail > 0 { std::process::exit(1); }
    Ok(())
}

fn brace_balance(s: &str) -> Result<(), String> {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string: Option<char> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if in_line_comment {
            if c == '\n' { in_line_comment = false; }
            i += 1; continue;
        }
        if in_block_comment {
            if c == '*' && next == Some('/') { in_block_comment = false; i += 2; continue; }
            i += 1; continue;
        }
        if let Some(q) = in_string {
            if c == '\\' { i += 2; continue; }
            if c == q { in_string = None; }
            i += 1; continue;
        }
        match c {
            '/' if next == Some('/') => { in_line_comment = true; i += 2; continue; }
            '/' if next == Some('*') => { in_block_comment = true; i += 2; continue; }
            '"' | '\'' | '`' => { in_string = Some(c); i += 1; continue; }
            '(' | '[' | '{' => stack.push(c),
            ')' => match stack.pop() { Some('(') => {}, _ => return Err(format!("Unmatched ) at char {}", i)) },
            ']' => match stack.pop() { Some('[') => {}, _ => return Err(format!("Unmatched ] at char {}", i)) },
            '}' => match stack.pop() { Some('{') => {}, _ => return Err(format!("Unmatched }} at char {}", i)) },
            _ => {}
        }
        i += 1;
    }
    if !stack.is_empty() { return Err(format!("Unclosed brackets: {:?}", stack)); }
    Ok(())
}

fn strip_json5(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    while i < bytes.len() {
        let c = bytes[i];
        let next = bytes.get(i + 1).copied();
        if in_line_comment { if c == b'\n' { in_line_comment = false; out.push('\n'); } i += 1; continue; }
        if in_block_comment { if c == b'*' && next == Some(b'/') { in_block_comment = false; i += 2; continue; } i += 1; continue; }
        if in_string {
            out.push(c as char);
            if c == b'\\' && i + 1 < bytes.len() { out.push(bytes[i + 1] as char); i += 2; continue; }
            if c == b'"' { in_string = false; }
            i += 1; continue;
        }
        match (c, next) {
            (b'/', Some(b'/')) => { in_line_comment = true; i += 2; }
            (b'/', Some(b'*')) => { in_block_comment = true; i += 2; }
            (b'"', _) => { in_string = true; out.push('"'); i += 1; }
            _ => { out.push(c as char); i += 1; }
        }
    }
    let re = regex::Regex::new(r",(\s*[}\]])").unwrap();
    re.replace_all(&out, "$1").to_string()
}
