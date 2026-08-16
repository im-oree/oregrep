use anyhow::Result;
use clap::Args;
use colored::*;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct WatchMultiArgs {
    /// Watch specs: repeat "-w path=command" (e.g. -w "src=cargo check" -w "tests=npm test")
    #[arg(short = 'w', long = "watch", required = true, num_args = 1..)]
    watches: Vec<String>,

    #[arg(short = 'd', long, default_value = "300")]
    debounce: u64,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 's', long)]
    stream: bool,

    #[arg(long)]
    initial: bool,

    #[arg(short = 'n', long)]
    no_recursive: bool,
}

struct WatchEntry {
    path: PathBuf,
    command: String,
}

pub fn run(args: WatchMultiArgs) -> Result<()> {
    let mut entries: Vec<WatchEntry> = Vec::new();
    for spec in &args.watches {
        let (p, c) = spec.split_once('=')
            .ok_or_else(|| anyhow::anyhow!("Bad --watch (expected path=command): {}", spec))?;
        let path = PathBuf::from(p.trim());
        if !path.exists() { anyhow::bail!("Path not found: {}", path.display()); }
        entries.push(WatchEntry { path, command: c.trim().to_string() });
    }
    let ext_filter: Option<Vec<String>> = args.ext.as_ref().map(|s| {
        s.split(',').map(|e| e.trim().trim_start_matches('.').to_lowercase()).collect()
    });

    println!("{} {} watchers", "Watching:".cyan().bold(), entries.len().to_string().yellow());
    for e in &entries {
        println!("  {} {} {} {}", "→".dimmed(), e.path.display().to_string().yellow(), "runs".dimmed(), e.command.magenta());
    }
    println!("{}", "Press Ctrl+C to stop.".dimmed());

    if args.initial {
        for e in &entries {
            exec_labeled(&e.command, args.stream, &e.path.display().to_string())?;
        }
    }

    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;
    let mode = if args.no_recursive { RecursiveMode::NonRecursive } else { RecursiveMode::Recursive };
    for e in &entries {
        watcher.watch(&e.path, mode)?;
    }

    let debounce = Duration::from_millis(args.debounce);
    let mut last_run: std::collections::HashMap<usize, Instant> =
        (0..entries.len()).map(|i| (i, Instant::now() - Duration::from_secs(60))).collect();

    for res in rx {
        let event = match res { Ok(e) => e, Err(_) => continue };
        if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) { continue; }

        // Ext filter
        if let Some(filters) = &ext_filter {
            let ok = event.paths.iter().any(|p| {
                p.extension().and_then(|e| e.to_str()).map(|e| filters.iter().any(|f| f == &e.to_lowercase())).unwrap_or(false)
            });
            if !ok { continue; }
        }
        // Skip our own backups
        if event.paths.iter().any(|p| p.to_string_lossy().contains(".bak") || p.to_string_lossy().contains(".oretmp")) {
            continue;
        }

        // Which watcher does this belong to?
        for (i, e) in entries.iter().enumerate() {
            let canonical_watch = std::fs::canonicalize(&e.path).unwrap_or_else(|_| e.path.clone());
            let matches = event.paths.iter().any(|p| {
                let cp = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
                cp.starts_with(&canonical_watch)
            });
            if !matches { continue; }
            let last = last_run.get(&i).copied().unwrap_or(Instant::now() - Duration::from_secs(60));
            if last.elapsed() < debounce { continue; }
            last_run.insert(i, Instant::now());
            let path_hit = event.paths.first().map(|p| p.display().to_string()).unwrap_or_default();
            println!("\n{} {} triggered by {}", "△".magenta(), e.path.display().to_string().yellow(), path_hit.dimmed());
            exec_labeled(&e.command, args.stream, &e.path.display().to_string())?;
        }
    }
    Ok(())
}

fn exec_labeled(cmd: &str, stream: bool, label: &str) -> Result<()> {
    let r = run_cmd(cmd, stream, false)?;
    if !stream {
        if !r.stdout.is_empty() { print!("{}", r.stdout); }
        if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
    }
    let tag = if r.success() { "OK".green().bold().to_string() } else { format!("EXIT {}", r.exit_code).red().bold().to_string() };
    println!("[{}] {} ({}ms)", label.dimmed(), tag, r.duration_ms.to_string().dimmed());
    Ok(())
}
