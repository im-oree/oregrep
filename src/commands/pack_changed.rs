use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;


#[derive(Args)]
pub struct PackChangedArgs {
    /// Git ref to compare against (default: HEAD)
    #[arg(default_value = "HEAD")]
    since: String,

    /// Output format: tag (default), md, plain
    #[arg(long, default_value = "tag")]
    format: String,

    /// Show line numbers
    #[arg(short = 'n', long)]
    numbers: bool,

    /// Working directory
    #[arg(long)]
    dir: Option<PathBuf>,

    /// Include untracked files
    #[arg(long)]
    untracked: bool,

    /// Only include files matching this extension (e.g. ts, rs)
    #[arg(short = 'e', long)]
    ext: Option<String>,
}

pub fn run(args: PackChangedArgs) -> Result<()> {
    let cwd = args
        .dir
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Get files changed since ref
    let git_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        args.since.clone(),
    ];

    let output = std::process::Command::new("git")
        .args(&git_args)
        .current_dir(&cwd)
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("git diff failed: {}", err.trim());
    }

    let diff_output = String::from_utf8_lossy(&output.stdout);
    let mut files: Vec<String> = diff_output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    // Also include staged files not yet committed
    let staged_output = std::process::Command::new("git")
        .args(["diff", "--name-only", "--cached"])
        .current_dir(&cwd)
        .output()?;

    if staged_output.status.success() {
        let staged = String::from_utf8_lossy(&staged_output.stdout);
        for line in staged.lines() {
            let l = line.trim().to_string();
            if !l.is_empty() && !files.contains(&l) {
                files.push(l);
            }
        }
    }

    // Include untracked if requested
    if args.untracked {
        let untracked_output = std::process::Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(&cwd)
            .output()?;

        if untracked_output.status.success() {
            let ut = String::from_utf8_lossy(&untracked_output.stdout);
            for line in ut.lines() {
                let l = line.trim().to_string();
                if !l.is_empty() && !files.contains(&l) {
                    files.push(l);
                }
            }
        }
    }

    // Filter by extension
    if let Some(ref ext) = args.ext {
        let ext_dot = if ext.starts_with('.') {
            ext.to_string()
        } else {
            format!(".{}", ext)
        };
        files.retain(|f| f.ends_with(&ext_dot));
    }

    files.sort();

    if files.is_empty() {
        println!("{} no files changed since {}", "Nothing:".yellow(), args.since);
        return Ok(());
    }

    let fmt = args.format.to_lowercase();
    let mut total_lines = 0usize;
    let mut packed = 0usize;

    for file_rel in &files {
        let file_path = cwd.join(file_rel);
        if !file_path.exists() {
            // File was deleted
            eprintln!(
                "{} {} (deleted, skipping)",
                "⚠".yellow(),
                file_rel.dimmed()
            );
            continue;
        }

        // Skip binary files — check raw bytes for NUL before decoding
        let raw_bytes = match std::fs::read(&file_path) {
            Ok(b) => b,
            Err(_) => {
                eprintln!(
                    "{} {} (unreadable, skipping)",
                    "⚠".yellow(),
                    file_rel.dimmed()
                );
                continue;
            }
        };

        // NUL byte in first 8KB = binary
        let check_len = raw_bytes.len().min(8192);
        if raw_bytes[..check_len].contains(&0u8) {
            eprintln!(
                "{} {} (binary, skipping)",
                "⚠".yellow(),
                file_rel.dimmed()
            );
            continue;
        }

        let content = String::from_utf8_lossy(&raw_bytes).into_owned();

        let lang = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let lines: Vec<&str> = content.lines().collect();
        total_lines += lines.len();
        packed += 1;

        match fmt.as_str() {
            "tag" => {
                println!("<file path=\"{}\">", file_rel);
                for (i, line) in lines.iter().enumerate() {
                    if args.numbers {
                        println!("{:>5} │ {}", i + 1, line);
                    } else {
                        println!("{}", line);
                    }
                }
                println!("</file>");
            }
            "md" => {
                println!("### `{}`\n", file_rel);
                println!("```{}", lang);
                for (i, line) in lines.iter().enumerate() {
                    if args.numbers {
                        println!("{:>5} │ {}", i + 1, line);
                    } else {
                        println!("{}", line);
                    }
                }
                println!("```\n");
            }
            "plain" => {
                println!("=== {} ===", file_rel);
                for (i, line) in lines.iter().enumerate() {
                    if args.numbers {
                        println!("{:>5} │ {}", i + 1, line);
                    } else {
                        println!("{}", line);
                    }
                }
                println!();
            }
            _ => anyhow::bail!("Unknown format: {}", args.format),
        }
    }

    eprintln!(
        "{} {} file{} ({} lines) changed since {}",
        "Packed:".dimmed(),
        packed.to_string().yellow(),
        if packed == 1 { "" } else { "s" },
        total_lines.to_string().yellow(),
        args.since.cyan()
    );

    Ok(())
}
