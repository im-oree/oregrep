use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebCheckArgs {
    /// URLs (inline)
    urls: Vec<String>,

    /// URL list file
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    /// Only show failures
    #[arg(short = 'F', long)]
    failures_only: bool,

    #[arg(short = 't', long, default_value = "15")]
    timeout: u64,
}

pub fn run(args: WebCheckArgs) -> Result<()> {
    let mut urls: Vec<String> = args.urls.clone();
    if let Some(f) = &args.file {
        let content = read_file_smart(f)?;
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') { urls.push(l.to_string()); }
        }
    }
    if urls.is_empty() { anyhow::bail!("Provide URLs or --file"); }

    let session = WebSession::launch(true, None)?;
    println!("{} {} URLs (headless render check)", "Check:".cyan().bold(), urls.len().to_string().yellow());
    let mut ok = 0usize;
    let mut fail = 0usize;
    for u in &urls {
        let start = std::time::Instant::now();
        match session.open(u, None, args.timeout) {
            Ok(tab) => {
                let ms = start.elapsed().as_millis();
                let title = tab.get_title().unwrap_or_default();
                let final_url = tab.get_url();
                let redirected = final_url != *u;
                let _ = tab.close(false);
                ok += 1;
                if !args.failures_only {
                    let redir = if redirected { format!(" → {}", final_url).dimmed().to_string() } else { String::new() };
                    println!("  {} {} {}ms  {}{}",
                        "OK".green().bold(),
                        u.cyan(),
                        ms.to_string().dimmed(),
                        title.dimmed(),
                        redir);
                }
            }
            Err(e) => {
                fail += 1;
                println!("  {} {}  {}", "FAIL".red().bold(), u.cyan(), e.to_string().dimmed());
            }
        }
    }
    println!("\n{} {} ok, {} failed", "Summary:".bold(), ok.to_string().green(), fail.to_string().red());
    if fail > 0 { std::process::exit(1); }
    Ok(())
}
