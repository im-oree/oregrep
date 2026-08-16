use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;

/// Replace an entire function body by name. Works for TS/JS/Rust/Python.
/// Detects function via regex + brace/indent matching.
#[derive(Args)]
pub struct PatchFnArgs {
    file: PathBuf,

    /// Function/method name to replace
    name: String,

    /// New body (full function including signature)
    #[arg(short = 'r', long)]
    replace: String,

    #[arg(long)]
    dry_run: bool,

    #[arg(long)]
    no_backup: bool,

    #[arg(short = 'l', long)]
    label: Option<String>,

    #[arg(long)]
    literal: bool,
}

pub fn run(args: PatchFnArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let content = read_file_smart(&args.file)?;
    let ext = args.file.extension().and_then(|s| s.to_str()).unwrap_or("");

    let (start, end) = find_function(&content, &args.name, ext)?;

    let replacement = if args.literal {
        args.replace.clone()
    } else {
        crate::engine::patch::unescape_arg(&args.replace)
    };

    let mut new_content = String::with_capacity(content.len());
    new_content.push_str(&content[..start]);
    new_content.push_str(&replacement);
    new_content.push_str(&content[end..]);

    if args.dry_run {
        println!("{} would replace function '{}' at bytes {}-{}",
            "[DRY]".yellow(), args.name, start, end);
        println!("\n{}", "Original:".dimmed());
        for l in content[start..end].lines() {
            println!("  {} {}", "-".red(), l);
        }
        println!("\n{}", "Replacement:".dimmed());
        for l in replacement.lines() {
            println!("  {} {}", "+".green(), l);
        }
        return Ok(());
    }

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(||
            chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
        );
        let bp = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bp.display().to_string().dimmed());
    }

    let raw_bytes = std::fs::read(&args.file)?;
    let had_bom = raw_bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    write_atomic(&args.file, &new_content, had_bom)?;

    println!("{} function '{}' in {}",
        "Replaced:".green().bold(),
        args.name.cyan(),
        args.file.display().to_string().cyan()
    );

    Ok(())
}

fn find_function(content: &str, name: &str, ext: &str) -> Result<(usize, usize)> {
    let n = regex::escape(name);

    // Try common function signature patterns depending on language
    let patterns: Vec<String> = match ext {
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => vec![
            format!(r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+{}\s*[<(]", n),
            format!(r"(?m)^\s*(?:export\s+)?const\s+{}\s*[:=]\s*(?:async\s*)?\(", n),
            format!(r"(?m)^\s*(?:export\s+)?const\s+{}\s*[:=]\s*(?:async\s*)?function", n),
            format!(r"(?m)^\s*(?:public|private|protected|static)?\s*(?:async\s+)?{}\s*[<(]", n),
        ],
        "rs" => vec![
            format!(r"(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+{}\s*[<(]", n),
        ],
        "py" => vec![
            format!(r"(?m)^\s*(?:async\s+)?def\s+{}\s*\(", n),
        ],
        "go" => vec![
            format!(r"(?m)^\s*func\s+(?:\([^)]*\)\s+)?{}\s*\(", n),
        ],
        _ => vec![
            format!(r"(?m)^\s*function\s+{}\s*\(", n),
            format!(r"(?m)^\s*fn\s+{}\s*\(", n),
            format!(r"(?m)^\s*def\s+{}\s*\(", n),
        ],
    };

    let mut start_match: Option<usize> = None;
    for p in &patterns {
        let re = Regex::new(p).map_err(|e| anyhow::anyhow!("Invalid regex {}: {}", p, e))?;
        if let Some(m) = re.find(content) {
            start_match = Some(m.start());
            break;
        }
    }

    let start = start_match.ok_or_else(||
        anyhow::anyhow!("Function '{}' not found in file", name)
    )?;

    // Find end of function by matching braces (for brace-based langs)
    // For Python: match by indentation
    let end = match ext {
        "py" => find_end_python(content, start)?,
        _ => find_end_braces(content, start)?,
    };

    Ok((start, end))
}

fn find_end_braces(content: &str, start: usize) -> Result<usize> {
    let bytes = content.as_bytes();
    let mut i = start;
    // Find first opening brace after start
    while i < bytes.len() && bytes[i] != b'{' { i += 1; }
    if i >= bytes.len() {
        anyhow::bail!("No opening brace found after function signature");
    }
    let mut depth = 0;
    let mut in_str = false;
    let mut str_ch: u8 = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < bytes.len() {
        let c = bytes[i];
        // Handle escape in strings
        if in_str {
            if c == b'\\' && i + 1 < bytes.len() { i += 2; continue; }
            if c == str_ch { in_str = false; }
            i += 1;
            continue;
        }
        if in_line_comment {
            if c == b'\n' { in_line_comment = false; }
            i += 1;
            continue;
        }
        if in_block_comment {
            if c == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        // Detect start of string / comment
        if c == b'"' || c == b'\'' || c == b'`' {
            in_str = true;
            str_ch = c;
            i += 1;
            continue;
        }
        if c == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' { in_line_comment = true; i += 2; continue; }
            if bytes[i + 1] == b'*' { in_block_comment = true; i += 2; continue; }
        }
        if c == b'{' { depth += 1; }
        else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                // Include the closing brace + trailing newline
                let mut end = i + 1;
                if end < bytes.len() && bytes[end] == b'\n' { end += 1; }
                return Ok(end);
            }
        }
        i += 1;
    }
    anyhow::bail!("Unbalanced braces in function body")
}

fn find_end_python(content: &str, start: usize) -> Result<usize> {
    // Find the indentation of the def line
    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let mut byte_pos = 0usize;
    let mut def_line_idx = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if byte_pos <= start && start < byte_pos + line.len() {
            def_line_idx = i;
            break;
        }
        byte_pos += line.len();
    }
    let def_line = lines[def_line_idx];
    let def_indent = def_line.len() - def_line.trim_start().len();

    // Find first line after def_line whose indent <= def_indent AND is non-blank
    let mut end_pos = byte_pos + def_line.len();
    for line in lines.iter().skip(def_line_idx + 1) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            end_pos += line.len();
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent <= def_indent {
            return Ok(end_pos);
        }
        end_pos += line.len();
    }
    Ok(end_pos)
}
