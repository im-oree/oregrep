#![allow(dead_code)] // Staged infrastructure — consumed by the Index/Database batch and future retrofits.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Strip the Windows extended-length `\\?\` prefix if present.
pub fn strip_extended_prefix(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p
    }
}

/// Canonicalize + strip extended prefix. Falls back to raw if canonicalize fails.
pub fn canonicalize_clean(p: &Path) -> PathBuf {
    match std::fs::canonicalize(p) {
        Ok(abs) => strip_extended_prefix(abs),
        Err(_) => p.to_path_buf(),
    }
}

/// Strict version that errors if the path doesn't exist.
pub fn canonicalize_strict(p: &Path) -> Result<PathBuf> {
    let abs = std::fs::canonicalize(p)
        .with_context(|| format!("Failed to canonicalize: {}", p.display()))?;
    Ok(strip_extended_prefix(abs))
}

/// Convert a path to display form using OS-native separators, without `\\?\`.
pub fn display_path(p: &Path) -> String {
    strip_extended_prefix(p.to_path_buf()).to_string_lossy().into_owned()
}

/// Normalize separators for cross-platform consistency (always forward slash).
pub fn normalize_separators(s: &str) -> String {
    s.replace('\\', "/")
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_home(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/").or_else(|| p.strip_prefix("~\\")) {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    if p == "~" {
        if let Some(h) = home_dir() { return h; }
    }
    PathBuf::from(p)
}

pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

/// Compute a relative path from `from_dir` to `to_path`.
/// e.g. from = /a/b/c, to = /a/b/d/e → ../d/e
pub fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let f = from_dir.components().collect::<Vec<_>>();
    let t = to_path.components().collect::<Vec<_>>();
    let mut i = 0;
    while i < f.len() && i < t.len() && f[i] == t[i] { i += 1; }
    let ups = f.len() - i;
    let mut result = PathBuf::new();
    for _ in 0..ups { result.push(".."); }
    for c in &t[i..] { result.push(c); }
    if result.as_os_str().is_empty() { result.push("."); }
    result
}

/// Short display: path relative to root if possible, else absolute.
pub fn short_path(root: &Path, p: &Path) -> String {
    let root_c = canonicalize_clean(root);
    let p_c = canonicalize_clean(p);
    match p_c.strip_prefix(&root_c) {
        Ok(rel) => rel.to_string_lossy().into_owned(),
        Err(_) => display_path(p),
    }
}

/// Check if a path is inside a directory (after canonicalization).
pub fn is_inside(child: &Path, parent: &Path) -> bool {
    let child_c = canonicalize_clean(child);
    let parent_c = canonicalize_clean(parent);
    child_c.starts_with(&parent_c)
}

/// Return the workspace root by walking up looking for .git, Cargo.toml, or package.json.
pub fn workspace_root(start: &Path) -> Option<PathBuf> {
    let mut cur: PathBuf = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cur.join(".git").exists() || cur.join("Cargo.toml").exists() || cur.join("package.json").exists() {
            return Some(canonicalize_clean(&cur));
        }
        match cur.parent() {
            Some(p) => cur = p.to_path_buf(),
            None => return None,
        }
    }
}

/// Ensure a directory exists (create all parents if needed).
pub fn ensure_dir(p: &Path) -> Result<()> {
    if !p.exists() {
        std::fs::create_dir_all(p).with_context(|| format!("Creating dir: {}", p.display()))?;
    }
    Ok(())
}
