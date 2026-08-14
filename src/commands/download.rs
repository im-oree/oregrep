use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::confirm::confirm;
use crate::engine::http::{apply_headers, build_client, filename_from_url, fmt_bytes, parse_headers_from_flags, read_body_with_progress};

#[derive(Args)]
pub struct DownloadArgs {
    url: String,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(long)]
    force: bool,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "300")]
    timeout: u64,

    #[arg(long)]
    proxy: Option<String>,

    #[arg(short = 'y', long)]
    yes: bool,

    /// Disable progress bar (useful for scripts)
    #[arg(long)]
    no_progress: bool,
}

pub fn run(args: DownloadArgs) -> Result<()> {
    let target = args.output.clone().unwrap_or(filename_from_url(&args.url)?);
    if target.exists() && !args.force {
        let ok = confirm(&format!("File exists: {}. Overwrite?", target.display()), args.yes)?;
        if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
    }

    let client = build_client(args.timeout, true, args.proxy.as_deref())?;
    let hdrs = parse_headers_from_flags(&args.headers)?;
    let req = apply_headers(client.get(&args.url), &hdrs);
    let start = std::time::Instant::now();
    let resp = req.send()?;
    if !resp.status().is_success() {
        anyhow::bail!("HTTP {} {}", resp.status().as_u16(), resp.status().canonical_reason().unwrap_or(""));
    }
    let total = resp.content_length();

    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut file = std::fs::File::create(&target)?;
    let written = if args.no_progress {
        let bytes = crate::engine::http::read_body_bytes(resp)?;
        std::io::Write::write_all(&mut file, &bytes)?;
        bytes.len() as u64
    } else {
        read_body_with_progress(resp, total, &mut file)?
    };
    let elapsed = start.elapsed().as_millis();

    println!("{} {}  ({}, {} ms)",
        "Downloaded:".green().bold(),
        target.display().to_string().cyan(),
        fmt_bytes(written).yellow(),
        elapsed.to_string().dimmed()
    );
    Ok(())
}
