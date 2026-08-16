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

    /// One or more rollback commands to run if any step fails.
    /// All rollback commands run in order regardless of their own exit codes.
    /// Can be specified multiple times: --rollback-on-fail "cmd1" --rollback-on-fail "cmd2"
    #[arg(long = "rollback-on-fail", value_name = "CMD")]
    rollback_on_fail: Vec<String>,

    /// Stream each command's output live (default: buffer and print after)
    #[arg(short = 's', long)]
    stream: bool,

    /// Silent (no per-step logs, only errors)
    #[arg(short = 'q', long)]
    silent: bool,
}

pub fn run(args: SequenceArgs) -> Result<()> {
    if args.commands.is_empty() {
        anyhow::bail!("Provide at least one command");
    }

    let total = args.commands.len();
    let mut failed_step: Option<(usize, i32)> = None; // (step index, exit code)
    let mut succeeded = 0usize;
    let mut failed = 0usize;

    'steps: for (i, cmd) in args.commands.iter().enumerate() {
        if !args.silent {
            println!(
                "{} [{}/{}] {}",
                "▶".cyan(),
                (i + 1).to_string().yellow(),
                total.to_string().yellow(),
                cmd.dimmed()
            );
        }

        let r = match run_cmd(cmd, args.stream, args.silent) {
            Ok(r) => r,
            Err(e) => {
                failed += 1;
                if !args.silent {
                    eprintln!("{} failed to spawn: {}", "FAIL".red().bold(), e);
                }
                if !args.continue_on_error {
                    failed_step = Some((i + 1, -1));
                    break 'steps;
                }
                continue;
            }
        };

        if !args.stream && !args.silent {
            if !r.stdout.is_empty() {
                print!("{}", r.stdout);
            }
            if !r.stderr.is_empty() {
                eprint!("{}", r.stderr);
            }
        }

        if r.success() {
            succeeded += 1;
            if !args.silent {
                println!(
                    "{} ({}ms)",
                    "OK".green(),
                    r.duration_ms.to_string().dimmed()
                );
            }
        } else {
            failed += 1;
            if !args.silent {
                eprintln!(
                    "{} exit {} ({}ms)",
                    "FAIL".red().bold(),
                    r.exit_code,
                    r.duration_ms.to_string().dimmed()
                );
            }

            if !args.continue_on_error {
                failed_step = Some((i + 1, r.exit_code));
                break 'steps;
            }
        }
    }

    // Run rollbacks if any step failed and rollbacks are configured
    if failed_step.is_some() && !args.rollback_on_fail.is_empty() {
        let (step, code) = failed_step.unwrap();
        eprintln!(
            "\n{} step {} (exit {}). Running {} rollback command{}...",
            "FAILED:".red().bold(),
            step.to_string().yellow(),
            code.to_string().red(),
            args.rollback_on_fail.len().to_string().yellow(),
            if args.rollback_on_fail.len() == 1 { "" } else { "s" }
        );

        for (i, rollback_cmd) in args.rollback_on_fail.iter().enumerate() {
            eprintln!(
                "{} [{}/{}] {}",
                "↩".yellow(),
                (i + 1).to_string().yellow(),
                args.rollback_on_fail.len().to_string().yellow(),
                rollback_cmd.dimmed()
            );

            match run_cmd(rollback_cmd, args.stream, false) {
                Ok(r) => {
                    if !r.stdout.is_empty() {
                        print!("{}", r.stdout);
                    }
                    if !r.stderr.is_empty() {
                        eprint!("{}", r.stderr);
                    }
                    if r.success() {
                        eprintln!("{} rollback ok", "✓".green());
                    } else {
                        eprintln!(
                            "{} rollback exit {} (continuing anyway)",
                            "⚠".yellow(),
                            r.exit_code
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} rollback error: {} (continuing anyway)",
                        "⚠".yellow(),
                        e
                    );
                }
            }
        }

        eprintln!("\n{}", "Rollback complete. Sequence failed.".red().bold());
        std::process::exit(1);
    }

    // Normal summary
    println!(
        "\n{} {} succeeded, {} failed",
        "Summary:".bold(),
        succeeded.to_string().green(),
        failed.to_string().red()
    );

    if failed > 0 || failed_step.is_some() {
        std::process::exit(1);
    }

    Ok(())
}
