use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::http::{apply_headers, build_client, parse_headers_from_flags, status_color};

#[derive(Args)]
pub struct HeadersArgs {
    url: String,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "10")]
    timeout: u64,

    /// Use GET instead of HEAD (some servers don't support HEAD)
    #[arg(short = 'g', long)]
    get: bool,
}

pub fn run(args: HeadersArgs) -> Result<()> {
    let client = build_client(args.timeout, true, None)?;
    let hdrs = parse_headers_from_flags(&args.headers)?;
    let req = if args.get { client.get(&args.url) } else { client.head(&args.url) };
    let req = apply_headers(req, &hdrs);
    let start = std::time::Instant::now();
    let resp = req.send()?;
    let elapsed = start.elapsed().as_millis();
    let color = status_color(resp.status().as_u16());
    println!("{} {} {}  ({}ms)",
        if args.get { "GET".dimmed() } else { "HEAD".dimmed() },
        format!("{} {}", resp.status().as_u16(), resp.status().canonical_reason().unwrap_or("")).color(color).bold(),
        resp.url().to_string().cyan(),
        elapsed.to_string().dimmed()
    );
    for (k, v) in resp.headers() {
        println!("  {}: {}", k.to_string().cyan(), v.to_str().unwrap_or(""));
    }
    Ok(())
}
