use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::walker::{collect_files, parse_excludes, parse_extensions, WalkConfig};

#[derive(Args)]
pub struct AiPromptArgs {
    /// The task you're describing to the AI (used to select relevant files)
    task: String,

    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(short = 'e', long)]
    ext: Option<String>,

    #[arg(short = 'x', long)]
    exclude: Option<String>,

    /// Max files to include
    #[arg(short = 'n', long, default_value = "12")]
    max_files: usize,

    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// Copy to clipboard
    #[arg(long)]
    copy: bool,

    /// Include structural digest at the top
    #[arg(long, default_value = "true")]
    with_digest: bool,

    /// Compress included file content (medium condense)
    #[arg(long, default_value = "true")]
    condense: bool,
}

pub fn run(args: AiPromptArgs) -> Result<()> {
    let ext_arg = args.ext.as_deref().map(parse_extensions).unwrap_or_default();
    let exc = args.exclude.as_deref().map(parse_excludes).unwrap_or_default();
    let cfg = WalkConfig {
        root: args.path.clone(),
        extensions: ext_arg.clone(),
        excludes: exc,
        skip_backups: true,
        ..Default::default()
    };
    let all = collect_files(&cfg)?;

    // Score files against the task
    let terms: Vec<String> = args.task.split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|w| w.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string())
        .filter(|w| !w.is_empty())
        .collect();
    let mut scored: Vec<(PathBuf, i32)> = Vec::new();
    for f in &all {
        let path_s = f.to_string_lossy().to_lowercase();
        let content = match read_file_smart(f) { Ok(c) => c.to_lowercase(), Err(_) => continue };
        let mut score = 0i32;
        for t in &terms {
            if path_s.contains(t) { score += 10; }
            score += content.matches(t.as_str()).count() as i32;
        }
        if score > 0 { scored.push((f.clone(), score)); }
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(args.max_files);

    let mut prompt = String::new();
    prompt.push_str(&format!("# Task\n\n{}\n\n", args.task));
    prompt.push_str("# Codebase context\n\n");
    prompt.push_str(&format!("Root: `{}`\n\n", args.path.display()));

    if args.with_digest {
        // Structural digest from our own binary (no shell, no PATH dependency)
        let mut cmd = std::process::Command::new(std::env::current_exe()?);
        cmd.arg("digest").arg(&args.path);
        if let Some(e) = &args.ext { cmd.arg("-e").arg(e); }
        cmd.arg("--max-exports").arg("40");
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                prompt.push_str("## Structural digest\n\n");
                prompt.push_str(&String::from_utf8_lossy(&output.stdout));
                prompt.push_str("\n\n");
            }
        }
    }

    prompt.push_str(&format!("## Relevant files ({} of {} scored)\n\n", scored.len(), all.len()));
    for (path, score) in &scored {
        prompt.push_str(&format!("### `{}` _(relevance: {})_\n\n", path.display(), score));
        let content = read_file_smart(path).unwrap_or_default();
        let body = if args.condense {
            crate::commands::condense::condense(&content, crate::commands::condense::Level::Medium)
        } else { content };
        prompt.push_str("```\n");
        prompt.push_str(&body);
        if !body.ends_with('\n') { prompt.push('\n'); }
        prompt.push_str("```\n\n");
    }

    prompt.push_str("# Instructions\n\nBased on the task and the code above, propose a solution. Reference files by path. If you need more context, list which additional files you'd want to see.\n");

    let bytes = prompt.len();
    if args.copy {
        #[cfg(windows)]
        {
            use std::io::Write;
            use std::process::{Command, Stdio};
            let mut child = Command::new("clip.exe").stdin(Stdio::piped()).spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(prompt.as_bytes())?;
            }
            child.wait()?;
        }
        eprintln!("{} copied to clipboard ({} bytes)", "OK:".green(), bytes.to_string().yellow());
    }
    match args.output {
        Some(p) => {
            std::fs::write(&p, &prompt)?;
            eprintln!("{} {}  ({} bytes)", "Wrote:".green().bold(), p.display().to_string().cyan(), bytes.to_string().yellow());
        }
        None if !args.copy => print!("{}", prompt),
        _ => {}
    }
    Ok(())
}
