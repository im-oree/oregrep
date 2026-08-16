use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct EnvSetArgs {
    file: PathBuf,
    key: String,
    value: String,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    /// Delete the key instead of setting it
    #[arg(long)]
    delete: bool,
}

pub fn run(args: EnvSetArgs) -> Result<()> {
    let content = if args.file.exists() { read_file_smart(&args.file)? } else { String::new() };
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let key_prefix = format!("{}=", args.key);
    let mut replaced = false;
    let mut deleted = false;
    let mut new_lines: Vec<String> = Vec::new();
    for l in &lines {
        let trimmed = l.trim_start();
        if trimmed.starts_with(&key_prefix) || trimmed == args.key.as_str() {
            if args.delete { deleted = true; continue; }
            new_lines.push(format!("{}={}", args.key, escape_env_value(&args.value)));
            replaced = true;
        } else {
            new_lines.push(l.clone());
        }
    }
    if !replaced && !args.delete {
        new_lines.push(format!("{}={}", args.key, escape_env_value(&args.value)));
    }
    lines = new_lines;
    let mut out = lines.join("\n");
    if !out.ends_with('\n') { out.push('\n'); }

    if args.file.exists() && !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    if let Some(parent) = args.file.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() { std::fs::create_dir_all(parent)?; }
    }
    std::fs::write(&args.file, out)?;
    if args.delete {
        if deleted { println!("{} {} removed from {}", "Deleted:".green().bold(), args.key.cyan(), args.file.display()); }
        else { println!("{} {} was not present", "Noop:".yellow(), args.key.cyan()); }
    } else {
        println!("{} {} = {}", "Set:".green().bold(), args.key.cyan(), args.value.yellow());
    }
    Ok(())
}

fn escape_env_value(v: &str) -> String {
    if v.chars().any(|c| c == ' ' || c == '#' || c == '"' || c == '\n') {
        format!("\"{}\"", v.replace('"', "\\\""))
    } else {
        v.to_string()
    }
}
