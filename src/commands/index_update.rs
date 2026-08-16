use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::engine::encoding::read_file_smart;
use crate::engine::index::{all_files, file_count, file_id_for, hash_file, insert_import, insert_symbol,
    open_index_if_exists, set_meta, stale_files, upsert_file};
use crate::engine::paths::canonicalize_clean;
use crate::engine::symbols::{extract_imports, extract_symbols, language_of, resolve_ts_import};
use crate::engine::walker::{collect_files, WalkConfig};

#[derive(Args)]
pub struct IndexUpdateArgs {
    #[arg(default_value = ".")]
    root: PathBuf,
}

pub fn run(args: IndexUpdateArgs) -> Result<()> {
    let root_abs = canonicalize_clean(&args.root);
    let conn = match open_index_if_exists(&root_abs)? {
        Some(c) => c,
        None => anyhow::bail!("No index found. Run `ore index-build` first."),
    };
    println!("{} {}", "Updating index:".cyan().bold(), root_abs.display().to_string().yellow());

    // Scan disk to find new files
    let cfg = WalkConfig {
        root: root_abs.clone(),
        extensions: vec!["ts".into(),"tsx".into(),"js".into(),"jsx".into(),"rs".into(),"py".into()],
        skip_backups: true,
        ..Default::default()
    };
    let disk_files = collect_files(&cfg)?;
    let stale = stale_files(&conn)?;
    let known: std::collections::HashSet<String> = all_files(&conn)?
        .into_iter().map(|f| f.path).collect();

    let mut new_files: Vec<PathBuf> = Vec::new();
    for f in &disk_files {
        let s = f.to_string_lossy().to_string();
        if !known.contains(&s) { new_files.push(f.clone()); }
    }

    println!("  {} {} new, {} changed",
        "→".dimmed(),
        new_files.len().to_string().green(),
        stale.len().to_string().yellow());

    conn.execute_batch("BEGIN;")?;
    let mut file_ids: Vec<(PathBuf, i64)> = Vec::new();

    // Process new + stale
    let mut to_process: Vec<PathBuf> = new_files.clone();
    for row in &stale { to_process.push(PathBuf::from(&row.path)); }

    for f in &to_process {
        let meta = match std::fs::metadata(f) { Ok(m) => m, Err(_) => continue };
        let size = meta.len() as i64;
        let mtime = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64).unwrap_or(0);
        let hash = match hash_file(f) { Ok(h) => h, Err(_) => continue };
        let lang = language_of(f);
        let path_str = f.to_string_lossy().to_string();
        let (file_id, _) = upsert_file(&conn, &path_str, size, mtime, &hash, Some(lang))?;
        file_ids.push((f.clone(), file_id));

        if let Ok(content) = read_file_smart(f) {
            for s in extract_symbols(&content, f) {
                let kind_str = format!("{:?}", s.kind).to_lowercase();
                insert_symbol(&conn, file_id, &s.name, &kind_str, s.line as i64, s.column as i64, s.exported)?;
            }
        }
    }
    // Second pass: rebuild imports for touched files. Seed the id map with
    // everything already indexed (normalized) plus the touched files, so
    // cross-file resolutions hit even when the target wasn't touched.
    let mut path_to_id: HashMap<PathBuf, i64> = HashMap::new();
    for row in all_files(&conn)? {
        path_to_id.insert(canonicalize_clean(Path::new(&row.path)), row.id);
    }
    for (path, id) in &file_ids {
        path_to_id.insert(canonicalize_clean(path), *id);
    }
    for (path, file_id) in &file_ids {
        if let Ok(content) = read_file_smart(path) {
            for imp in extract_imports(&content, path) {
                let resolved_id = resolve_ts_import(path, &imp.source)
                    .and_then(|resolved| {
                        let c = canonicalize_clean(&resolved);
                        path_to_id.get(&c).copied()
                            .or_else(|| file_id_for(&conn, &c.to_string_lossy()))
                    });
                let named = imp.named.join(",");
                let _ = insert_import(&conn, *file_id, &imp.source, resolved_id, &named, imp.line as i64);
            }
        }
    }
    set_meta(&conn, "updated_at", &chrono::Local::now().timestamp().to_string())?;
    conn.execute_batch("COMMIT;")?;

    let fc = file_count(&conn)?;
    println!("{} total files in index: {}", "Done.".green().bold(), fc.to_string().yellow());
    Ok(())
}
