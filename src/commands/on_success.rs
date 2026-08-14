use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::proc::run_cmd;

#[derive(Args)]
pub struct OnSuccessArgs {
    command: String,

    /// Command to run if first succeeds
    #[arg(long)]
    then: String,

    #[arg(short = 's', long)]
    stream: bool,
    #[arg(short = 'q', long)]
    silent: bool,
}

pub fn run(args: OnSuccessArgs) -> Result<()> {
    let r = run_cmd(&args.command, args.stream, args.silent)?;
    if !args.stream && !args.silent {
        if !r.stdout.is_empty() { print!("{}", r.stdout); }
        if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
    }
    if !r.success() {
        if !args.silent {
            eprintln!("{} primary failed (exit {}), skipping follow-up", "!".yellow(), r.exit_code);
        }
        std::process::exit(r.exit_code);
    }
    if !args.silent {
        println!("{} primary ok, running follow-up", "OK".green());
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
