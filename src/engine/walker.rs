use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::engine::encoding::is_binary;

/// Configuration for walking a project/directory to collect target files.
pub struct WalkConfig {
    /// Root path to walk
    pub root: PathBuf,
    /// Extensions to include (empty = all)
    pub extensions: Vec<String>,
    /// Include hidden files
    pub hidden: bool,
    /// Respect .gitignore
    pub respect_gitignore: bool,
    /// Include binary files
    pub include_binary: bool,
    /// Skip files/dirs whose name contains any of these substrings
    pub excludes: Vec<String>,
    /// Skip .bak* files
    pub skip_backups: bool,
}

impl Default for WalkConfig {
    fn default() -> Self {
        WalkConfig {
            root: PathBuf::from("."),
            extensions: vec![],
            hidden: false,
            respect_gitignore: true,
            include_binary: false,
            excludes: vec![],
            skip_backups: true,
        }
    }
}

/// Collect all files matching the walk config.
pub fn collect_files(cfg: &WalkConfig) -> Result<Vec<PathBuf>> {
    if !cfg.root.exists() {
        anyhow::bail!("Path not found: {}", cfg.root.display());
    }

    let mut files = Vec::new();
    let walker = WalkBuilder::new(&cfg.root)
        .hidden(!cfg.hidden)
        .git_ignore(cfg.respect_gitignore)
        .git_global(cfg.respect_gitignore)
        .git_exclude(cfg.respect_gitignore)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Skip backup files
        if cfg.skip_backups {
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if fname.contains(".bak") {
                continue;
            }
        }

        // Exclude substrings
        let path_str = path.to_string_lossy().to_lowercase();
        if cfg.excludes.iter().any(|e| path_str.contains(&e.to_lowercase())) {
            continue;
        }

        // Extension filter
        if !cfg.extensions.is_empty() {
            let matches_ext = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| cfg.extensions.iter().any(|f| f.to_lowercase() == e.to_lowercase()))
                .unwrap_or(false);
            if !matches_ext {
                continue;
            }
        }

        // Binary skip
        if !cfg.include_binary {
            match is_binary(path) {
                Ok(true) => continue,
                Ok(false) => {}
                Err(_) => continue,
            }
        }

        files.push(path.to_path_buf());
    }

    Ok(files)
}

/// Parse a comma-separated extension list ("ts,tsx,rs") into Vec.
pub fn parse_extensions(s: &str) -> Vec<String> {
    s.split(',')
        .map(|e| e.trim().trim_start_matches('.').to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

/// Parse a comma-separated exclude list into Vec.
pub fn parse_excludes(s: &str) -> Vec<String> {
    s.split(',')
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .collect()
}

#[allow(dead_code)]
pub fn root_path() -> &'static Path {
    Path::new(".")
}
