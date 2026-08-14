use anyhow::Result;
use clap::Args;
use colored::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct ParallelArgs {
    /// Commands to run in parallel (each argument is one command)
    commands: Vec<String>,

    /// Max concurrent jobs (default: unlimited)
    #[arg(short = 'l', long)]
    limit: Option<usize>,

    /// Stream output live (interleaved)
    #[arg(short = 's', long)]
    stream: bool,

    /// Suppress per-job output
    #[arg(short = 'q', long)]
    silent: bool,

    /// Stop all if any fails
    #[arg(long)]
    fail_fast: bool,
}

pub fn run(args: ParallelArgs) -> Result<()> {
    if args.commands.is_empty() {
        anyhow::bail!("Provide at least one command");
    }

    let results: Arc<Mutex<Vec<(usize, i32, u128)>>> = Arc::new(Mutex::new(Vec::new()));
    let stop_flag: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    // If limit is set, use a simple semaphore via a channel
    let limit = args.limit.unwrap_or(args.commands.len());
    let (perm_tx, perm_rx) = std::sync::mpsc::sync_channel(limit);
    for _ in 0..limit {
        perm_tx.send(()).ok();
    }
    let perm_rx = Arc::new(Mutex::new(perm_rx));

    let mut handles = Vec::new();
    for (i, cmd) in args.commands.iter().enumerate() {
        let cmd = cmd.clone();
        let results = Arc::clone(&results);
        let stop_flag = Arc::clone(&stop_flag);
        let perm_rx = Arc::clone(&perm_rx);
        let perm_tx = perm_tx.clone();
        let stream = args.stream;
        let silent = args.silent;
        let fail_fast = args.fail_fast;

        let h = thread::spawn(move || {
            let _permit = perm_rx.lock().unwrap().recv().ok();
            if *stop_flag.lock().unwrap() {
                perm_tx.send(()).ok();
                return;
            }
            if !silent {
                println!("{} [{}] {}", "▶".cyan(), i.to_string().yellow(), cmd.dimmed());
            }
            let r = run_cmd(&cmd, stream, silent).unwrap_or_else(|_| crate::engine::proc::RunResult {
                exit_code: -1, stdout: String::new(), stderr: String::new(), duration_ms: 0,
            });
            if !stream && !silent {
                if !r.stdout.is_empty() { print!("[{}]out: {}", i, r.stdout); }
                if !r.stderr.is_empty() { eprint!("[{}]err: {}", i, r.stderr); }
            }
            let label = if r.success() { format!("{}", "OK".green()) } else { format!("EXIT {}", r.exit_code).red().to_string() };
            if !silent {
                println!("{} [{}] {} ({}ms)", label, i.to_string().yellow(), cmd.dimmed(), r.duration_ms.to_string().dimmed());
            }
            results.lock().unwrap().push((i, r.exit_code, r.duration_ms));
            if fail_fast && !r.success() {
                *stop_flag.lock().unwrap() = true;
            }
            perm_tx.send(()).ok();
        });
        handles.push(h);
    }

    for h in handles { h.join().ok(); }

    let mut rs = results.lock().unwrap().clone();
    rs.sort_by_key(|r| r.0);
    let failed = rs.iter().filter(|(_, code, _)| *code != 0).count();
    let total = rs.len();
    println!("\n{} {} succeeded, {} failed",
        "Summary:".bold(),
        (total - failed).to_string().green(),
        failed.to_string().red()
    );
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}
