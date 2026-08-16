use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebCookiesArgs {
    url: String,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: WebCookiesArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let cookies = tab.get_cookies()?;

    if args.json {
        let arr: Vec<_> = cookies.iter().map(|c| serde_json::json!({
            "name": c.name,
            "value": c.value,
            "domain": c.domain,
            "path": c.path,
            "http_only": c.http_only,
            "secure": c.secure,
            "expires": c.expires,
        })).collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
    } else {
        for c in &cookies {
            println!("  {} {}={}", c.domain.dimmed(), c.name.cyan(), c.value.yellow());
        }
        eprintln!("\n{} {} cookies", "Total:".bold(), cookies.len().to_string().yellow());
    }
    Ok(())
}
