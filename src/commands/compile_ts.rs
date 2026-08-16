use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::compile::{parse_tsc_output, save_report, CompileReport};
use crate::engine::proc::run_cmd_in;

#[derive(Args)]
pub struct CompileTsArgs {
    /// Path to project (with tsconfig.json)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Extra tsc args
    #[arg(short = 'a', long)]
    pub args: Option<String>,

    /// Stream output live
    #[arg(short = 's', long)]
    pub stream: bool,

    /// JSON output (parsed errors)
    #[arg(short = 'j', long)]
    pub json: bool,

    /// Only show errors from this file (substring match)
    #[arg(short = 'f', long)]
    pub file: Option<String>,

    /// Disable incremental mode (default: on — 3-10x faster after first run)
    #[arg(long)]
    pub no_incremental: bool,

    /// Only check files changed since last git commit + their dependents
    #[arg(long)]
    pub changed: bool,
}

pub fn run(args: CompileTsArgs) -> Result<()> {
    // Resolve path but STRIP the \\\\? Windows extended prefix — cmd.exe can't handle it.
    let cwd = std::fs::canonicalize(&args.path)
        .map(|p| strip_extended_prefix(&p))
        .unwrap_or_else(|_| args.path.clone());
    let extra = args.args.unwrap_or_default();

    // Build tsc flag set
    let mut tsc_flags: Vec<String> = vec![
        "--noEmit".to_string(),
        "--pretty".to_string(),
        "false".to_string(),
    ];

    // Incremental mode: 3-10x faster after first run. Uses .tsbuildinfo cache.
    if !args.no_incremental {
        tsc_flags.push("--incremental".to_string());
        // Cache file in .ore/ so it doesn't pollute the project
        let ore_dir = cwd.join(".ore");
        std::fs::create_dir_all(&ore_dir)?;
        tsc_flags.push("--tsBuildInfoFile".to_string());
        tsc_flags.push(format!(".ore{}tsc-buildinfo", std::path::MAIN_SEPARATOR));
    }

    // --changed: only check git-changed .ts/.tsx files (misses dependents but fast)
    if args.changed {
        match get_changed_ts_files(&cwd) {
            Ok(files) if !files.is_empty() => {
                eprintln!("{} checking {} changed file{}",
                    "→".cyan(),
                    files.len().to_string().yellow(),
                    if files.len() == 1 { "" } else { "s" }
                );
                for f in files {
                    tsc_flags.push(f);
                }
            }
            Ok(_) => {
                eprintln!("{} no changed .ts/.tsx files", "→".dimmed());
                return Ok(());
            }
            Err(e) => {
                eprintln!("{} --changed failed: {} — falling back to full check", "⚠".yellow(), e);
            }
        }
    }

    if !extra.is_empty() {
        tsc_flags.push(extra);
    }

    // Prefer node_modules-local tsc
    let local_tsc_rel = std::path::PathBuf::from("node_modules").join(".bin")
        .join(if cfg!(windows) { "tsc.cmd" } else { "tsc" });
    let local_tsc_abs = cwd.join(&local_tsc_rel);

    let cmd = if local_tsc_abs.exists() {
        format!("{} {}", local_tsc_rel.display(), tsc_flags.join(" "))
    } else {
        format!("npx tsc {}", tsc_flags.join(" "))
    };
    eprintln!("{} {} {}",
        "Running:".cyan().bold(),
        cmd.dimmed(),
        format!("(in {})", cwd.display()).dimmed()
    );
    let result = match run_cmd_in(&cmd, Some(&cwd), args.stream, false) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Failed to run TypeScript compiler:".red().bold(), e);
            eprintln!("{}", "  Ensure npm install has been run and node_modules exists.".dimmed());
            eprintln!("{}", "  Or install tsc globally: npm install -g typescript".dimmed());
            std::process::exit(1);
        }
    };

    let combined = format!("{}\n{}", result.stdout, result.stderr);
    let (errors, warnings) = parse_tsc_output(&combined);

    // If tsc failed but we couldn't parse any errors, surface the raw output
    // so the user sees WHY (npx noise, spawn issues, missing tsconfig...).
    // Streamed runs already showed it live, so only do this when not streaming.
    if !result.success() && errors.is_empty() && warnings.is_empty() && !args.stream {
        let has_stdout = !result.stdout.trim().is_empty();
        let has_stderr = !result.stderr.trim().is_empty();
        if has_stdout || has_stderr {
            eprintln!("\n{}", "── raw tsc output ──".dimmed());
            eprint!("{}", result.stdout);
            eprint!("{}", result.stderr);
        } else {
            eprintln!("{} tsc exited {} with no output — likely path/spawn issue.",
                "⚠".yellow().bold(), result.exit_code);
        }
    }

    let report = CompileReport {
        tool: "tsc".to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        exit_code: result.exit_code,
        errors: errors.clone(),
        warnings: warnings.clone(),
        raw_output: combined,
    };
    save_report(&report)?;

    if args.json {
        if let Some(ref filter) = args.file {
            let mut filtered = report.clone();
            filtered.errors.retain(|e| e.file.contains(filter));
            filtered.warnings.retain(|w| w.file.contains(filter));
            println!("{}", serde_json::to_string_pretty(&filtered)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        return Ok(());
    }

    let errors = if let Some(ref filter) = args.file {
        errors.into_iter().filter(|e| e.file.contains(filter)).collect::<Vec<_>>()
    } else {
        errors
    };
    let warnings = if let Some(ref filter) = args.file {
        warnings.into_iter().filter(|w| w.file.contains(filter)).collect::<Vec<_>>()
    } else {
        warnings
    };

    println!();
    for e in &errors {
        println!("  {} {} {}:{}:{}  {}",
            "error".red().bold(),
            e.code.yellow(),
            e.file.cyan(),
            e.line, e.column,
            e.message);
    }
    for w in &warnings {
        println!("  {} {} {}:{}:{}  {}",
            "warn".yellow().bold(),
            w.code.dimmed(),
            w.file.cyan(),
            w.line, w.column,
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

/// Get git-changed .ts/.tsx files (staged + unstaged + untracked).
fn get_changed_ts_files(cwd: &std::path::Path) -> anyhow::Result<Vec<String>> {
    use std::process::Command;
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output()?;
    if !out.status.success() {
        anyhow::bail!("git status failed");
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut files = Vec::new();
    for line in text.lines() {
        if line.len() < 4 { continue; }
        let path = line[3..].trim();
        // Handle renames: "old -> new"
        let path = if let Some(idx) = path.find(" -> ") {
            &path[idx + 4..]
        } else {
            path
        };
        if path.ends_with(".ts") || path.ends_with(".tsx") {
            files.push(path.to_string());
        }
    }
    Ok(files)
}

/// Strip Windows extended-length path prefix (\\?\) that canonicalize adds,
/// because cmd.exe and many tools can't handle it.
#[cfg(windows)]
fn strip_extended_prefix(p: &std::path::Path) -> std::path::PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        std::path::PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

#[cfg(not(windows))]
fn strip_extended_prefix(p: &std::path::Path) -> std::path::PathBuf {
    p.to_path_buf()
}
