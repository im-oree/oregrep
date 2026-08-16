use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct MergeFilesArgs {
    /// Files to merge (in order)
    #[arg(required = true, num_args = 1..)]
    files: Vec<PathBuf>,

    /// Output file
    #[arg(short = 'o', long, required = true)]
    output: PathBuf,

    /// Include a header comment before each file's content
    #[arg(short = 'H', long, default_value = "true")]
    headers: bool,

    /// Deduplicate identical import lines at the top of each file
    #[arg(short = 'd', long, default_value = "true")]
    dedup_imports: bool,

    /// Skip empty files
    #[arg(short = 's', long, default_value = "true")]
    skip_empty: bool,

    #[arg(long)]
    force: bool,

    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: MergeFilesArgs) -> Result<()> {
    if args.output.exists() && !args.force {
        anyhow::bail!("Output exists: {} (use --force)", args.output.display());
    }

    let mut merged_imports: Vec<String> = Vec::new();
    let mut seen_imports: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut bodies: Vec<(PathBuf, String)> = Vec::new();

    for f in &args.files {
        if !f.exists() { anyhow::bail!("File not found: {}", f.display()); }
        let content = read_file_smart(f)?;
        if args.skip_empty && content.trim().is_empty() { continue; }

        if args.dedup_imports {
            let mut body_start = 0usize;
            for (i, line) in content.lines().enumerate() {
                let t = line.trim_start();
                if t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") {
                    if seen_imports.insert(line.to_string()) {
                        merged_imports.push(line.to_string());
                    }
                    body_start = i + 1;
                } else if t.is_empty() && body_start > 0 {
                    body_start = i + 1;
                } else if !t.is_empty() {
                    break;
                }
            }
            let body: String = content.lines().skip(body_start).collect::<Vec<_>>().join("\n");
            bodies.push((f.clone(), body));
        } else {
            bodies.push((f.clone(), content));
        }
    }

    let mut out = String::new();
    if !merged_imports.is_empty() {
        for line in &merged_imports { out.push_str(line); out.push('\n'); }
        out.push('\n');
    }
    for (path, body) in &bodies {
        if args.headers {
            out.push_str(&format!("// ===== {} =====\n\n", path.display()));
        }
        out.push_str(body.trim());
        out.push_str("\n\n");
    }

    println!("{} {} files → {}",
        "Merging:".cyan().bold(),
        bodies.len().to_string().yellow(),
        args.output.display().to_string().green());
    if args.dedup_imports && !merged_imports.is_empty() {
        println!("  {} {} unique imports collected", "→".dimmed(), merged_imports.len().to_string().yellow());
    }
    println!("  {} output size: {} bytes", "→".dimmed(), out.len().to_string().yellow());

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing written]".yellow().bold());
        return Ok(());
    }

    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&args.output, out)?;
    println!("\n{} {}", "Done:".green().bold(), args.output.display().to_string().cyan());
    Ok(())
}
