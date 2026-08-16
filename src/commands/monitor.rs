use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::Duration;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct MonitorArgs {
    /// Command whose output/exit is monitored
    command: String,

    /// Interval in seconds
    #[arg(short = 'i', long, default_value = "30")]
    interval: f64,

    /// Max iterations (0 = forever)
    #[arg(short = 'n', long, default_value = "0")]
    count: usize,

    /// Alert command to run when the state changes
    #[arg(long)]
    on_change: Option<String>,

    /// Alert command to run when the command exits non-zero
    #[arg(long)]
    on_error: Option<String>,

    /// Alert when output contains this text
    #[arg(long)]
    on_contains: Option<String>,

    /// Alert when output STOPS containing this text
    #[arg(long)]
    on_missing: Option<String>,

    /// Show every poll (default: only on change or alert)
    #[arg(short = 'v', long)]
    verbose: bool,
}

pub fn run(args: MonitorArgs) -> Result<()> {
    println!("{} {}", "Monitor:".cyan().bold(), args.command.magenta());
    println!("  interval: {}s, count: {}", args.interval, if args.count == 0 { "∞".to_string() } else { args.count.to_string() });

    let mut prev_output: Option<String> = None;
    let mut prev_success: Option<bool> = None;
    let mut i = 0usize;

    loop {
        i += 1;
        if args.count > 0 && i > args.count { break; }
        let r = run_cmd(&args.command, false, true)?;
        let out = r.stdout.trim().to_string();
        let success = r.success();

        let changed = prev_output.as_ref().map(|p| p != &out).unwrap_or(true);
        let status_changed = prev_success.map(|p| p != success).unwrap_or(true);
        let alert_contains = args.on_contains.as_ref().map(|s| out.contains(s)).unwrap_or(false);
        let alert_missing = args.on_missing.as_ref().map(|s| {
            let now_missing = !out.contains(s);
            let was_present = prev_output.as_ref().map(|p| p.contains(s)).unwrap_or(false);
            now_missing && was_present
        }).unwrap_or(false);

        let should_print = args.verbose || changed || status_changed || alert_contains || alert_missing;
        if should_print {
            let ts = chrono::Local::now().format("%H:%M:%S").to_string();
            let tag = if success { "OK".green() } else { format!("EXIT {}", r.exit_code).red() };
            println!("[{}] {} {}", ts.dimmed(), tag, out.lines().next().unwrap_or("").chars().take(120).collect::<String>().dimmed());
        }

        // Alerts
        if changed && !prev_output.is_none() {
            if let Some(alert) = &args.on_change {
                println!("  {} triggering on-change", "△".magenta());
                let _ = run_cmd(alert, false, false);
            }
        }
        if !success && prev_success.unwrap_or(true) {
            if let Some(alert) = &args.on_error {
                println!("  {} triggering on-error", "△".red());
                let _ = run_cmd(alert, false, false);
            }
        }
        if alert_contains {
            println!("  {} on-contains matched", "△".yellow());
        }
        if alert_missing {
            println!("  {} on-missing matched", "△".yellow());
        }

        prev_output = Some(out);
        prev_success = Some(success);
        std::thread::sleep(Duration::from_secs_f64(args.interval.max(0.1)));
    }
    Ok(())
}
