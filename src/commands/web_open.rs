use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebOpenArgs {
    url: String,

    /// Show the browser window (default: headless)
    #[arg(short = 'V', long)]
    visible: bool,

    /// Wait for this CSS selector before returning
    #[arg(short = 'w', long)]
    wait_selector: Option<String>,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    /// Keep the browser open for N seconds after loading (useful with --visible)
    #[arg(short = 'k', long, default_value = "0")]
    keep_open: u64,
}

pub fn run(args: WebOpenArgs) -> Result<()> {
    let session = WebSession::launch(!args.visible, None)?;
    println!("{} {}", "Opening:".cyan().bold(), args.url.yellow());
    let start = std::time::Instant::now();
    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;
    let elapsed = start.elapsed().as_millis();
    let title = tab.get_title().unwrap_or_default();
    let final_url = tab.get_url();
    println!("{} {}  ({}ms)", "Loaded:".green().bold(), final_url.cyan(), elapsed.to_string().dimmed());
    if !title.is_empty() { println!("{} {}", "Title:".dimmed(), title); }
    if args.keep_open > 0 {
        println!("{} keeping open for {}s...", "…".dimmed(), args.keep_open);
        std::thread::sleep(std::time::Duration::from_secs(args.keep_open));
    }
    Ok(())
}
