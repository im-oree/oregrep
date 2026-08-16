use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

/// Show current state of a file relevant to patching:
/// imports, exports, function/class names, line count, last modified.
/// Answers "what does this file look like RIGHT NOW" in one command.
#[derive(Args)]
pub struct StateArgs {
    /// File to inspect
    pub file: PathBuf,

    /// Show line numbers next to detected symbols
    #[arg(short = 'n', long)]
    pub lines: bool,

    /// Compact one-line summary
    #[arg(short = 'c', long)]
    pub compact: bool,

    /// JSON output
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Focus on a specific symbol: show its signature, body, and location
    #[arg(long, value_name = "SYMBOL")]
    pub at: Option<String>,

    /// Context lines around the --at symbol (default: entire body)
    #[arg(short = 'C', long, default_value = "0")]
    pub context: usize,
}

pub fn run(args: StateArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let meta = std::fs::metadata(&args.file)?;
    let content = read_file_smart(&args.file)?;
    let lines: Vec<&str> = content.lines().collect();
    let ext = args.file.extension().and_then(|s| s.to_str()).unwrap_or("");

    let imports = detect_imports(&lines, ext);
    let exports = detect_exports(&lines, ext);
    let symbols = detect_symbols(&lines, ext);

    // --at mode: focus on a specific symbol
    if let Some(target) = &args.at {
        return show_symbol_focus(&args.file, &lines, &symbols, target, args.context, ext);
    }

    let has_bom = std::fs::read(&args.file)?
        .starts_with(&[0xEF, 0xBB, 0xBF]);
    let newline_style = if content.contains("\r\n") { "CRLF" } else { "LF" };
    let mtime = meta.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            let secs = d.as_secs();
            let dt = chrono::DateTime::from_timestamp(secs as i64, 0);
            dt.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| format!("epoch+{}", secs))
        })
        .unwrap_or_else(|| "unknown".to_string());

    if args.json {
        let json = serde_json::json!({
            "file": args.file.display().to_string(),
            "size": meta.len(),
            "lines": lines.len(),
            "encoding": if has_bom { "UTF-8 BOM" } else { "UTF-8" },
            "newlines": newline_style,
            "modified": mtime,
            "imports": imports.iter().map(|(l, t)| serde_json::json!({"line": l, "text": t})).collect::<Vec<_>>(),
            "exports": exports.iter().map(|(l, t)| serde_json::json!({"line": l, "text": t})).collect::<Vec<_>>(),
            "symbols": symbols.iter().map(|(l, k, n)| serde_json::json!({"line": l, "kind": k, "name": n})).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    if args.compact {
        println!(
            "{} · {} lines · {} · {} · {} imports · {} exports · {} symbols · modified {}",
            args.file.display().to_string().cyan(),
            lines.len().to_string().yellow(),
            newline_style.dimmed(),
            if has_bom { "BOM".yellow().to_string() } else { "no BOM".dimmed().to_string() },
            imports.len().to_string().green(),
            exports.len().to_string().green(),
            symbols.len().to_string().green(),
            mtime.dimmed(),
        );
        return Ok(());
    }

    // Full report
    println!("{}", "═══ FILE STATE ═══".cyan().bold());
    println!("  {}: {}", "Path".dimmed(), args.file.display().to_string().cyan());
    println!("  {}: {} bytes, {} lines", "Size".dimmed(),
        meta.len().to_string().yellow(),
        lines.len().to_string().yellow());
    println!("  {}: {}, {}", "Encoding".dimmed(),
        if has_bom { "UTF-8 with BOM".yellow().to_string() } else { "UTF-8".dimmed().to_string() },
        newline_style.yellow());
    println!("  {}: {}", "Modified".dimmed(), mtime.dimmed());

    if !imports.is_empty() {
        println!("\n{} ({})", "Imports".cyan().bold(), imports.len().to_string().yellow());
        for (ln, text) in &imports {
            if args.lines {
                println!("  {:>4} │ {}", ln.to_string().green(), text.trim());
            } else {
                println!("  {}", text.trim());
            }
        }
    }

    if !exports.is_empty() {
        println!("\n{} ({})", "Exports".cyan().bold(), exports.len().to_string().yellow());
        for (ln, text) in &exports {
            if args.lines {
                println!("  {:>4} │ {}", ln.to_string().green(), text.trim());
            } else {
                println!("  {}", text.trim());
            }
        }
    }

    if !symbols.is_empty() {
        println!("\n{} ({})", "Symbols".cyan().bold(), symbols.len().to_string().yellow());
        for (ln, kind, name) in &symbols {
            if args.lines {
                println!("  {:>4} │ {} {}", ln.to_string().green(), kind.magenta(), name);
            } else {
                println!("  {} {}", kind.magenta(), name);
            }
        }
    }

    println!();
    Ok(())
}

fn show_symbol_focus(
    file: &std::path::Path,
    lines: &[&str],
    symbols: &[(usize, String, String)],
    target: &str,
    context: usize,
    ext: &str,
) -> Result<()> {
    // Find matching symbols
    let matches: Vec<&(usize, String, String)> = symbols
        .iter()
        .filter(|(_, _, name)| name == target || name.eq_ignore_ascii_case(target))
        .collect();

    if matches.is_empty() {
        eprintln!("{} symbol '{}' not found in {}",
            "Not found:".red().bold(), target, file.display());
        eprintln!("{}", "Available symbols:".dimmed());
        for (ln, kind, name) in symbols.iter().take(20) {
            eprintln!("  {} {} {}", ln.to_string().dimmed(), kind.magenta(), name);
        }
        std::process::exit(1);
    }

    for m in matches {
        let (start_line, kind, name) = m;
        let start_idx = start_line - 1;
        let end_idx = find_symbol_end(lines, start_idx, ext);

        let ctx_start = start_idx.saturating_sub(context);
        let ctx_end = (end_idx + context).min(lines.len());

        println!("{}", format!("═══ {} {} ═══", kind, name).cyan().bold());
        println!("  {}: {}:{}", "Location".dimmed(),
            file.display().to_string().cyan(),
            start_line.to_string().yellow());
        println!("  {}: {} lines", "Body".dimmed(),
            (end_idx - start_idx).to_string().yellow());
        println!();

        for i in ctx_start..ctx_end {
            let ln = i + 1;
            let is_body = i >= start_idx && i < end_idx;
            if is_body {
                println!("{:>5} │ {}", ln.to_string().yellow(), lines[i]);
            } else {
                println!("{:>5} │ {}", ln.to_string().dimmed(), lines[i].dimmed());
            }
        }
        println!();
    }

    Ok(())
}

fn find_symbol_end(lines: &[&str], start_idx: usize, ext: &str) -> usize {
    match ext {
        "py" => find_end_python_indent(lines, start_idx),
        _ => find_end_braces(lines, start_idx),
    }
}

fn find_end_braces(lines: &[&str], start_idx: usize) -> usize {
    // Find opening brace, then match balanced braces (skipping comments approximately)
    let mut depth = 0i32;
    let mut seen_open = false;
    let mut in_block_comment = false;

    for i in start_idx..lines.len() {
        let line = lines[i];
        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            let c = bytes[j];
            if in_block_comment {
                if c == b'*' && j + 1 < bytes.len() && bytes[j + 1] == b'/' {
                    in_block_comment = false;
                    j += 2;
                    continue;
                }
                j += 1;
                continue;
            }
            if c == b'/' && j + 1 < bytes.len() {
                if bytes[j + 1] == b'/' { break; }
                if bytes[j + 1] == b'*' { in_block_comment = true; j += 2; continue; }
            }
            if c == b'{' { depth += 1; seen_open = true; }
            else if c == b'}' {
                depth -= 1;
                if seen_open && depth == 0 { return i + 1; }
            }
            j += 1;
        }
        // Single-line def (no body braces on same line)
        if seen_open && depth == 0 { return i + 1; }
    }
    lines.len()
}

fn find_end_python_indent(lines: &[&str], start_idx: usize) -> usize {
    let def_line = lines.get(start_idx).unwrap_or(&"");
    let def_indent = def_line.len() - def_line.trim_start().len();
    for i in (start_idx + 1)..lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        let indent = line.len() - trimmed.len();
        if indent <= def_indent { return i; }
    }
    lines.len()
}

