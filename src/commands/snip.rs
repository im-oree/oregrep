use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::io::Read;
use std::path::PathBuf;

use crate::engine::storage::{snippet_path, snippets_dir};

#[derive(Args)]
pub struct SnipArgs {
    #[command(subcommand)]
    pub action: SnipAction,
}

#[derive(Subcommand)]
pub enum SnipAction {
    /// Save a snippet from stdin or --file
    Save {
        name: String,
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Load a snippet to stdout
    Load { name: String },
    /// List all snippets
    List,
    /// Delete a snippet
    Rm {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show snippet path
    Path { name: String },
    /// Search snippet contents for text
    Find { query: String },
    /// Export all snippets to a directory
    Export {
        #[arg(short = 'o', long, default_value = "./snippets-export")]
        dir: PathBuf,
    },
    /// Import snippets from a directory
    Import {
        dir: PathBuf,
        #[arg(long)]
        force: bool,
    },
    /// Copy a snippet's contents to clipboard
    Copy { name: String },
}

pub fn run(args: SnipArgs) -> Result<()> {
    match args.action {
        SnipAction::Save { name, file, force } => {
            let path = snippet_path(&name)?;
            if path.exists() && !force {
                anyhow::bail!("Snippet exists: {} (use --force)", name);
            }
            let content = if let Some(f) = file {
                std::fs::read_to_string(f)?
            } else {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            };
            if content.trim().is_empty() { anyhow::bail!("Empty content"); }
            std::fs::write(&path, &content)?;
            println!("{} {} ({} bytes)", "Saved:".green().bold(), name.cyan(), content.len().to_string().yellow());
        }
        SnipAction::Load { name } => {
            let path = snippet_path(&name)?;
            if !path.exists() { anyhow::bail!("Snippet not found: {}", name); }
            print!("{}", std::fs::read_to_string(&path)?);
        }
        SnipAction::List => {
            let d = snippets_dir()?;
            let mut count = 0;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("txt") {
                    let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                    println!("  {} {} bytes", name.cyan(), size.to_string().dimmed());
                    count += 1;
                }
            }
            if count == 0 { println!("{}", "(no snippets)".dimmed()); }
            else { println!("\n{} {} snippets", "Total:".bold(), count.to_string().yellow()); }
        }
        SnipAction::Rm { name, yes } => {
            let path = snippet_path(&name)?;
            if !path.exists() { anyhow::bail!("Snippet not found: {}", name); }
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Delete snippet '{}'?", name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            std::fs::remove_file(&path)?;
            println!("{} {}", "Deleted:".green(), name.cyan());
        }
        SnipAction::Path { name } => {
            println!("{}", snippet_path(&name)?.display());
        }
        SnipAction::Find { query } => {
            let d = snippets_dir()?;
            let q_lc = query.to_lowercase();
            let mut hits = 0;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) != Some("txt") { continue; }
                let content = std::fs::read_to_string(e.path())?;
                if content.to_lowercase().contains(&q_lc) {
                    let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let count = content.to_lowercase().matches(&q_lc).count();
                    println!("  {} ({}×)", name.cyan(), count.to_string().yellow());
                    hits += 1;
                }
            }
            if hits == 0 { println!("{}", "(no matches)".dimmed()); }
        }
        SnipAction::Export { dir } => {
            std::fs::create_dir_all(&dir)?;
            let d = snippets_dir()?;
            let mut n = 0;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("txt") {
                    let name = e.path().file_name().unwrap().to_string_lossy().to_string();
                    std::fs::copy(e.path(), dir.join(&name))?;
                    n += 1;
                }
            }
            println!("{} {} snippets → {}", "Exported:".green().bold(), n.to_string().yellow(), dir.display().to_string().cyan());
        }
        SnipAction::Import { dir, force } => {
            if !dir.is_dir() { anyhow::bail!("Not a directory: {}", dir.display()); }
            let dst = snippets_dir()?;
            let mut n = 0;
            let mut skipped = 0;
            for entry in std::fs::read_dir(&dir)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("txt") {
                    let target = dst.join(e.file_name());
                    if target.exists() && !force { skipped += 1; continue; }
                    std::fs::copy(e.path(), &target)?;
                    n += 1;
                }
            }
            println!("{} {} imported, {} skipped", "Done:".green().bold(), n.to_string().green(), skipped.to_string().yellow());
        }
        SnipAction::Copy { name } => {
            let path = snippet_path(&name)?;
            if !path.exists() { anyhow::bail!("Snippet not found: {}", name); }
            let content = std::fs::read_to_string(&path)?;
            #[cfg(windows)]
            {
                use std::io::Write;
                use std::process::{Command, Stdio};
                let mut child = Command::new("clip.exe").stdin(Stdio::piped()).spawn()?;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(content.as_bytes())?;
                }
                child.wait()?;
            }
            #[cfg(not(windows))]
            {
                eprintln!("{}", "Clipboard not implemented on this platform".yellow());
            }
            eprintln!("{} {} ({} bytes copied)", "OK:".green().bold(), name.cyan(), content.len().to_string().yellow());
        }
    }
    Ok(())
}
