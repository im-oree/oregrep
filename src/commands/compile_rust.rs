use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::compile::{parse_cargo_output, save_report, CompileReport};
use crate::engine::proc::run_cmd_in;

#[derive(Args)]
pub struct CompileRustArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Use `cargo check` (fast) instead of `cargo build`
    #[arg(short = 'c', long, default_value = "true")]
    pub check: bool,

    /// Extra cargo args
    #[arg(short = 'a', long)]
    pub args: Option<String>,

    /// Stream live output
    #[arg(short = 's', long)]
    pub stream: bool,

    /// JSON parsed output
    #[arg(short = 'j', long)]
    pub json: bool,
}

pub fn run(args: CompileRustArgs) -> Result<()> {
    let cwd = std::fs::canonicalize(&args.path).unwrap_or_else(|_| args.path.clone());
    let extra = args.args.unwrap_or_default();
    let sub = if args.check { "check" } else { "build" };
    let cmd = format!("cargo {} {}", sub, extra);
    println!("{} {}", "Running:".cyan().bold(), format!("cargo {}", sub).dimmed());
    let result = match run_cmd_in(&cmd, Some(&cwd), args.stream, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Failed to run cargo:".red().bold(), e);
            eprintln!("{}", "  Ensure Rust is installed and cargo is on PATH.".dimmed());
            std::process::exit(1);
        }
    };
    let combined = format!("{}\n{}", result.stdout, result.stderr);
    let (errors, warnings) = parse_cargo_output(&combined);

    // If failed but produced no parsed errors, surface stderr so user sees WHY
    if !result.success() && errors.is_empty() {
        if !result.stderr.trim().is_empty() && !args.stream {
            eprintln!("\n{}", "── stderr ──".dimmed());
            eprint!("{}", result.stderr);
        } else if result.stdout.trim().is_empty() && result.stderr.trim().is_empty() {
            eprintln!("{} cargo exited {} with no output.",
                "⚠".yellow().bold(), result.exit_code);
        }
    }

    let report = CompileReport {
        tool: format!("cargo-{}", sub),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        exit_code: result.exit_code,
        errors: errors.clone(),
        warnings: warnings.clone(),
        raw_output: combined,
    };
    save_report(&report)?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!();
    for e in &errors {
        let loc = if e.line > 0 { format!("{}:{}:{}", e.file, e.line, e.column) } else { e.file.clone() };
        println!("  {} {} {}  {}",
            "error".red().bold(),
            e.code.yellow(),
            loc.cyan(),
            e.message);
    }
    for w in &warnings {
        let loc = if w.line > 0 { format!("{}:{}:{}", w.file, w.line, w.column) } else { w.file.clone() };
        println!("  {} {} {}  {}",
            "warn".yellow().bold(),
            w.code.dimmed(),
            loc.cyan(),
            w.message);
    }
    println!("\n{} {} errors, {} warnings (exit {})",
        "Summary:".bold(),
        errors.len().to_string().red(),
        warnings.len().to_string().yellow(),
        result.exit_code);
    if !result.success() { std::process::exit(result.exit_code); }
    Ok(())
}
