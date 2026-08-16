use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::compile::{save_report, CompileReport};
use crate::engine::proc::run_cmd_in;

#[derive(Args)]
pub struct CompileNodeArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// npm script to run (default: "build")
    #[arg(short = 'r', long, default_value = "build")]
    pub script: String,

    /// Use yarn / pnpm instead of npm
    #[arg(long, default_value = "npm")]
    pub pm: String,

    #[arg(short = 's', long)]
    pub stream: bool,
}

pub fn run(args: CompileNodeArgs) -> Result<()> {
    let cwd = std::fs::canonicalize(&args.path).unwrap_or_else(|_| args.path.clone());
    let cmd = format!("{} run {}", args.pm, args.script);
    println!("{} {}", "Running:".cyan().bold(), format!("{} run {}", args.pm, args.script).dimmed());
    let result = match run_cmd_in(&cmd, Some(&cwd), args.stream, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Failed to run script:".red().bold(), e);
            eprintln!("{}", "  Check that npm/yarn/pnpm is installed and script exists in package.json.".dimmed());
            std::process::exit(1);
        }
    };
    let combined = format!("{}\n{}", result.stdout, result.stderr);

    // Surface stderr on non-stream failure
    if !result.success() && !result.stderr.trim().is_empty() && !args.stream {
        eprintln!("\n{}", "── stderr ──".dimmed());
        eprint!("{}", result.stderr);
    }

    let report = CompileReport {
        tool: format!("{}-{}", args.pm, args.script),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        exit_code: result.exit_code,
        errors: vec![],
        warnings: vec![],
        raw_output: combined.clone(),
    };
    save_report(&report)?;

    if !args.stream {
        print!("{}", combined);
    }
    let status = if result.success() { "OK".green().bold() } else { "FAIL".red().bold() };
    println!("\n{} exit {}", status, result.exit_code);
    if !result.success() { std::process::exit(result.exit_code); }
    Ok(())
}
