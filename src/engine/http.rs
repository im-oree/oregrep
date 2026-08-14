use anyhow::{Context, Result};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::redirect::Policy;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

/// Build a shared HTTP client with common settings.
pub fn build_client(timeout_secs: u64, follow_redirects: bool, proxy: Option<&str>) -> Result<Client> {
    let mut b = ClientBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(if follow_redirects { Policy::limited(10) } else { Policy::none() })
        .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
        .gzip(true)
        .brotli(true)
        .deflate(true);
    if let Some(p) = proxy {
        b = b.proxy(reqwest::Proxy::all(p)?);
    }
    Ok(b.build()?)
}

/// Reserved for the upcoming `session` command (cookie-persistent client).
#[allow(dead_code)]
pub fn build_client_with_cookies(timeout_secs: u64, follow_redirects: bool) -> Result<Client> {
    let b = ClientBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(if follow_redirects { Policy::limited(10) } else { Policy::none() })
        .user_agent(concat!("ore/", env!("CARGO_PKG_VERSION")))
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true);
    Ok(b.build()?)
}

/// Parse a "Key: Value" header string
pub fn parse_header_arg(s: &str) -> Result<(String, String)> {
    if let Some(idx) = s.find(':') {
        let (k, v) = s.split_at(idx);
        Ok((k.trim().to_string(), v[1..].trim().to_string()))
    } else {
        anyhow::bail!("Invalid header format (expected 'Key: Value'): {}", s)
    }
}

/// Apply headers HashMap to a request builder.
pub fn apply_headers(req: reqwest::blocking::RequestBuilder, headers: &HashMap<String, String>) -> reqwest::blocking::RequestBuilder {
    let mut b = req;
    for (k, v) in headers {
        b = b.header(k, v);
    }
    b
}

/// Read full response body as bytes.
pub fn read_body_bytes(mut resp: Response) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    resp.read_to_end(&mut buf).context("Reading response body")?;
    Ok(buf)
}

/// Read body while showing a progress bar (for downloads).
pub fn read_body_with_progress(mut resp: Response, total: Option<u64>, out: &mut dyn std::io::Write) -> Result<u64> {
    use indicatif::{ProgressBar, ProgressStyle};
    let pb = if let Some(t) = total {
        let p = ProgressBar::new(t);
        p.set_style(ProgressStyle::with_template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {eta}")
            .unwrap()
            .progress_chars("█▊ "));
        p
    } else {
        let p = ProgressBar::new_spinner();
        p.set_style(ProgressStyle::with_template("  {spinner} {bytes} ({bytes_per_sec})").unwrap());
        p
    };

    let mut buf = [0u8; 65536];
    let mut written = 0u64;
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 { break; }
        out.write_all(&buf[..n])?;
        written += n as u64;
        pb.set_position(written);
    }
    pb.finish_and_clear();
    Ok(written)
}

/// Human-readable byte size
pub fn fmt_bytes(bytes: u64) -> String {
    if bytes < 1024 { format!("{}B", bytes) }
    else if bytes < 1024 * 1024 { format!("{:.1}K", bytes as f64 / 1024.0) }
    else if bytes < 1024 * 1024 * 1024 { format!("{:.1}M", bytes as f64 / 1024.0 / 1024.0) }
    else { format!("{:.2}G", bytes as f64 / 1024.0 / 1024.0 / 1024.0) }
}

/// Color code for HTTP status.
pub fn status_color(code: u16) -> &'static str {
    match code {
        200..=299 => "green",
        300..=399 => "cyan",
        400..=499 => "yellow",
        500..=599 => "red",
        _ => "white",
    }
}

pub fn parse_headers_from_flags(flags: &[String]) -> Result<HashMap<String, String>> {
    let mut m = HashMap::new();
    for h in flags {
        let (k, v) = parse_header_arg(h)?;
        m.insert(k, v);
    }
    Ok(m)
}

/// Extract filename from URL path (last segment or "download")
pub fn filename_from_url(url_str: &str) -> Result<std::path::PathBuf> {
    let parsed = url::Url::parse(url_str)?;
    let last = parsed
        .path_segments()
        .and_then(|mut s| s.next_back())
        .unwrap_or("download")
        .to_string();
    let name = if last.is_empty() { "download".to_string() } else { last };
    Ok(std::path::PathBuf::from(name))
}
