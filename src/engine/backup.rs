use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_BACKUPS_PER_FILE: usize = 3;

/// Resolve the parent directory of a file, defaulting to "." when the
/// path has no parent component (e.g. a bare filename like "notes.txt").
fn parent_dir(file: &Path) -> &Path {
    match file.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    }
}

/// Create a backup of the given file with the label appended.
/// Format: `filename.ext.bakLABEL`
/// If MAX_BACKUPS_PER_FILE already exist, the oldest is deleted first.
pub fn create_backup(file: &Path, label: &str) -> Result<PathBuf> {
    if !file.exists() {
        anyhow::bail!("Cannot backup: file does not exist: {}", file.display());
    }

    // Get parent dir + filename
    let parent = parent_dir(file);
    let fname = file
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path: {}", file.display()))?
        .to_string_lossy()
        .to_string();

    // Find existing backups for this file
    let existing = list_backups(file)?;
    if existing.len() >= MAX_BACKUPS_PER_FILE {
        // Delete oldest (by mtime)
        let mut sorted = existing.clone();
        sorted.sort_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .ok()
        });
        if let Some(oldest) = sorted.first() {
            fs::remove_file(oldest)
                .with_context(|| format!("Failed to delete old backup: {}", oldest.display()))?;
        }
    }

    let backup_name = format!("{}.bak{}", fname, label);
    let backup_path = parent.join(backup_name);

    fs::copy(file, &backup_path)
        .with_context(|| format!("Failed to copy {} to {}", file.display(), backup_path.display()))?;

    Ok(backup_path)
}

/// List all backups for a given file (looks for `<filename>.bak*` in same dir).
pub fn list_backups(file: &Path) -> Result<Vec<PathBuf>> {
    let parent = parent_dir(file);
    let fname = file
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path: {}", file.display()))?
        .to_string_lossy()
        .to_string();
    let prefix = format!("{}.bak", fname);

    if !parent.exists() {
        return Ok(vec![]);
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) {
            backups.push(entry.path());
        }
    }
    Ok(backups)
}

/// Restore a file from a backup with the given label.
pub fn restore_backup(file: &Path, label: &str) -> Result<PathBuf> {
    let parent = parent_dir(file);
    let fname = file
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid file path: {}", file.display()))?
        .to_string_lossy()
        .to_string();

    let backup_name = format!("{}.bak{}", fname, label);
    let backup_path = parent.join(&backup_name);

    if !backup_path.exists() {
        anyhow::bail!("Backup not found: {}", backup_path.display());
    }

    fs::copy(&backup_path, file)
        .with_context(|| format!("Failed to restore {} from {}", file.display(), backup_path.display()))?;

    Ok(backup_path)
}
