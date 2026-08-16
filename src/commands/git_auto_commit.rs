use anyhow::Result;
use clap::Args;
use colored::*;
use std::process::Command;

use crate::engine::commit_msg::{analyze_diff, compose_message, detect_convention};
use crate::engine::git::{ensure_repo, git};

#[derive(Args)]
pub struct GitAutoCommitArgs {
    /// Auto-stage all modified tracked files first (like git commit -a)
    #[arg(short = 'a', long)]
    all: bool,

    /// Only preview the message; don't commit
    #[arg(short = 'p', long)]
    preview: bool,

    /// Force conventional-commits style
    #[arg(long)]
    conventional: bool,

    /// Force simple English
    #[arg(long, conflicts_with = "conventional")]
    simple: bool,

    /// Subject line only, no body
    #[arg(short = 'S', long)]
    subject_only: bool,

    /// Open message in $EDITOR before committing
    #[arg(short = 'e', long)]
    edit: bool,

    /// Bypass confirmation
    #[arg(short = 'y', long)]
    yes: bool,

    /// Only include files matching this substring (filter which are staged & committed)
    #[arg(long)]
    only: Option<String>,

    /// Exclude files matching
    #[arg(long)]
    except: Option<String>,
}

pub fn run(args: GitAutoCommitArgs) -> Result<()> {
    ensure_repo()?;

    // Stage first if -a
    if args.all {
        if let Some(only) = &args.only {
            let changed = crate::engine::git::changed_files()?;
            let paths: Vec<String> = changed.iter()
                .filter(|(_, p)| p.contains(only))
                .filter(|(_, p)| args.except.as_ref().map(|e| !p.contains(e)).unwrap_or(true))
                .map(|(_, p)| p.clone())
                .collect();
            if paths.is_empty() { anyhow::bail!("No matching files to stage."); }
            let mut cmd = vec!["add".to_string(), "--".to_string()];
            for p in &paths { cmd.push(p.clone()); }
            let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
            git(&refs)?;
        } else if args.except.is_some() {
            // Stage everything then unstage excludes
            git(&["add", "-A"])?;
            let changed = crate::engine::git::changed_files()?;
            let except = args.except.as_ref().unwrap();
            let unstage: Vec<String> = changed.iter()
                .filter(|(_, p)| p.contains(except))
                .map(|(_, p)| p.clone())
                .collect();
            for p in &unstage {
                let _ = git(&["reset", "HEAD", "--", p]);
            }
        } else {
            git(&["add", "-A"])?;
        }
    }

    let a = analyze_diff(true)?; // Always analyze staged for commit
    if a.files.is_empty() {
        eprintln!("{}", "No staged changes to commit.".yellow());
        return Ok(());
    }
    let style = if args.conventional { "conventional".to_string() }
        else if args.simple { "simple".to_string() }
        else { detect_convention() };
    let mut msg = compose_message(&a, &style, !args.subject_only);

    println!("{}", "Generated message:".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!("{}", msg);
    println!("{}", "─".repeat(60).dimmed());

    if args.preview {
        return Ok(());
    }

    if args.edit {
        msg = open_in_editor(&msg)?;
    }

    if !args.yes {
        let ok = crate::engine::confirm::confirm("Commit with this message?", false)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    git(&["commit", "-m", &msg])?;
    println!("{}", "Committed.".green().bold());
    Ok(())
}

fn open_in_editor(text: &str) -> Result<String> {
    let temp = std::env::temp_dir().join(format!("ore-commit-{}.txt", chrono::Local::now().format("%H%M%S")));
    std::fs::write(&temp, text)?;
    let editor = std::env::var("EDITOR").ok().or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| if cfg!(windows) { "notepad".to_string() } else { "vi".to_string() });
    // Use launch_editor helper if available; fall back to direct spawn
    let status = Command::new(&editor).arg(&temp).status();
    if status.is_err() {
        // Try cmd /C on windows
        #[cfg(windows)]
        {
            Command::new("cmd").args(["/C", &editor, &temp.to_string_lossy()]).status().ok();
        }
    }
    let edited = std::fs::read_to_string(&temp)?;
    let _ = std::fs::remove_file(&temp);
    Ok(edited.trim_end().to_string())
}
