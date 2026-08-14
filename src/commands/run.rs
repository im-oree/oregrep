use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct RunArgs {
    /// Command to execute (via cmd.exe /C on Windows). Quote it.
    command: String,

    /// Stream output live (default: capture and print after)
    #[arg(short = 's', long)]
    stream: bool,

    /// Suppress all output
    #[arg(short = 'q', long)]
    silent: bool,

    /// Fail (exit non-zero) if command exits non-zero
    #[arg(long)]
    fail_on_error: bool,

    /// Print timing/exit summary
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Write stdout to this file
    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,

    /// Write stderr to this file
    #[arg(long)]
    err_output: Option<std::path::PathBuf>,
}

pub fn run(args: RunArgs) -> Result<()> {
    let result = run_cmd(&args.command, args.stream, args.silent)?;

    if !args.stream && !args.silent {
        if !result.stdout.is_empty() {
            print!("{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            eprint!("{}", result.stderr);
        }
    }

    if let Some(p) = &args.output {
        std::fs::write(p, &result.stdout)?;
    }
    if let Some(p) = &args.err_output {
        std::fs::write(p, &result.stderr)?;
    }

    if args.verbose || (args.fail_on_error && !result.success()) {
        let label = if result.success() { "OK".green().bold().to_string() } else { format!("EXIT {}", result.exit_code).red().bold().to_string() };
        eprintln!("{} {} {}ms", label, args.command.dimmed(), result.duration_ms.to_string().dimmed());
    }

    if args.fail_on_error && !result.success() {
        std::process::exit(result.exit_code);
    }

    Ok(())
}
