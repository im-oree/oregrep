use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::symbols::{extract_imports, resolve_ts_import, collect_source_files};
use crate::engine::walker::{parse_excludes, parse_extensions};

#[derive(Args)]
pub struct MoveWithImportsArgs {
    /// Source file
    src: PathBuf,

    /// Destination path (file or dir)
    dst: PathBuf,

    /// Root path to scan for importers
    #[arg(short = 'r', long, default_value = ".")]
    root: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    #[arg(long)]
    no_backup: bool,

    #[arg(short = 'l', long)]
    label: Option<String>,

    #[arg(long)]
    dry_run: bool,

    #[arg(short = 'y', long)]
    yes: bool,
}

pub fn run(args: MoveWithImportsArgs) -> Result<()> {
    if !args.src.exists() { anyhow::bail!("Source not found: {}", args.src.display()); }

    let dst_file = if args.dst.is_dir() || (!args.dst.exists() && args.dst.extension().is_none()) {
        let fname = args.src.file_name().ok_or_else(|| anyhow::anyhow!("Invalid src filename"))?;
        args.dst.join(fname)
    } else {
        args.dst.clone()
    };

    if dst_file == args.src {
        anyhow::bail!("Destination equals source");
    }
    if dst_file.exists() {
        anyhow::bail!("Destination file exists: {}", dst_file.display());
    }

    let src_abs = std::fs::canonicalize(&args.src).map(|p| strip_prefix(&p))?;
    // Predict destination absolute path (won't exist yet)
    let dst_parent = dst_file.parent().unwrap_or_else(|| std::path::Path::new("."));
    let dst_parent_abs = if dst_parent.exists() {
        std::fs::canonicalize(dst_parent).map(|p| strip_prefix(&p))?
    } else {
        // Compose from current dir
        let cwd = std::env::current_dir()?;
        strip_prefix(&cwd.join(dst_parent))
    };
    let dst_abs = dst_parent_abs.join(dst_file.file_name().unwrap());

    // Find all importers in the root
    let ext = args.ext.as_deref().map(parse_extensions).unwrap_or_else(|| vec!["ts".into(), "tsx".into(), "js".into(), "jsx".into()]);
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();
    let all_files = collect_source_files(&args.root, &ext, &exc)?;

    let mut edits: Vec<(PathBuf, Vec<(usize, String, String)>)> = Vec::new(); // (file, [(line, old, new)])

    for (p, c) in &all_files {
        let p_abs = std::fs::canonicalize(p).map(|x| strip_prefix(&x)).unwrap_or_else(|_| p.clone());
        if p_abs == src_abs { continue; } // Will be moved — no self-edit
        let imports = extract_imports(c, p);
        let mut file_edits: Vec<(usize, String, String)> = Vec::new();
        for imp in &imports {
            if let Some(resolved) = resolve_ts_import(p, &imp.source) {
                if let Ok(resolved_abs) = std::fs::canonicalize(&resolved) {
                    if strip_prefix(&resolved_abs) == src_abs {
                        // Compute new import path from p to dst_abs
                        let new_path = compute_relative(&p_abs, &dst_abs);
                        if new_path != imp.source {
                            file_edits.push((imp.line, imp.source.clone(), new_path));
                        }
                    }
                }
            }
        }
        if !file_edits.is_empty() { edits.push((p.clone(), file_edits)); }
    }

    println!("{} {} → {}",
        "Moving:".cyan().bold(),
        args.src.display().to_string().yellow(),
        dst_file.display().to_string().green());
    println!("  {} importers to update: {}", "→".dimmed(), edits.len().to_string().yellow());
    for (f, es) in &edits {
        println!("  {} {}", "~".yellow(), f.display().to_string().cyan());
        for (line, old, new) in es {
            println!("      L{}: {} → {}", line.to_string().dimmed(), old.red(), new.green());
        }
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing changed]".yellow().bold());
        return Ok(());
    }

    if !args.yes {
        let ok = crate::engine::confirm::confirm(&format!("Move file + update {} importers?", edits.len()), false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    // Backup source
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.src, &label)?;
        println!("{} {}", "Backup src:".dimmed(), bak.display().to_string().dimmed());
    }

    // Ensure dst parent exists
    if let Some(parent) = dst_file.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Move (rename or copy+delete)
    std::fs::rename(&args.src, &dst_file).or_else(|_| {
        std::fs::copy(&args.src, &dst_file)?;
        std::fs::remove_file(&args.src)
    })?;

    // Apply edits to each importer
    for (f, es) in &edits {
        let content = read_file_smart(f)?;
        let mut new_content = content.clone();
        for (_line, old, new) in es {
            // Only replace within import context: quoted string
            let patterns = [format!("'{}'", old), format!("\"{}\"", old)];
            let repls = [format!("'{}'", new), format!("\"{}\"", new)];
            for (pat, rep) in patterns.iter().zip(repls.iter()) {
                if new_content.contains(pat) {
                    new_content = new_content.replace(pat, rep);
                }
            }
        }
        if new_content != content {
            if !args.no_backup {
                let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
                let _ = create_backup(f, &label);
            }
            write_atomic(f, &new_content, content.starts_with('\u{FEFF}'))?;
        }
    }

    println!("\n{} moved + {} importers updated",
        "Done:".green().bold(),
        edits.len().to_string().yellow());
    Ok(())
}

fn strip_prefix(p: &std::path::Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") { PathBuf::from(stripped) } else { p.to_path_buf() }
}

fn compute_relative(from_file: &std::path::Path, to_file: &std::path::Path) -> String {
    let from_dir = from_file.parent().unwrap_or_else(|| std::path::Path::new(""));
    let to_stem = to_file.with_extension("");
    let f_parts: Vec<_> = from_dir.components().collect();
    let t_parts: Vec<_> = to_stem.components().collect();
    let mut i = 0;
    while i < f_parts.len() && i < t_parts.len() && f_parts[i] == t_parts[i] { i += 1; }
    let ups = f_parts.len() - i;
    let mut rel = PathBuf::new();
    for _ in 0..ups { rel.push(".."); }
    for c in &t_parts[i..] { rel.push(c); }
    let mut s = rel.to_string_lossy().replace('\\', "/");
    if !s.starts_with('.') { s = format!("./{}", s); }
    s
}
