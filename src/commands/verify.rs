use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::proc::run_cmd_in;

#[derive(Args)]
pub struct VerifyArgs {
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Detect project type auto (default: auto). Can force: ts, rust, node
    #[arg(short = 't', long, default_value = "auto")]
    kind: String,

    /// Skip tests
    #[arg(long)]
    no_test: bool,

    /// Skip lint
    #[arg(long)]
    no_lint: bool,
}

pub fn run(args: VerifyArgs) -> Result<()> {
    let kind = if args.kind == "auto" { detect_kind(&args.path) } else { args.kind.clone() };
    println!("{} {} at {}", "Verifying:".cyan().bold(), kind.yellow(), args.path.display().to_string().dimmed());

    let mut steps: Vec<(&str, String)> = Vec::new();
    let cwd = std::fs::canonicalize(&args.path).unwrap_or_else(|_| args.path.clone());

    match kind.as_str() {
        "rust" => {
            steps.push(("cargo check", "cargo check".to_string()));
            if !args.no_lint { steps.push(("cargo clippy", "cargo clippy -- -D warnings".to_string())); }
            if !args.no_test { steps.push(("cargo test", "cargo test".to_string())); }
        }
        "ts" => {
            steps.push(("tsc --noEmit", "npx tsc --noEmit".to_string()));
            if !args.no_lint { steps.push(("eslint", "npx eslint . --max-warnings=0".to_string())); }
            if !args.no_test { steps.push(("test", "npm test --silent".to_string())); }
        }
        "node" => {
            if !args.no_lint { steps.push(("eslint", "npx eslint .".to_string())); }
            if !args.no_test { steps.push(("test", "npm test --silent".to_string())); }
        }
        other => anyhow::bail!("Unknown project kind: {} (use ts, rust, node)", other),
    }

    let mut failed = 0usize;
    for (name, cmd) in &steps {
        println!("\n{} {}", "▶".cyan(), name.yellow());
        let r = run_cmd_in(cmd, Some(&cwd), false, false)?;
        if r.success() {
            println!("  {}  ({}ms)", "OK".green().bold(), r.duration_ms);
        } else {
            failed += 1;
            println!("  {}  exit {}  ({}ms)", "FAIL".red().bold(), r.exit_code, r.duration_ms);
            if !r.stdout.is_empty() { print!("{}", r.stdout); }
            if !r.stderr.is_empty() { eprint!("{}", r.stderr); }
        }
    }

    println!("\n{} {}/{} steps passed",
        "Summary:".bold(),
        (steps.len() - failed).to_string().green(),
        steps.len().to_string().yellow());
    if failed > 0 { std::process::exit(1); }
    Ok(())
}

fn detect_kind(path: &PathBuf) -> String {
    if path.join("Cargo.toml").exists() { return "rust".to_string(); }
    if path.join("tsconfig.json").exists() { return "ts".to_string(); }
    if path.join("package.json").exists() { return "node".to_string(); }
    "node".to_string()
}
