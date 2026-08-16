use anyhow::Result;
use clap::Args;
use colored::*;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::web::{fmt_bytes, WebSession};

#[derive(Args)]
pub struct WebScreenshotManyArgs {
    /// URL list file (one URL per line, # comments allowed)
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,

    /// Or inline URLs
    urls: Vec<String>,

    #[arg(short = 'o', long, default_value = "./screenshots")]
    out_dir: PathBuf,

    #[arg(short = 'F', long)]
    full_page: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: WebScreenshotManyArgs) -> Result<()> {
    let mut urls: Vec<String> = args.urls.clone();
    if let Some(f) = &args.file {
        let content = read_file_smart(f)?;
        for line in content.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') { urls.push(l.to_string()); }
        }
    }
    if urls.is_empty() { anyhow::bail!("Provide URLs (positional) or --file"); }
    std::fs::create_dir_all(&args.out_dir)?;

    // Single browser, sequential tabs
    let session = WebSession::launch(true, Some((1280, 720)))?;
    println!("{} {} URLs → {}",
        "Batch screenshot:".cyan().bold(),
        urls.len().to_string().yellow(),
        args.out_dir.display().to_string().cyan());

    for (i, u) in urls.iter().enumerate() {
        let name = sanitize_filename(u);
        let out = args.out_dir.join(format!("{}.png", name));
        match snap(&session, u, &out, args.full_page, args.timeout) {
            Ok(bytes) => {
                println!("  {} [{}/{}] {} ({})",
                    "OK".green(),
                    (i + 1).to_string().dimmed(),
                    urls.len().to_string().dimmed(),
                    out.display().to_string().cyan(),
                    fmt_bytes(bytes).dimmed());
            }
            Err(e) => {
                println!("  {} [{}/{}] {}: {}",
                    "FAIL".red(),
                    (i + 1).to_string().dimmed(),
                    urls.len().to_string().dimmed(),
                    u.cyan(),
                    e.to_string().dimmed());
            }
        }
    }
    Ok(())
}

fn snap(session: &WebSession, url: &str, out: &PathBuf, full: bool, timeout: u64) -> Result<u64> {
    let tab = session.open(url, None, timeout)?;
    let bytes = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, full)?;
    std::fs::write(out, &bytes)?;
    let _ = tab.close(false);
    Ok(bytes.len() as u64)
}

fn sanitize_filename(s: &str) -> String {
    let cleaned: String = s.chars().map(|c| {
        if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' }
    }).collect();
    cleaned.chars().take(120).collect()
}
