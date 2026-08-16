use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct CondenseArgs {
    file: PathBuf,
    /// How aggressive to condense
    #[arg(short = 'l', long, default_value = "medium")]
    level: Level,
    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Level {
    Light,
    Medium,
    Aggressive,
}

pub fn run(args: CondenseArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let original_lines = content.lines().count();
    let original_bytes = content.len();
    let result = condense(&content, args.level);
    let new_lines = result.lines().count();
    let new_bytes = result.len();

    match args.output {
        Some(p) => {
            std::fs::write(&p, &result)?;
            eprintln!("{} {}  ({} → {} lines, {} → {} bytes, {:.0}% saved)",
                "Wrote:".green().bold(),
                p.display().to_string().cyan(),
                original_lines, new_lines,
                original_bytes, new_bytes,
                (1.0 - new_bytes as f64 / original_bytes as f64) * 100.0);
        }
        None => {
            print!("{}", result);
            eprintln!("\n{} {} → {} lines, {} → {} bytes ({:.0}% saved)",
                "Condensed:".green().bold(),
                original_lines, new_lines,
                original_bytes, new_bytes,
                (1.0 - new_bytes as f64 / original_bytes as f64) * 100.0);
        }
    }
    Ok(())
}

pub fn condense(content: &str, level: Level) -> String {
    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len());
    let mut in_string: Option<u8> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut i = 0;

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
        if let Some(q) = in_string {
            out.push(c as char);
            if c == b'\\' && i + 1 < bytes.len() { out.push(bytes[i + 1] as char); i += 2; continue; }
            if c == q { in_string = None; }
            i += 1; continue;
        }
        match (c, next) {
            (b'/', Some(b'/')) if !matches!(level, Level::Light) => { in_line_comment = true; i += 2; }
            (b'/', Some(b'*')) if !matches!(level, Level::Light) => { in_block_comment = true; i += 2; }
            (b'#', _) if is_line_start(&out) && !matches!(level, Level::Light) => {
                in_line_comment = true; i += 1;
            }
            (b'"' | b'\'' | b'`', _) => { in_string = Some(c); out.push(c as char); i += 1; }
            _ => { out.push(c as char); i += 1; }
        }
    }

    // Collapse blank lines
    let text = out;
    let mut result: Vec<String> = Vec::new();
    let mut blank_run = 0;
    let max_blanks = match level {
        Level::Light => 2,
        Level::Medium => 1,
        Level::Aggressive => 0,
    };
    for line in text.lines() {
        let stripped = if matches!(level, Level::Aggressive) { line.trim().to_string() } else { line.trim_end().to_string() };
        if stripped.is_empty() {
            blank_run += 1;
            if blank_run <= max_blanks { result.push(stripped); }
        } else {
            blank_run = 0;
            result.push(stripped);
        }
    }

    let mut joined = result.join("\n");
    if !joined.ends_with('\n') { joined.push('\n'); }
    joined
}

fn is_line_start(s: &str) -> bool {
    match s.chars().last() {
        None => true,
        Some(c) => c == '\n',
    }
}
