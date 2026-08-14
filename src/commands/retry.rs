use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::Duration;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct RetryArgs {
    /// Command to run
    command: String,

    /// Max attempts (default 5)
    #[arg(short = 'n', long, default_value = "5")]
    max: usize,

    /// Wait between attempts (seconds)
    #[arg(short = 'i', long, default_value = "1.0")]
    interval: f64,

    /// Exponential backoff multiplier (default 1.0 = constant)
    #[arg(short = 'b', long, default_value = "1.0")]
    backoff: f64,

    /// Suppress per-attempt logs
    #[arg(short = 'q', long)]
    silent: bool,

    /// Stream command output
    #[arg(short = 's', long)]
    stream: bool,
}

pub fn run(args: RetryArgs) -> Result<()> {
    let mut wait = args.interval;
    for attempt in 1..=args.max {
        if !args.silent {
            println!("{} attempt {}/{}", "▶".cyan(), attempt.to_string().yellow(), args.max.to_string().yellow());
        }
        let r = run_cmd(&args.command, args.stream, args.silent)?;
        if r.success() {
            if !args.silent {
                println!("{} after {} attempts", "OK".green().bold(), attempt.to_string().yellow());
            }
            return Ok(());
        }
        if attempt < args.max {
            if !args.silent {
                eprintln!("{} exit {} — waiting {:.1}s", "…".yellow(), r.exit_code, wait);
            }
            std::thread::sleep(Duration::from_secs_f64(wait));
            wait *= args.backoff;
        }
    }
    anyhow::bail!("Command failed after {} attempts", args.max);
}
