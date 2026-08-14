use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::http::{apply_headers, build_client, parse_headers_from_flags, read_body_bytes, status_color};

#[derive(Args)]
pub struct FetchArgs {
    url: String,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(long)]
    no_redirect: bool,

    #[arg(long)]
    proxy: Option<String>,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'i', long)]
    include_headers: bool,

    /// Skip body output (status + headers only)
    #[arg(short = 'q', long)]
    no_body: bool,

    #[arg(short = 'j', long)]
    pretty: bool,
}

pub fn run(args: FetchArgs) -> Result<()> {
    let client = build_client(args.timeout, !args.no_redirect, args.proxy.as_deref())?;
    let hdrs = parse_headers_from_flags(&args.headers)?;

    let start = std::time::Instant::now();
    let req = apply_headers(client.get(&args.url), &hdrs);
    let resp = req.send()?;
    let final_url = resp.url().to_string();
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let response_headers: Vec<(String, String)> = resp.headers().iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = read_body_bytes(resp)?;
    let elapsed = start.elapsed().as_millis();

    let color = status_color(status);
    eprintln!("{} {} {}  {}  {}",
        format!("HTTP {}", status).color(color).bold(),
        status_text.dimmed(),
        format!("({} bytes)", body.len()).dimmed(),
        final_url.cyan(),
        format!("({} ms)", elapsed).dimmed()
    );

    if args.include_headers {
        println!();
        for (k, v) in &response_headers {
            println!("{}: {}", k.cyan(), v);
        }
    }

    if let Some(path) = &args.output {
        std::fs::write(path, &body)?;
        println!("{} {} ({} bytes)", "Wrote:".green().bold(),
            path.display().to_string().cyan(),
            body.len().to_string().yellow());
        return Ok(());
    }

    if args.no_body { return Ok(()); }

    let ctype = response_headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.to_lowercase()).unwrap_or_default();
    let is_json = ctype.contains("json");
    let text = String::from_utf8_lossy(&body);

    println!();
    if args.pretty && is_json {
        match serde_json::from_str::<serde_json::Value>(&text) {
            Ok(v) => println!("{}", serde_json::to_string_pretty(&v)?),
            Err(_) => print!("{}", text),
        }
    } else {
        print!("{}", text);
    }
    if !text.ends_with('\n') { println!(); }
    Ok(())
}
