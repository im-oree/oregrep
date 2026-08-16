use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct BenchmarkArgs {
    /// Command to benchmark
    command: String,

    /// Number of runs
    #[arg(short = 'n', long, default_value = "10")]
    runs: usize,

    /// Warmup runs (not counted)
    #[arg(short = 'w', long, default_value = "2")]
    warmup: usize,

    /// Show per-run timings
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Fail if any run errored
    #[arg(long)]
    strict: bool,
}

pub fn run(args: BenchmarkArgs) -> Result<()> {
    println!("{} {} × {} (warmup: {})", "Benchmarking:".cyan().bold(), args.command.magenta(), args.runs.to_string().yellow(), args.warmup.to_string().dimmed());

    for i in 0..args.warmup {
        if args.verbose { println!("  {} warmup {}/{}", "▶".dimmed(), (i + 1).to_string().dimmed(), args.warmup.to_string().dimmed()); }
        let _ = run_cmd(&args.command, false, true);
    }

    let mut times: Vec<u128> = Vec::with_capacity(args.runs);
    let mut errors = 0usize;
    for i in 0..args.runs {
        let r = run_cmd(&args.command, false, true)?;
        if !r.success() {
            errors += 1;
            if args.strict { anyhow::bail!("Run {} failed (exit {})", i + 1, r.exit_code); }
        }
        times.push(r.duration_ms);
        if args.verbose {
            let tag = if r.success() { "OK".green() } else { format!("EXIT {}", r.exit_code).red() };
            println!("  [{}] {} {}ms", (i + 1).to_string().dimmed(), tag, r.duration_ms.to_string().yellow());
        }
    }

    times.sort();
    let n = times.len();
    let min = *times.first().unwrap();
    let max = *times.last().unwrap();
    let mean = times.iter().sum::<u128>() / n as u128;
    let p50 = times[n / 2];
    let p95 = times[(n * 95 / 100).min(n - 1)];
    let p99 = times[(n * 99 / 100).min(n - 1)];
    let stddev = {
        let m = mean as f64;
        let variance = times.iter().map(|&t| { let d = t as f64 - m; d * d }).sum::<f64>() / n as f64;
        variance.sqrt() as u128
    };

    println!("\n{}", "Results:".bold());
    println!("  Runs:   {}   Errors: {}", args.runs.to_string().yellow(), errors.to_string().red());
    println!("  min:    {} ms", min.to_string().green());
    println!("  mean:   {} ms  (±{} stddev)", mean.to_string().green(), stddev.to_string().dimmed());
    println!("  p50:    {} ms", p50.to_string().green());
    println!("  p95:    {} ms", p95.to_string().yellow());
    println!("  p99:    {} ms", p99.to_string().yellow());
    println!("  max:    {} ms", max.to_string().red());
    Ok(())
}
