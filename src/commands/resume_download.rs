use anyhow::Result;
use clap::Args;
use colored::*;
use std::io::Write;
use std::path::PathBuf;

use crate::engine::http::{apply_headers, build_client, filename_from_url, fmt_bytes, parse_headers_from_flags, read_body_with_progress};

#[derive(Args)]
pub struct ResumeDownloadArgs {
    url: String,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    #[arg(short = 't', long, default_value = "600")]
    timeout: u64,

    /// Force restart (ignore existing partial file)
    #[arg(long)]
    restart: bool,
}

pub fn run(args: ResumeDownloadArgs) -> Result<()> {
    let target = args.output.clone().unwrap_or(filename_from_url(&args.url)?);
    let client = build_client(args.timeout, true, None)?;

    // Check existing partial
    let mut existing_bytes: u64 = 0;
    if target.exists() && !args.restart {
        existing_bytes = std::fs::metadata(&target)?.len();
        if existing_bytes > 0 {
            println!("{} {} bytes already downloaded, resuming...",
                "Resume:".cyan().bold(), existing_bytes.to_string().yellow());
        }
    } else if target.exists() && args.restart {
        std::fs::remove_file(&target)?;
    }

    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let mut hdrs = parse_headers_from_flags(&args.headers)?;
    if existing_bytes > 0 {
        hdrs.insert("Range".to_string(), format!("bytes={}-", existing_bytes));
    }
    let req = apply_headers(client.get(&args.url), &hdrs);
    let start = std::time::Instant::now();
    let resp = req.send()?;
    let status = resp.status().as_u16();
    // 206 Partial Content = server honored range; 200 = full restart
    if status == 200 && existing_bytes > 0 {
        println!("{} Server ignored Range header, restarting from 0", "!".yellow());
        std::fs::remove_file(&target)?;
        existing_bytes = 0;
    } else if status != 200 && status != 206 {
        anyhow::bail!("HTTP {} {}", status, resp.status().canonical_reason().unwrap_or(""));
    }

    // Total from Content-Range or Content-Length + existing
    let remaining = resp.content_length();
    let total_size = remaining.map(|r| r + existing_bytes);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(existing_bytes > 0)
        .write(true)
        .open(&target)?;

    // Use progress bar with total
    struct OffsetWriter<'a> {
        inner: &'a mut std::fs::File,
    }
    impl<'a> Write for OffsetWriter<'a> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.inner.write(buf) }
        fn flush(&mut self) -> std::io::Result<()> { self.inner.flush() }
    }
    let mut w = OffsetWriter { inner: &mut file };
    let written = read_body_with_progress(resp, remaining, &mut w)?;
    let elapsed = start.elapsed().as_millis();

    let total_now = existing_bytes + written;
    println!("{} {}  ({} downloaded now, {} total on disk, {}ms)",
        "Done:".green().bold(),
        target.display().to_string().cyan(),
        fmt_bytes(written).yellow(),
        fmt_bytes(total_now).yellow(),
        elapsed.to_string().dimmed()
    );
    if let Some(expected) = total_size {
        if total_now < expected {
            println!("{} still partial ({}/{})", "!".yellow(),
                fmt_bytes(total_now), fmt_bytes(expected));
        }
    }
    Ok(())
}
