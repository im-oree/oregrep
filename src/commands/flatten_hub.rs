use anyhow::Result;
use clap::Args;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::symbols::resolve_ts_import;

#[derive(Args)]
pub struct FlattenHubArgs {
    /// Hub barrel file
    hub: PathBuf,

    /// Include imports from each source into flattened file
    #[arg(short = 'i', long, default_value = "true")]
    carry_imports: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: FlattenHubArgs) -> Result<()> {
    if !args.hub.exists() { anyhow::bail!("Hub not found: {}", args.hub.display()); }
    let content = read_file_smart(&args.hub)?;
    let re = Regex::new(r#"(?m)^export\s+(?:\{[^}]*\}|\*)\s+from\s+['"]([^'"]+)['"];?"#)?;

    let mut sources: Vec<PathBuf> = Vec::new();
    for cap in re.captures_iter(&content) {
        let src = &cap[1];
        if let Some(resolved) = resolve_ts_import(&args.hub, src) {
            sources.push(resolved);
        }
    }

    if sources.is_empty() {
        anyhow::bail!("No re-exports found in {}", args.hub.display());
    }

    // Concat with headers
    let mut merged_imports: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut bodies: Vec<String> = Vec::new();

    for s in &sources {
        let c = read_file_smart(s)?;
        let mut body_start = 0usize;
        if args.carry_imports {
            for (i, line) in c.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with("import ") || t.starts_with("use ") {
                    if seen.insert(line.to_string()) { merged_imports.push(line.to_string()); }
                    body_start = i + 1;
                } else if t.is_empty() && body_start > 0 { body_start = i + 1; }
                else if !t.is_empty() { break; }
            }
        }
        let body = c.lines().skip(body_start).collect::<Vec<_>>().join("\n");
        bodies.push(format!("// ===== {} =====\n\n{}\n", s.display(), body.trim()));
    }

    let mut out = String::new();
    for i in &merged_imports { out.push_str(i); out.push('\n'); }
    if !merged_imports.is_empty() { out.push('\n'); }
    for b in &bodies { out.push_str(b); out.push('\n'); }

    println!("{} {} ← {} source files",
        "Flattening:".cyan().bold(),
        args.hub.display().to_string().yellow(),
        sources.len().to_string().green());
    for s in &sources {
        let display = crate::engine::paths::canonicalize_clean(s);
        println!("  {} {}", "+".green(), display.display().to_string().dimmed());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing written]".yellow().bold());
        return Ok(());
    }
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.hub, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    write_atomic(&args.hub, &out, content.starts_with('\u{FEFF}'))?;
    println!("\n{} {}", "Done:".green().bold(), args.hub.display().to_string().cyan());
    Ok(())
}
