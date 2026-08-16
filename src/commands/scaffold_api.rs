use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct ScaffoldApiArgs {
    name: String,
    #[arg(short = 'o', long, default_value = "src/lib/api")]
    out_dir: PathBuf,

    /// Base URL
    #[arg(short = 'u', long, default_value = "/api")]
    base_url: String,
}

pub fn run(args: ScaffoldApiArgs) -> Result<()> {
    std::fs::create_dir_all(&args.out_dir)?;
    let f = args.out_dir.join(format!("{}.ts", args.name.to_lowercase()));
    let n = capitalize(&args.name);
    let content = format!(
        "const BASE = '{base}';\n\nexport interface {n} {{\n  id: string;\n}}\n\nexport async function list{n}s(): Promise<{n}[]> {{\n  const r = await fetch(`${{BASE}}/{lower}s`);\n  return r.json();\n}}\n\nexport async function get{n}(id: string): Promise<{n}> {{\n  const r = await fetch(`${{BASE}}/{lower}s/${{id}}`);\n  return r.json();\n}}\n\nexport async function create{n}(data: Partial<{n}>): Promise<{n}> {{\n  const r = await fetch(`${{BASE}}/{lower}s`, {{ method: 'POST', headers: {{ 'Content-Type': 'application/json' }}, body: JSON.stringify(data) }});\n  return r.json();\n}}\n",
        base = args.base_url, n = n, lower = args.name.to_lowercase());
    std::fs::write(&f, content)?;
    println!("  {} {}", "+".green(), f.display().to_string().dimmed());
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
