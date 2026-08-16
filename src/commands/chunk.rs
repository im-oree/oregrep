use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::function_bodies;
use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_symbols, SymbolKind};

#[derive(Args)]
pub struct ChunkArgs {
    file: PathBuf,
    /// Chunk boundary strategy
    #[arg(short = 'b', long, default_value = "function")]
    by: ChunkBy,
    /// Output directory (default: "<stem>-chunks/")
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,
    /// Also write a manifest (chunks.json) listing all chunks with metadata
    #[arg(long, default_value = "true")]
    manifest: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ChunkBy {
    Function,
    Class,
    Export,
    Section,
}

pub fn run(args: ChunkArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let stem = args.file.file_stem().and_then(|s| s.to_str()).unwrap_or("chunk").to_string();
    let out_dir = args.output_dir.clone().unwrap_or_else(|| args.file.parent().unwrap_or(std::path::Path::new(".")).join(format!("{}-chunks", stem)));

    let chunks: Vec<(String, String, usize)> = match args.by {
        ChunkBy::Function => function_bodies(&content),
        ChunkBy::Class | ChunkBy::Export => {
            let syms = extract_symbols(&content, &args.file);
            let mut out = Vec::new();
            for s in syms.iter() {
                let want = match args.by {
                    ChunkBy::Class => matches!(s.kind, SymbolKind::Class | SymbolKind::Struct | SymbolKind::Interface),
                    ChunkBy::Export => s.exported,
                    _ => false,
                };
                if !want { continue; }
                if let Some((_, _, body)) = crate::engine::symbols::extract_body(&content, &s.name) {
                    out.push((s.name.clone(), body, s.line));
                }
            }
            out
        }
        ChunkBy::Section => split_by_section_comments(&content),
    };

    if chunks.is_empty() {
        anyhow::bail!("No chunks found using strategy {:?}", args.by);
    }

    println!("{} {} → {} chunks",
        "Chunking:".cyan().bold(),
        args.file.display().to_string().yellow(),
        chunks.len().to_string().green());

    for (name, _, line) in &chunks {
        println!("  {} {}  (L{})", "+".green(), name.cyan(), line.to_string().dimmed());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — no files written]".yellow().bold());
        return Ok(());
    }

    std::fs::create_dir_all(&out_dir)?;
    let mut manifest: Vec<serde_json::Value> = Vec::new();
    for (name, body, line) in &chunks {
        let fname = format!("{}.chunk", sanitize(name));
        let path = out_dir.join(&fname);
        std::fs::write(&path, body)?;
        manifest.push(serde_json::json!({
            "name": name,
            "line": line,
            "file": fname,
            "bytes": body.len(),
            "lines": body.lines().count(),
        }));
    }

    if args.manifest {
        let mpath = out_dir.join("chunks.json");
        std::fs::write(&mpath, serde_json::to_string_pretty(&manifest)?)?;
        println!("\n{} {}", "Manifest:".dimmed(), mpath.display().to_string().dimmed());
    }
    println!("\n{} {} chunks written to {}",
        "Done:".green().bold(),
        chunks.len().to_string().green(),
        out_dir.display().to_string().cyan());
    Ok(())
}

fn sanitize(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect()
}

fn split_by_section_comments(content: &str) -> Vec<(String, String, usize)> {
    // Split on lines that look like: // === SECTION NAME === or // SECTION: name
    let re = regex::Regex::new(r"(?m)^\s*//\s*(?:={2,}|SECTION[:\s]).*").unwrap();
    let mut boundaries: Vec<(usize, String)> = Vec::new();
    for m in re.find_iter(content) {
        let line_no = content[..m.start()].matches('\n').count() + 1;
        let title = m.as_str().trim().trim_start_matches('/').trim().trim_matches('=').trim().replace("SECTION:", "").trim().to_string();
        let title = if title.is_empty() { format!("section-{}", boundaries.len() + 1) } else { title };
        boundaries.push((line_no, title));
    }
    if boundaries.is_empty() { return vec![]; }
    let lines: Vec<&str> = content.lines().collect();
    let mut out = Vec::new();
    for (i, (start, title)) in boundaries.iter().enumerate() {
        let end = boundaries.get(i + 1).map(|(s, _)| *s - 1).unwrap_or(lines.len());
        let body = lines[start - 1..end].join("\n");
        out.push((title.clone(), body, *start));
    }
    out
}
