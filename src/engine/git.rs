use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Output};

/// Run a git command with args in the given working directory.
/// Returns stdout as String on success, error with stderr on failure.
pub fn git(args: &[&str]) -> Result<String> {
    git_in(std::env::current_dir()?.as_path(), args)
}

pub fn git_in(cwd: &Path, args: &[&str]) -> Result<String> {
    let output: Output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("Failed to run: git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.into_owned())
}

/// Check if we're inside a git repository.
pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Ensure we're in a git repo, error if not.
pub fn ensure_repo() -> Result<()> {
    if !is_git_repo() {
        anyhow::bail!("Not inside a git repository (or git not installed).");
    }
    Ok(())
}

/// Get list of changed files (staged + unstaged + untracked).
/// Returns Vec<(status, path)> where status is a 2-char code from `git status --porcelain`.
pub fn changed_files() -> Result<Vec<(String, String)>> {
    let out = git(&["status", "--porcelain"])?;
    let mut result = Vec::new();
    for line in out.lines() {
        if line.len() < 4 { continue; }
        let status = line[..2].to_string();
        let path = line[3..].to_string();
        result.push((status, path));
    }
    Ok(result)
}

/// Filter files by user-facing criteria: only/except/starts/matching/changed_in.
pub struct FileFilter {
    pub only: Option<String>,       // glob or substring
    pub except: Option<String>,
    pub starts: Option<String>,
    pub matching: Option<String>,   // content match — checked separately
    pub changed_in: Option<String>, // subdir
}

impl FileFilter {
    pub fn apply(&self, paths: Vec<String>) -> Vec<String> {
        paths.into_iter().filter(|p| self.matches(p)).collect()
    }

    fn matches(&self, path: &str) -> bool {
        if let Some(only) = &self.only {
            if !path.contains(only) { return false; }
        }
        if let Some(except) = &self.except {
            if path.contains(except) { return false; }
        }
        if let Some(starts) = &self.starts {
            let fname = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);
            if !fname.starts_with(starts) {
                return false;
            }
        }
        if let Some(dir) = &self.changed_in {
            let normalized = path.replace('\\', "/");
            let dir_norm = dir.replace('\\', "/");
            if !normalized.starts_with(&dir_norm) { return false; }
        }
        if let Some(m) = &self.matching {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            if !content.contains(m) { return false; }
        }
        true
    }
}
