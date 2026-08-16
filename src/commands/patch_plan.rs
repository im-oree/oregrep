use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

/// Execute a patch plan file — DSL for multi-step atomic operations.
///
/// Plan format (one directive per line, order matters):
///   patch file.ts | find: OLD | replace: NEW
///   patch-lines file.ts | 42:50 | new content here
///   patch-insert file.ts | 100 | // new comment | --after
///   verify: compile-ts . -s
///   rollback-all-on-fail
///
/// All patches backed up. If any step (patch OR verify) fails, all files restore.
#[derive(Args)]
pub struct PatchPlanArgs {
    /// Path to plan file (or - for stdin, or use --inline)
    #[arg(default_value = "")]
    source: String,

    /// Inline plan content
    #[arg(long, conflicts_with = "source")]
    inline: Option<String>,

    /// Working directory
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Backup label (default: PLAN_<timestamp>)
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Dry-run: show plan, don't execute
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug)]
enum PlanStep {
    Patch { file: String, find: String, replace: String },
    PatchLines { file: String, range: String, text: String },
    PatchInsert { file: String, line: String, text: String, before: bool },
    Verify { command: String },
    RollbackAllOnFail,
}

pub fn run(args: PatchPlanArgs) -> Result<()> {
    let cwd = args.dir.clone().unwrap_or_else(|| std::env::current_dir().unwrap());

    // The ore executable running right now — use it for sub-invocations so
    // this works even when `ore` isn't on PATH.
    let ore_exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().replace('\\', "\\\\"))
        .unwrap_or_else(|_| "ore".to_string());
    let ore_exe_quoted = format!("\"{}\"", ore_exe);

    let content = if let Some(inline) = args.inline {
        inline
    } else if args.source == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else if args.source.is_empty() {
        anyhow::bail!("Provide plan file path, --inline, or - for stdin");
    } else {
        std::fs::read_to_string(&args.source)?
    };

    let steps = parse_plan(&content)?;

    println!("{} {} step{}", "Loaded plan:".cyan().bold(),
        steps.len().to_string().yellow(),
        if steps.len() == 1 { "" } else { "s" }
    );

    // Determine rollback mode + files touched
    let mut rollback = false;
    let mut touched_files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for s in &steps {
        match s {
            PlanStep::RollbackAllOnFail => rollback = true,
            PlanStep::Patch { file, .. } |
            PlanStep::PatchLines { file, .. } |
            PlanStep::PatchInsert { file, .. } => {
                touched_files.insert(file.clone());
            }
            _ => {}
        }
    }

    println!("  Files touched: {}", touched_files.len().to_string().yellow());
    println!("  Auto-rollback: {}", if rollback { "YES".green() } else { "no".dimmed() });

    for (i, step) in steps.iter().enumerate() {
        println!("  [{}] {}", (i + 1).to_string().dimmed(), describe(step));
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN] no execution".yellow().bold());
        return Ok(());
    }

    let label = args.label.clone().unwrap_or_else(||
        format!("PLAN_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"))
    );

    // Backup phase
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new();
    if rollback {
        println!("\n{}", "Phase: Backup".cyan().bold());
        for fs in &touched_files {
            let p = PathBuf::from(fs);
            if !p.exists() { continue; }
            match crate::engine::backup::create_backup(&p, &label) {
                Ok(bp) => {
                    println!("  {} {}", "✓".green(), fs);
                    backups.push((p, bp));
                }
                Err(e) => {
                    eprintln!("  {} backup failed for {}: {}", "✗".red(), fs, e);
                    anyhow::bail!("Aborting — backup failed.");
                }
            }
        }
    }

    // Execute phase
    println!("\n{}", "Phase: Execute".cyan().bold());
    for (i, step) in steps.iter().enumerate() {
        let cmd = to_ore_command(step);
        if cmd.is_empty() { continue; } // metadata step
        println!("\n[{}] {} {}", (i + 1).to_string().dimmed(), "→".cyan(), cmd.dimmed());
        let result = crate::engine::proc::run_cmd_in(&format!("{} {}", ore_exe_quoted, cmd), Some(&cwd), true, false)?;
        if !result.success() {
            eprintln!("\n{} step {} failed (exit {})",
                "✗".red().bold(),
                (i + 1).to_string().yellow(),
                result.exit_code
            );
            if rollback && !backups.is_empty() {
                eprintln!("{}", "Rolling back...".yellow());
                for (target, backup) in &backups {
                    let _ = std::fs::copy(backup, target);
                    eprintln!("  {} restored {}", "↻".yellow(), target.display());
                }
                eprintln!("{}", "Plan aborted. Files restored.".red().bold());
            }
            std::process::exit(1);
        }
    }

    println!("\n{}", "── PLAN COMPLETE ─────────────────".green().bold());
    Ok(())
}

