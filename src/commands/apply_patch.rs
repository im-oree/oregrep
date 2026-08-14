use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::engine::backup::create_backup;

#[derive(Args)]
pub struct ApplyPatchArgs {
    /// .patch or .diff file
    pub patch: PathBuf,

    /// Path to apply within (default: current dir)
    #[arg(short = 'p', long)]
    pub path: Option<PathBuf>,

    /// Skip backups
    #[arg(long)]
    pub no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    pub label: Option<String>,

    /// Reverse-apply (undo the patch)
    #[arg(short = 'R', long)]
    pub reverse: bool,

    /// Dry-run: just check if it would apply cleanly
    #[arg(long)]
    pub check: bool,
}

pub fn run(args: ApplyPatchArgs) -> Result<()> {
    if !args.patch.exists() {
        anyhow::bail!("Patch file not found: {}", args.patch.display());
    }
    let workdir = args.path.clone().unwrap_or_else(|| PathBuf::from("."));
    if !workdir.exists() {
        anyhow::bail!("Work directory not found: {}", workdir.display());
    }

    // Resolve absolute patch path so cwd change doesn't break it.
    // Strip Windows' \\\\?\ extended-path prefix, which git can't open.
    let patch_abs = strip_extended_prefix(&std::fs::canonicalize(&args.patch)?);

    // Backups
    if !args.no_backup && !args.check {
        let content = crate::engine::encoding::read_file_smart(&args.patch)?;
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        for line in content.lines() {
            let candidate = if let Some(rest) = line.strip_prefix("+++ b/") {
                Some(rest.trim().to_string())
            } else if let Some(rest) = line.strip_prefix("+++ ") {
                let c = rest.trim();
                if !c.starts_with('/') && c != "/dev/null" { Some(c.to_string()) } else { None }
            } else {
                None
            };
            if let Some(rel) = candidate {
                let target = workdir.join(&rel);
                if target.exists() {
                    if let Err(e) = create_backup(&target, &label) {
                        eprintln!("{} backup skipped for {}: {}", "!".yellow(), target.display(), e);
                    }
                }
            }
        }
    }

    // Direct git process (no shell nesting)
    let mut cmd = Command::new("git");
    cmd.arg("apply");
    if args.check { cmd.arg("--check"); }
    if args.reverse { cmd.arg("-R"); }
    cmd.arg(&patch_abs);
    cmd.current_dir(&workdir);

    let output = cmd.output()?;
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if output.status.success() {
        let label = if args.check { "CHECK OK" } else if args.reverse { "REVERSED" } else { "APPLIED" };
        println!("{} {}", label.green().bold(), args.patch.display().to_string().cyan());
    } else {
        anyhow::bail!("git apply failed (exit {})", output.status.code().unwrap_or(-1));
    }
    Ok(())
}

fn strip_extended_prefix(p: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(rest);
        }
    }
    p.to_path_buf()
}
