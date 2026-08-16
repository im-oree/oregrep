use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

/// Auto-detects project type (ts / rust / node) and runs the appropriate
/// compile check. Saves the user from remembering compile-ts vs compile-rust.
#[derive(Args)]
pub struct VerifyCompileArgs {
    /// Path to project (default: .)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Stream output live
    #[arg(short = 's', long)]
    stream: bool,

    /// JSON output (if the underlying tool supports it)
    #[arg(short = 'j', long)]
    json: bool,

    /// Force a specific type: ts | rust | node
    #[arg(short = 't', long)]
    force_type: Option<String>,
}

pub fn run(args: VerifyCompileArgs) -> Result<()> {
    let cwd = std::fs::canonicalize(&args.path).unwrap_or_else(|_| args.path.clone());

    let detected = if let Some(t) = args.force_type.as_deref() {
        t.to_string()
    } else {
        detect_project_type(&cwd)?
    };

    println!(
        "{} project detected: {}",
        "verify-compile".cyan().bold(),
        detected.yellow()
    );

    match detected.as_str() {
        "ts" | "typescript" => {
            let a = crate::commands::compile_ts::CompileTsArgs {
                path: cwd,
                args: None,
                stream: args.stream,
                json: args.json,
                file: None,
                no_incremental: false,
                changed: false,
            };
            crate::commands::compile_ts::run(a)
        }
        "rust" | "cargo" => {
            let a = crate::commands::compile_rust::CompileRustArgs {
                path: cwd,
                check: true,
                args: None,
                stream: args.stream,
                json: args.json,
            };
            crate::commands::compile_rust::run(a)
        }
        "node" | "npm" => {
            let a = crate::commands::compile_node::CompileNodeArgs {
                path: cwd,
                script: "build".to_string(),
                pm: "npm".to_string(),
                stream: args.stream,
            };
            crate::commands::compile_node::run(a)
        }
        other => {
            anyhow::bail!(
                "Unknown project type: {}. Use --force-type ts|rust|node.",
                other
            );
        }
    }
}

fn detect_project_type(cwd: &std::path::Path) -> Result<String> {
    if cwd.join("Cargo.toml").exists() {
        return Ok("rust".to_string());
    }
    if cwd.join("tsconfig.json").exists() {
        return Ok("ts".to_string());
    }
    if cwd.join("package.json").exists() {
        // TS check inside package.json — look for typescript dep
        let pkg_path = cwd.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            if content.contains("\"typescript\"") || content.contains("\"tsc\"") {
                return Ok("ts".to_string());
            }
        }
        return Ok("node".to_string());
    }
    anyhow::bail!(
        "No recognized project files (Cargo.toml, tsconfig.json, package.json) in {}",
        cwd.display()
    );
}