fn parse_plan(content: &str) -> Result<Vec<PlanStep>> {
    let mut steps = Vec::new();
    for (lineno, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        // Directives without pipes:
        if line == "rollback-all-on-fail" || line == "rollback-on-fail" {
            steps.push(PlanStep::RollbackAllOnFail);
            continue;
        }
        if let Some(cmd) = line.strip_prefix("verify:").map(str::trim) {
            steps.push(PlanStep::Verify { command: cmd.to_string() });
            continue;
        }

        // Pipe-separated: cmd file | args | args...
        let parts: Vec<&str> = line.split('|').map(str::trim).collect();
        if parts.is_empty() {
            anyhow::bail!("Line {}: empty directive", lineno + 1);
        }

        let head_parts: Vec<&str> = parts[0].splitn(2, ' ').collect();
        let cmd = head_parts[0];
        let file = head_parts.get(1).unwrap_or(&"").to_string();

        match cmd {
            "patch" => {
                if parts.len() < 3 {
                    anyhow::bail!("Line {}: patch needs 3 parts: file | find: X | replace: Y", lineno + 1);
                }
                let find = parts[1].strip_prefix("find:").unwrap_or(parts[1]).trim().to_string();
                let replace = parts[2].strip_prefix("replace:").unwrap_or(parts[2]).trim().to_string();
                steps.push(PlanStep::Patch { file, find, replace });
            }
            "patch-lines" => {
                if parts.len() < 3 {
                    anyhow::bail!("Line {}: patch-lines needs: file | range | text", lineno + 1);
                }
                steps.push(PlanStep::PatchLines {
                    file,
                    range: parts[1].to_string(),
                    text: parts[2].to_string(),
                });
            }
            "patch-insert" => {
                if parts.len() < 3 {
                    anyhow::bail!("Line {}: patch-insert needs: file | line | text [| --before]", lineno + 1);
                }
                let before = parts.get(3).map(|s| *s == "--before").unwrap_or(false);
                steps.push(PlanStep::PatchInsert {
                    file,
                    line: parts[1].to_string(),
                    text: parts[2].to_string(),
                    before,
                });
            }
            _ => anyhow::bail!("Line {}: unknown directive '{}'", lineno + 1, cmd),
        }
    }
    Ok(steps)
}

fn describe(s: &PlanStep) -> String {
    match s {
        PlanStep::Patch { file, .. } => format!("patch {}", file),
        PlanStep::PatchLines { file, range, .. } => format!("patch-lines {} {}", file, range),
        PlanStep::PatchInsert { file, line, before, .. } =>
            format!("patch-insert {} {} {}", file, line, if *before { "--before" } else { "--after" }),
        PlanStep::Verify { command } => format!("verify: {}", command),
        PlanStep::RollbackAllOnFail => "rollback-all-on-fail".to_string(),
    }
}

fn to_ore_command(s: &PlanStep) -> String {
    match s {
        PlanStep::Patch { file, find, replace } => {
            format!("patch {} -f \"{}\" -r \"{}\"", file, escape_arg(find), escape_arg(replace))
        }
        PlanStep::PatchLines { file, range, text } => {
            if text.is_empty() {
                format!("patch-lines {} {}", file, range)
            } else {
                format!("patch-lines {} {} \"{}\"", file, range, escape_arg(text))
            }
        }
        PlanStep::PatchInsert { file, line, text, before } => {
            format!("patch-insert {} {} \"{}\" {}", file, line, escape_arg(text),
                if *before { "--before" } else { "--after" })
        }
        PlanStep::Verify { command } => command.clone(),
        PlanStep::RollbackAllOnFail => String::new(),
    }
}

fn escape_arg(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