fn detect_imports(lines: &[&str], ext: &str) -> Vec<(usize, String)> {
    let patterns: Vec<Regex> = match ext {
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => vec![
            Regex::new(r"^\s*import\s+.*").unwrap(),
            Regex::new(r#"^\s*const\s+\{?[^=]*\}?\s*=\s*require\s*\("#).unwrap(),
        ],
        "rs" => vec![
            Regex::new(r"^\s*use\s+.*;").unwrap(),
        ],
        "py" => vec![
            Regex::new(r"^\s*(from\s+\S+\s+)?import\s+.*").unwrap(),
        ],
        "go" => vec![
            Regex::new(r"^\s*import\s+.*").unwrap(),
        ],
        _ => vec![],
    };

    let mut hits = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for pat in &patterns {
            if pat.is_match(line) {
                hits.push((i + 1, line.to_string()));
                break;
            }
        }
    }
    hits
}

fn detect_exports(lines: &[&str], ext: &str) -> Vec<(usize, String)> {
    let patterns: Vec<Regex> = match ext {
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => vec![
            Regex::new(r"^\s*export\s+(default\s+)?(function|class|const|let|var|interface|type|enum)\s+\w+").unwrap(),
            Regex::new(r"^\s*export\s*\{").unwrap(),
            Regex::new(r"^\s*module\.exports\s*=").unwrap(),
        ],
        "rs" => vec![
            Regex::new(r"^\s*pub\s+(fn|struct|enum|trait|mod|const|static|type)\s+\w+").unwrap(),
            Regex::new(r"^\s*pub\s+use\s+").unwrap(),
        ],
        _ => vec![],
    };

    let mut hits = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for pat in &patterns {
            if pat.is_match(line) {
                hits.push((i + 1, line.to_string()));
                break;
            }
        }
    }
    hits
}

