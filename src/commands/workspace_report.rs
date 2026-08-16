use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct WorkspaceReportArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'o', long, default_value = "workspace-report.md")]
    output: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
}

pub fn run(args: WorkspaceReportArgs) -> Result<()> {
    let ext_arg = args.ext.as_ref().map(|e| e.clone()).unwrap_or_default();
    let path_s = args.path.display().to_string();
    let mut md = String::new();
    md.push_str(&format!("# Workspace Report — {}\n\n", path_s));
    md.push_str(&format!("_Generated: {}_\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    add_section(&mut md, "Health", &["health", &path_s, "-e", &ext_arg])?;
    add_section(&mut md, "Stats", &["stats", &path_s, "-e", &ext_arg])?;
    add_section(&mut md, "Structure (top-level)", &["tree", &path_s, "-d", "2"])?;
    add_section(&mut md, "Recent changes", &["git-log", "-n", "10"])?;
    let _ = add_section(&mut md, "Contributors", &["git-who", "Cargo.toml"]);
    let _ = add_section(&mut md, "Most-imported files", &["analyze-imports", &path_s, "-s", "fanin", "-n", "10", "-e", &ext_arg]);
    let _ = add_section(&mut md, "Most complex functions", &["analyze-complexity", &path_s, "-e", &ext_arg, "-n", "15"]);
    let _ = add_section(&mut md, "Dead exports (sample)", &["analyze-dead-exports", &path_s, "-e", &ext_arg, "-n", "20"]);
    let _ = add_section(&mut md, "TODOs", &["find", "TODO|FIXME|HACK", &path_s, "-c"]);

    std::fs::write(&args.output, &md)?;
    println!("{} {}  ({} bytes)", "Wrote:".green().bold(), args.output.display().to_string().cyan(), md.len().to_string().yellow());
    Ok(())
}

/// Run one of our own subcommands as a child process (no shell, no PATH
/// dependency) and embed its stdout in a fenced section.
fn add_section(md: &mut String, title: &str, argv: &[&str]) -> Result<()> {
    md.push_str(&format!("## {}\n\n", title));
    md.push_str("```\n");
    let mut cmd = std::process::Command::new(std::env::current_exe()?);
    cmd.args(argv);
    if let Ok(output) = cmd.output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        md.push_str(&stdout);
        if !stdout.ends_with('\n') { md.push('\n'); }
    }
    md.push_str("```\n\n");
    Ok(())
}
