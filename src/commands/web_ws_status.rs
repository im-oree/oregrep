use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebWsStatusArgs {
    url: String,
    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: WebWsStatusArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let start = std::time::Instant::now();
    let tab = session.open(&args.url, None, args.timeout)?;
    let elapsed = start.elapsed().as_millis();
    let final_url = tab.get_url();
    let title = tab.get_title().unwrap_or_default();
    // Basic status via evaluate
    let ok_val = tab.evaluate("document.readyState === 'complete' || document.readyState === 'interactive'", true).ok();
    let ready = ok_val.and_then(|r| r.value).and_then(|v| v.as_bool()).unwrap_or(false);

    if args.quiet {
        println!("{}", if ready { "ok" } else { "not-ready" });
    } else {
        let label = if ready { "READY".green().bold() } else { "NOT READY".yellow().bold() };
        println!("{} {}  ({}ms)", label, final_url.cyan(), elapsed.to_string().dimmed());
        if !title.is_empty() { println!("  {} {}", "Title:".dimmed(), title); }
    }
    if !ready { std::process::exit(1); }
    Ok(())
}
