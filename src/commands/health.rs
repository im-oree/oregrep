use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::encoding::{is_binary, read_file_smart};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct HealthArgs {
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Extensions to include (comma-separated)
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Excludes
    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: HealthArgs) -> Result<()> {
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_default(),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;

    let mut total_lines = 0usize;
    let mut total_bytes = 0u64;
    let mut by_ext: HashMap<String, (usize, usize)> = HashMap::new();
    let mut todos = 0usize;
    let mut fixmes = 0usize;
    let mut hacks = 0usize;
    let mut any_types = 0usize;
    let mut unwraps = 0usize;
    let mut console_logs = 0usize;
    let mut binary_files = 0usize;

    // Meta-file presence uses the raw path (walker skips gitignored files, so we check filesystem)
    let has_readme = std::fs::read_dir(&args.path)
        .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().to_lowercase().starts_with("readme")))
        .unwrap_or(false);
    let has_gitignore = args.path.join(".gitignore").exists();
    let has_license = std::fs::read_dir(&args.path)
        .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().to_lowercase().starts_with("license")))
        .unwrap_or(false);

    for f in &files {
        if let Ok(sz) = std::fs::metadata(f).map(|m| m.len()) { total_bytes += sz; }
        if is_binary(f).unwrap_or(false) { binary_files += 1; continue; }
        let content = match read_file_smart(f) { Ok(c) => c, Err(_) => continue };
        let lc = content.lines().count();
        total_lines += lc;
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let e = by_ext.entry(ext.clone()).or_insert((0, 0));
        e.0 += 1; e.1 += lc;

        for line in content.lines() {
            if line.contains("TODO") { todos += 1; }
            if line.contains("FIXME") { fixmes += 1; }
            if line.contains("HACK") { hacks += 1; }
            if ext == "ts" || ext == "tsx" {
                if line.contains(": any") || line.contains("<any>") { any_types += 1; }
            }
            if ext == "rs" && line.contains(".unwrap()") { unwraps += 1; }
            if (ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx") && line.contains("console.log") { console_logs += 1; }
        }
    }

    if args.json {
        let mut obj = serde_json::Map::new();
        obj.insert("files".into(), files.len().into());
        obj.insert("total_lines".into(), total_lines.into());
        obj.insert("total_bytes".into(), total_bytes.into());
        obj.insert("todos".into(), todos.into());
        obj.insert("fixmes".into(), fixmes.into());
        obj.insert("hacks".into(), hacks.into());
        obj.insert("any_types".into(), any_types.into());
        obj.insert("unwraps".into(), unwraps.into());
        obj.insert("console_logs".into(), console_logs.into());
        obj.insert("binary_files".into(), binary_files.into());
        obj.insert("has_readme".into(), has_readme.into());
        obj.insert("has_gitignore".into(), has_gitignore.into());
        obj.insert("has_license".into(), has_license.into());
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    println!("{} {}", "Codebase health:".cyan().bold(), args.path.display().to_string().yellow());
    println!("\n{}", "Size:".bold());
    println!("  Files: {}", files.len().to_string().yellow());
    println!("  Lines: {}", total_lines.to_string().yellow());
    println!("  Bytes: {}", format_size(total_bytes).yellow());
    println!("  Binary files: {}", binary_files.to_string().dimmed());

    println!("\n{}", "By extension:".bold());
    let mut entries: Vec<_> = by_ext.iter().collect();
    entries.sort_by(|a, b| b.1.1.cmp(&a.1.1));
    for (ext, (f, l)) in entries.iter().take(10) {
        println!("  {:<8} {:>6} files  {:>10} lines", ext.cyan(), f.to_string().yellow(), l.to_string().green());
    }

    println!("\n{}", "Comments/markers:".bold());
    println!("  TODO: {}", color_count(todos, 20, 50));
    println!("  FIXME: {}", color_count(fixmes, 5, 20));
    println!("  HACK: {}", color_count(hacks, 3, 10));

    println!("\n{}", "Code smells:".bold());
    println!("  `any` types (TS): {}", color_count(any_types, 5, 30));
    println!("  `.unwrap()` (Rust): {}", color_count(unwraps, 20, 100));
    println!("  `console.log`: {}", color_count(console_logs, 5, 30));

    println!("\n{}", "Project files:".bold());
    println!("  README: {}", bool_indicator(has_readme));
    println!("  .gitignore: {}", bool_indicator(has_gitignore));
    println!("  LICENSE: {}", bool_indicator(has_license));

    let score = compute_score(has_readme, has_gitignore, todos, any_types, unwraps, console_logs);
    let color = if score >= 80 { "green" } else if score >= 60 { "yellow" } else { "red" };
    println!("\n{} {} / 100", "Score:".bold(), score.to_string().color(color).bold());

    Ok(())
}

fn color_count(n: usize, warn: usize, alert: usize) -> String {
    if n >= alert { n.to_string().red().to_string() }
    else if n >= warn { n.to_string().yellow().to_string() }
    else { n.to_string().green().to_string() }
}

fn bool_indicator(b: bool) -> String {
    if b { "yes".green().to_string() } else { "no".red().to_string() }
}

fn compute_score(readme: bool, gi: bool, todos: usize, anys: usize, unwraps: usize, logs: usize) -> u32 {
    let mut s: i32 = 100;
    if !readme { s -= 10; }
    if !gi { s -= 5; }
    s -= (todos as i32 / 5).min(15);
    s -= (anys as i32 / 3).min(15);
    s -= (unwraps as i32 / 10).min(10);
    s -= (logs as i32 / 5).min(10);
    s.max(0) as u32
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}
