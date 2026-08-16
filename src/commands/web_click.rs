use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebClickArgs {
    url: String,
    selector: String,

    /// Wait after click (ms)
    #[arg(short = 'd', long, default_value = "500")]
    delay: u64,

    /// Screenshot after click (path)
    #[arg(long)]
    screenshot: Option<std::path::PathBuf>,

    /// Show browser window
    #[arg(short = 'V', long)]
    visible: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: WebClickArgs) -> Result<()> {
    let session = WebSession::launch(!args.visible, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let el = tab.wait_for_element(&args.selector)?;
    el.click()?;
    std::thread::sleep(std::time::Duration::from_millis(args.delay));
    println!("{} clicked {}", "OK".green().bold(), args.selector.yellow());
    let url_after = tab.get_url();
    let title = tab.get_title().unwrap_or_default();
    println!("  {} {}", "URL:".dimmed(), url_after);
    if !title.is_empty() { println!("  {} {}", "Title:".dimmed(), title); }
    if let Some(shot) = args.screenshot {
        let bytes = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None, None, false)?;
        if let Some(parent) = shot.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() { std::fs::create_dir_all(parent)?; }
        }
        std::fs::write(&shot, bytes)?;
        println!("  {} {}", "Screenshot:".dimmed(), shot.display());
    }
    Ok(())
}
