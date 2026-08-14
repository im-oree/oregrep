use anyhow::Result;
use clap::Args;
use colored::*;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct WatchArgs {
    /// Path to watch (file or directory)
    path: PathBuf,

    /// Command to run when a change is detected
    command: String,

    /// Non-recursive
    #[arg(short = 'n', long)]
    no_recursive: bool,

    /// Debounce (ms)
    #[arg(short = 'd', long, default_value = "300")]
    debounce: u64,

    /// Extension filter (comma-separated)
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Stream command output
    #[arg(short = 's', long)]
    stream: bool,

    /// Run once at start (before any change)
    #[arg(long)]
    initial: bool,
}

pub fn run(args: WatchArgs) -> Result<()> {
    if !args.path.exists() {
        anyhow::bail!("Path not found: {}", args.path.display());
    }
    let ext_filter: Option<Vec<String>> = args.ext.as_ref().map(|s| {
        s.split(',').map(|e| e.trim().trim_start_matches('.').to_lowercase()).collect()
    });

    println!("{} {}",
        "Watching".cyan().bold(),
        args.path.display().to_string().yellow()
    );
    println!("{} {}", "Command:".dimmed(), args.command.dimmed());
    println!("{}", "Press Ctrl+C to stop.".dimmed());

    if args.initial {
        exec_and_log(&args.command, args.stream)?;
    }

    let (tx, rx) = channel::<notify::Result<Event>>();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;
    let mode = if args.no_recursive { RecursiveMode::NonRecursive } else { RecursiveMode::Recursive };
    watcher.watch(&args.path, mode)?;

    let mut last_run = Instant::now() - Duration::from_secs(60);
    let debounce = Duration::from_millis(args.debounce);

    for res in rx {
        let event = match res { Ok(e) => e, Err(_) => continue };
        if !matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)) {
            continue;
        }
        // Ext filter
        if let Some(filters) = &ext_filter {
            let ok = event.paths.iter().any(|p| {
                p.extension().and_then(|e| e.to_str()).map(|e| filters.iter().any(|f| f == &e.to_lowercase())).unwrap_or(false)
            });
            if !ok { continue; }
        }
        // Skip our own backup files
        if event.paths.iter().any(|p| p.to_string_lossy().contains(".bak") || p.to_string_lossy().contains(".oretmp")) {
            continue;
        }
        if last_run.elapsed() < debounce { continue; }
        last_run = Instant::now();

        println!("\n{} {}",
            "△ change".magenta(),
            event.paths.first().map(|p| p.display().to_string()).unwrap_or_default().dimmed()
        );
        exec_and_log(&args.command, args.stream)?;
    }
    Ok(())
}

fn exec_and_log(cmd: &str, stream: bool) -> Result<()> {
    let r = run_cmd(cmd, stream, false)?;
    if !stream {
        if !r.stdout.is_empty() { print!("{}", r.stdout); }
        if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
    }
    let tag = if r.success() { "OK".green().bold().to_string() } else { format!("EXIT {}", r.exit_code).red().bold().to_string() };
    println!("{} ({}ms)", tag, r.duration_ms.to_string().dimmed());
    Ok(())
}
