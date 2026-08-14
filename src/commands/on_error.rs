use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct OnErrorArgs {
    /// First command
    command: String,

    /// Command to run if first fails
    #[arg(long)]
    then: String,

    /// Stream output
    #[arg(short = 's', long)]
    stream: bool,

    /// Silent per-step
    #[arg(short = 'q', long)]
    silent: bool,
}

pub fn run(args: OnErrorArgs) -> Result<()> {
    let r = run_cmd(&args.command, args.stream, args.silent)?;
    if !args.stream && !args.silent {
        if !r.stdout.is_empty() { print!("{}", r.stdout); }
        if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
    }
    if r.success() {
        if !args.silent {
            println!("{} primary ok, no fallback triggered", "OK".green().bold());
        }
        return Ok(());
    }
    if !args.silent {
        println!("{} primary failed (exit {}), running fallback", "!".yellow(), r.exit_code);
    }
    let r2 = run_cmd(&args.then, args.stream, args.silent)?;
    if !args.stream && !args.silent {
        if !r2.stdout.is_empty() { print!("{}", r2.stdout); }
        if !r2.stderr.is_empty() { eprint!("{}", r2.stderr); }
    }
    if !r2.success() {
        std::process::exit(r2.exit_code);
    }
    Ok(())
}