fn detect_symbols(lines: &[&str], ext: &str) -> Vec<(usize, String, String)> {
    let patterns: Vec<(Regex, &'static str)> = match ext {
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => vec![
            (Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)").unwrap(), "fn"),
            (Regex::new(r"^\s*(?:export\s+)?class\s+(\w+)").unwrap(), "class"),
            (Regex::new(r"^\s*(?:export\s+)?interface\s+(\w+)").unwrap(), "interface"),
            (Regex::new(r"^\s*(?:export\s+)?type\s+(\w+)\s*=").unwrap(), "type"),
            (Regex::new(r"^\s*(?:export\s+)?enum\s+(\w+)").unwrap(), "enum"),
            (Regex::new(r"^\s*(?:export\s+)?const\s+(\w+)\s*[:=]\s*(?:async\s*)?\(").unwrap(), "fn"),
            (Regex::new(r"^\s*(?:export\s+)?const\s+(\w+)\s*=\s*(?:async\s*)?function").unwrap(), "fn"),
        ],
        "rs" => vec![
            (Regex::new(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap(), "fn"),
            (Regex::new(r"^\s*(?:pub\s+)?struct\s+(\w+)").unwrap(), "struct"),
            (Regex::new(r"^\s*(?:pub\s+)?enum\s+(\w+)").unwrap(), "enum"),
            (Regex::new(r"^\s*(?:pub\s+)?trait\s+(\w+)").unwrap(), "trait"),
            (Regex::new(r"^\s*impl(?:<[^>]*>)?\s+(\w+)").unwrap(), "impl"),
        ],
        "py" => vec![
            (Regex::new(r"^\s*(?:async\s+)?def\s+(\w+)").unwrap(), "fn"),
            (Regex::new(r"^\s*class\s+(\w+)").unwrap(), "class"),
        ],
        "go" => vec![
            (Regex::new(r"^\s*func\s+(?:\([^)]*\)\s+)?(\w+)").unwrap(), "fn"),
            (Regex::new(r"^\s*type\s+(\w+)\s+struct").unwrap(), "struct"),
            (Regex::new(r"^\s*type\s+(\w+)\s+interface").unwrap(), "interface"),
        ],
        _ => vec![],
    };

    let mut hits = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        for (pat, kind) in &patterns {
            if let Some(cap) = pat.captures(line) {
                if let Some(m) = cap.get(1) {
                    hits.push((i + 1, kind.to_string(), m.as_str().to_string()));
                    break;
                }
            }
        }
    }
    hits
}
