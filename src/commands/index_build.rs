use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::index::{ensure_gitignore_entry, file_count, hash_file, import_count, insert_import,
    insert_symbol, open_index, resolve_db_path, set_meta, symbol_count, upsert_file};
use crate::engine::paths::canonicalize_clean;
use crate::engine::progress::Progress;
use crate::engine::symbols::{extract_imports, extract_symbols, language_of, resolve_ts_import};
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct IndexBuildArgs {
    #[arg(default_value = ".")]
    root: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Force full rebuild even if index exists
    #[arg(short = 'f', long)]
    force: bool,

    /// Add .ore-index/ to .gitignore automatically
    #[arg(long, default_value = "true")]
    gitignore: bool,
}

pub fn run(args: IndexBuildArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    if !root_abs.exists() { anyhow::bail!("Path not found: {}", args.root.display()); }
    println!("{} {}", "Indexing:".cyan().bold(), root_abs.display().to_string().yellow());

    let db_path = resolve_db_path(&root_abs)?;
    if db_path.exists() && !args.force {
        println!("{}", "Index exists. Use --force to rebuild, or `ore index-update` for incremental.".yellow());
        return Ok(());
    }
    if db_path.exists() && args.force {
        std::fs::remove_file(&db_path).ok();
    }

    if args.gitignore {
        if let Some(ws) = crate::engine::paths::workspace_root(&root_abs) {
            let _ = ensure_gitignore_entry(&ws);
        }
    }

    let conn = open_index(&root_abs)?;
    set_meta(&conn, "root", &root_abs.to_string_lossy())?;
    set_meta(&conn, "built_at", &chrono::Local::now().timestamp().to_string())?;

    let cfg = WalkConfig {
        root: root_abs.clone(),
        extensions: args.ext.as_deref().map(parse_extensions).unwrap_or_else(||
            vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()]),
        excludes: args.exclude.as_deref().map(parse_excludes).unwrap_or_default(),
        skip_backups: true,
        ..Default::default()
    };
    let files = collect_files(&cfg)?;
    let total = files.len() as u64;
    let pb = Progress::bar(total, "indexing");

    let mut file_ids: Vec<(PathBuf, i64)> = Vec::with_capacity(files.len());
    conn.execute_batch("BEGIN;")?;
    for f in &files {
        pb.inc(1);
        let meta = match std::fs::metadata(f) { Ok(m) => m, Err(_) => continue };
        let size = meta.len() as i64;
        let mtime = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64).unwrap_or(0);
        let hash = match hash_file(f) { Ok(h) => h, Err(_) => continue };
        let lang = language_of(f);
        let path_str = f.to_string_lossy().to_string();
        let (file_id, _changed) = upsert_file(&conn, &path_str, size, mtime, &hash, Some(lang))?;
        file_ids.push((f.clone(), file_id));

        // Symbols
        if let Ok(content) = read_file_smart(f) {
            for s in extract_symbols(&content, f) {
                let kind_str = format!("{:?}", s.kind).to_lowercase();
                insert_symbol(&conn, file_id, &s.name, &kind_str, s.line as i64, s.column as i64, s.exported)?;
            }
        }
    }
    // Second pass: resolve imports (needs all file_ids known first). Keys and
    // lookups are canonicalized so walker paths and resolved import paths match.
    let path_to_id: HashMap<PathBuf, i64> = file_ids.iter()
        .map(|(p, id)| (canonicalize_clean(p), *id))
        .collect();
    for (path, file_id) in &file_ids {
        if let Ok(content) = read_file_smart(path) {
            for imp in extract_imports(&content, path) {
                let resolved_id = resolve_ts_import(path, &imp.source)
                    .and_then(|resolved| path_to_id.get(&canonicalize_clean(&resolved)).copied());
                let named = imp.named.join(",");
                let _ = insert_import(&conn, *file_id, &imp.source, resolved_id, &named, imp.line as i64);
            }
        }
    }
    conn.execute_batch("COMMIT;")?;
    pb.finish("done");

    let fc = file_count(&conn)?;
    let sc = symbol_count(&conn)?;
    let ic = import_count(&conn)?;
    println!("\n{} {}\n  files:   {}\n  symbols: {}\n  imports: {}",
        "Index built:".green().bold(),
        db_path.display().to_string().cyan(),
        fc.to_string().yellow(),
        sc.to_string().yellow(),
        ic.to_string().yellow());
    Ok(())
}
