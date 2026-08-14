use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct SequenceArgs {
    /// Commands to run sequentially
    commands: Vec<String>,

    /// Continue on failure (default: stop on first failure)
    #[arg(short = 'c', long)]
    continue_on_error: bool,

    /// Stream each command's output
    #[arg(short = 's', long)]
    stream: bool,

    /// Silent (no per-step logs)
    #[arg(short = 'q', long)]
    silent: bool,
}

pub fn run(args: SequenceArgs) -> Result<()> {
    if args.commands.is_empty() {
        anyhow::bail!("Provide at least one command");
    }
    let mut failed = 0;
    let mut succeeded = 0;
    for (i, cmd) in args.commands.iter().enumerate() {
        if !args.silent {
            println!("{} [{}/{}] {}", "▶".cyan(), (i + 1).to_string().yellow(), args.commands.len().to_string().yellow(), cmd.dimmed());
        }
        let r = run_cmd(cmd, args.stream, args.silent)?;
        if !args.stream && !args.silent {
            if !r.stdout.is_empty() { print!("{}", r.stdout); }
            if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
        }
        if r.success() {
            succeeded += 1;
            if !args.silent {
                println!("{} ({}ms)", "OK".green(), r.duration_ms.to_string().dimmed());
            }
        } else {
            failed += 1;
            if !args.silent {
                eprintln!("{} exit {} ({}ms)", "FAIL".red().bold(), r.exit_code, r.duration_ms.to_string().dimmed());
            }
            if !args.continue_on_error {
                anyhow::bail!("Sequence stopped at step {} (exit {})", i + 1, r.exit_code);
            }
        }
    }
    println!("\n{} {} succeeded, {} failed",
        "Summary:".bold(),
        succeeded.to_string().green(),
        failed.to_string().red()
    );
    if failed > 0 { std::process::exit(1); }
    Ok(())
}
