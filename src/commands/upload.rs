use anyhow::Result;
use clap::Args;
use colored::*;
use reqwest::blocking::multipart::{Form, Part};
use std::path::PathBuf;

use crate::engine::http::{apply_headers, build_client, parse_headers_from_flags, read_body_bytes, status_color};

#[derive(Args)]
pub struct UploadArgs {
    /// URL to upload to
    url: String,

    /// File(s) as "fieldname=path"; repeat for multiple
    #[arg(short = 'f', long = "file", required = true)]
    files: Vec<String>,

    /// Additional form fields "key=value"
    #[arg(short = 'F', long = "field")]
    fields: Vec<String>,

    #[arg(short = 'X', long, default_value = "POST")]
    method: String,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "600")]
    timeout: u64,

    #[arg(long)]
    proxy: Option<String>,

    #[arg(short = 'i', long)]
    include_headers: bool,
}

pub fn run(args: UploadArgs) -> Result<()> {
    let client = build_client(args.timeout, true, args.proxy.as_deref())?;
    let mut form = Form::new();

    for spec in &args.files {
        let (field, path) = spec.split_once('=').ok_or_else(|| anyhow::anyhow!("Bad --file (expected field=path): {}", spec))?;
        let pb = PathBuf::from(path);
        if !pb.exists() { anyhow::bail!("File not found: {}", pb.display()); }
        let file_name = pb.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let bytes = std::fs::read(&pb)?;
        let part = Part::bytes(bytes).file_name(file_name);
        form = form.part(field.to_string(), part);
        println!("  {} {} = {}", "→".cyan(), field.cyan(), pb.display().to_string().yellow());
    }
    for f in &args.fields {
        let (k, v) = f.split_once('=').ok_or_else(|| anyhow::anyhow!("Bad --field (expected k=v): {}", f))?;
        form = form.text(k.to_string(), v.to_string());
    }

    let method = reqwest::Method::from_bytes(args.method.to_uppercase().as_bytes())?;
    let hdrs = parse_headers_from_flags(&args.headers)?;
    let req = apply_headers(client.request(method, &args.url).multipart(form), &hdrs);
    let start = std::time::Instant::now();
    let resp = req.send()?;
    let status = resp.status().as_u16();
    let response_headers: Vec<(String, String)> = resp.headers().iter().map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect();
    let body = read_body_bytes(resp)?;
    let elapsed = start.elapsed().as_millis();

    let color = status_color(status);
    println!("{} {} {}  {}  ({}ms)",
        "Uploaded".color(color).bold(),
        format!("HTTP {}", status).color(color).bold(),
        format!("({} bytes response)", body.len()).dimmed(),
        args.url.cyan(),
        elapsed.to_string().dimmed()
    );
    if args.include_headers {
        for (k, v) in &response_headers { println!("  {}: {}", k.cyan(), v); }
    }
    let text = String::from_utf8_lossy(&body);
    if !text.is_empty() { println!("\n{}", text); }
    Ok(())
}
