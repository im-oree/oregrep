use anyhow::Result;
use clap::Args;
use colored::*;
use std::time::{Duration, Instant};

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebWaitArgs {
    url: String,

    /// Wait for CSS selector to appear
    #[arg(long)]
    selector: Option<String>,

    /// Wait for text to appear anywhere on the page
    #[arg(long)]
    text: Option<String>,

    /// Wait for URL to contain this substring (e.g. after a login redirect)
    #[arg(long)]
    url_contains: Option<String>,

    /// Timeout in seconds
    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    /// Poll interval (ms)
    #[arg(short = 'i', long, default_value = "500")]
    interval: u64,
}

pub fn run(args: WebWaitArgs) -> Result<()> {
    if args.selector.is_none() && args.text.is_none() && args.url_contains.is_none() {
        anyhow::bail!("Provide one of --selector / --text / --url-contains");
    }
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let deadline = Instant::now() + Duration::from_secs(args.timeout);
    let mut attempts = 0usize;

    loop {
        attempts += 1;
        if let Some(sel) = &args.selector {
            if tab.find_element(sel).is_ok() {
                println!("{} selector {} appeared ({} tries)", "OK".green().bold(), sel.yellow(), attempts.to_string().dimmed());
                return Ok(());
            }
        }
        if let Some(text) = &args.text {
            let html = tab.get_content().unwrap_or_default();
            if html.contains(text) {
                println!("{} text '{}' present ({} tries)", "OK".green().bold(), text.yellow(), attempts.to_string().dimmed());
                return Ok(());
            }
        }
        if let Some(sub) = &args.url_contains {
            let cur = tab.get_url();
            if cur.contains(sub) {
                println!("{} URL contains '{}' → {} ({} tries)", "OK".green().bold(), sub.yellow(), cur.cyan(), attempts.to_string().dimmed());
                return Ok(());
            }
        }
        if Instant::now() > deadline {
            anyhow::bail!("Timed out after {}s ({} tries)", args.timeout, attempts);
        }
        std::thread::sleep(Duration::from_millis(args.interval));
    }
}
