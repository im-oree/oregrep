use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct VerifyImportsArgs {
    files: Vec<PathBuf>,

    /// Also try resolving with common extensions (.ts, .tsx, .js, .jsx, /index.ts, etc.)
    #[arg(short = 'r', long, default_value = "true")]
    resolve_ext: bool,
}

pub fn run(args: VerifyImportsArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("At least one file required"); }
    let import_re = regex::Regex::new(r#"(?m)^\s*(?:import|export).*?from\s+['"]([^'"]+)['"]"#).unwrap();
    let mut total_imports = 0usize;
    let mut missing = 0usize;

    for f in &args.files {
        if !f.exists() { println!("  {} {}", "MISSING".red(), f.display()); continue; }
        let content = read_file_smart(f)?;
        let dir = f.parent().unwrap_or(std::path::Path::new("."));
        let mut file_missing = Vec::new();
        for cap in import_re.captures_iter(&content) {
            let raw = &cap[1];
            total_imports += 1;
            // Skip node_modules (starts with letter/@)
            if !raw.starts_with('.') && !raw.starts_with('/') { continue; }
            let base = dir.join(raw);
            let resolved = if base.exists() { Some(base.clone()) }
                else if args.resolve_ext {
                    let candidates = [
                        base.with_extension("ts"),
                        base.with_extension("tsx"),
                        base.with_extension("js"),
                        base.with_extension("jsx"),
                        base.with_extension("mjs"),
                        base.join("index.ts"),
                        base.join("index.tsx"),
                        base.join("index.js"),
                    ];
                    candidates.iter().find(|p| p.exists()).cloned()
                } else { None };
            if resolved.is_none() {
                missing += 1;
                file_missing.push(raw.to_string());
            }
        }
        if file_missing.is_empty() {
            println!("  {} {}", "OK".green().bold(), f.display().to_string().cyan());
        } else {
            println!("  {} {}", "MISS".red().bold(), f.display().to_string().cyan());
            for m in file_missing {
                println!("    {} {}", "✗".red(), m.yellow());
            }
        }
    }
    println!("\n{} {} imports checked, {} unresolved",
        "Summary:".bold(),
        total_imports.to_string().yellow(),
        missing.to_string().red());
    if missing > 0 { std::process::exit(1); }
    Ok(())
}
