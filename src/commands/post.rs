use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::http::{apply_headers, build_client, parse_headers_from_flags, read_body_bytes, status_color};

#[derive(Args)]
pub struct PostArgs {
    url: String,

    #[arg(short = 'd', long)]
    data: Option<String>,

    #[arg(long)]
    file: Option<PathBuf>,

    #[arg(short = 'j', long)]
    json: Option<String>,

    /// Form field, repeatable: --form key=value
    #[arg(short = 'F', long = "form")]
    form: Vec<String>,

    #[arg(short = 'X', long, default_value = "POST")]
    method: String,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "60")]
    timeout: u64,

    #[arg(long)]
    no_redirect: bool,

    #[arg(long)]
    proxy: Option<String>,

    #[arg(short = 'i', long)]
    include_headers: bool,

    #[arg(short = 'q', long)]
    no_body: bool,

    #[arg(long)]
    pretty: bool,
}

pub fn run(args: PostArgs) -> Result<()> {
    let client = build_client(args.timeout, !args.no_redirect, args.proxy.as_deref())?;
    let method = reqwest::Method::from_bytes(args.method.to_uppercase().as_bytes())?;
    let mut hdrs = parse_headers_from_flags(&args.headers)?;
    let mut req = client.request(method, &args.url);

    if let Some(j) = &args.json {
        hdrs.entry("Content-Type".to_string()).or_insert_with(|| "application/json".to_string());
        req = req.body(j.clone());
    } else if !args.form.is_empty() {
        let mut form_pairs: Vec<(String, String)> = Vec::new();
        for f in &args.form {
            if let Some(idx) = f.find('=') {
                let (k, v) = f.split_at(idx);
                form_pairs.push((k.to_string(), v[1..].to_string()));
            } else {
                anyhow::bail!("Bad form field (expected key=value): {}", f);
            }
        }
        req = req.form(&form_pairs);
    } else if let Some(f) = &args.file {
        req = req.body(std::fs::read(f)?);
    } else if let Some(d) = &args.data {
        req = req.body(d.clone());
    }

    req = apply_headers(req, &hdrs);
    let start = std::time::Instant::now();
    let resp = req.send()?;
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let final_url = resp.url().to_string();
    let response_headers: Vec<(String, String)> = resp.headers().iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
    let body = read_body_bytes(resp)?;
    let elapsed = start.elapsed().as_millis();

    let color = status_color(status);
    eprintln!("{} {} {}  {}  {}",
        format!("{} {}", args.method.to_uppercase(), status).color(color).bold(),
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
