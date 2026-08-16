use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct VerifyJsonArgs {
    files: Vec<PathBuf>,

    /// Show format info (compact vs pretty, size)
    #[arg(short = 'f', long)]
    format_info: bool,

    /// Accept JSON5-style comments and trailing commas (tsconfig-friendly)
    #[arg(short = 'L', long)]
    lenient: bool,
}

pub fn run(args: VerifyJsonArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("At least one file required"); }
    let mut ok_count = 0usize;
    let mut fail_count = 0usize;
    for f in &args.files {
        if !f.exists() { println!("  {} {}", "MISSING".red(), f.display()); fail_count += 1; continue; }
        let raw = read_file_smart(f)?;
        let content = if args.lenient { strip_json5(&raw) } else { raw.clone() };
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(v) => {
                ok_count += 1;
                let info = if args.format_info {
                    let is_pretty = content.contains("\n  ") || content.contains("\n    ");
                    format!(" ({}, {} bytes)", if is_pretty { "pretty" } else { "compact" }, content.len())
                } else { String::new() };
                let kind = match &v {
                    serde_json::Value::Object(o) => format!("object, {} keys", o.len()),
                    serde_json::Value::Array(a) => format!("array, {} items", a.len()),
                    _ => "value".to_string(),
                };
                let tag = if args.lenient { " (lenient)".dimmed().to_string() } else { String::new() };
                println!("  {} {}  ({}){}{}", "OK".green().bold(), f.display().to_string().cyan(), kind.dimmed(), info.dimmed(), tag);
            }
            Err(e) => {
                fail_count += 1;
                let hint = if !args.lenient && (raw.contains("//") || raw.contains("/*")) {
                    "  (has comments — retry with --lenient)".yellow().to_string()
                } else { String::new() };
                println!("  {} {}  {}{}", "INVALID".red().bold(), f.display().to_string().cyan(), e.to_string().dimmed(), hint);
            }
        }
    }
    println!("\n{} {} ok, {} invalid", "Summary:".bold(), ok_count.to_string().green(), fail_count.to_string().red());
    if fail_count > 0 { std::process::exit(1); }
    Ok(())
}

/// Strip JSON5-style // and /* */ comments, and remove trailing commas before ] or }.
/// Keeps strings intact.
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

        if in_line_comment {
            if c == b'\n' { in_line_comment = false; out.push('\n'); }
            i += 1; continue;
        }
        if in_block_comment {
            if c == b'*' && next == Some(b'/') { in_block_comment = false; i += 2; continue; }
            i += 1; continue;
        }
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

    // Second pass: remove trailing commas (,\s*[}\]])
    let re = regex::Regex::new(r",(\s*[}\]])").unwrap();
    re.replace_all(&out, "$1").to_string()
}
