use anyhow::Result;
use clap::Args;
use colored::*;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use std::path::PathBuf;

use crate::engine::web::{device_viewport, fmt_bytes, parse_viewport, WebSession};

#[derive(Args)]
pub struct WebScreenshotArgs {
    url: String,

    #[arg(short = 'o', long, default_value = "screenshot.png")]
    output: PathBuf,

    /// Full page (default: viewport only)
    #[arg(short = 'f', long)]
    full_page: bool,

    /// CSS selector: screenshot just this element
    #[arg(short = 's', long)]
    selector: Option<String>,

    /// Viewport: WIDTHxHEIGHT (e.g. 1920x1080)
    #[arg(long)]
    viewport: Option<String>,

    /// Device preset (iphone-14, ipad, desktop, fhd, 4k, ...)
    #[arg(short = 'd', long)]
    device: Option<String>,

    /// Wait for selector before capture
    #[arg(short = 'w', long)]
    wait_selector: Option<String>,

    /// Format: png (default) or jpeg
    #[arg(short = 'F', long, default_value = "png")]
    format: String,

    /// JPEG quality (1..=100), ignored for PNG
    #[arg(short = 'q', long, default_value = "90")]
    quality: u8,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'k', long, default_value = "0")]
    delay: u64,
}

pub fn run(args: WebScreenshotArgs) -> Result<()> {
    let viewport = if let Some(dev) = &args.device {
        device_viewport(dev).ok_or_else(|| anyhow::anyhow!("Unknown device preset: {}", dev))?
    } else if let Some(vp) = &args.viewport {
        parse_viewport(vp)?
    } else {
        (1280, 720)
    };

    let session = WebSession::launch(true, Some(viewport))?;
    println!("{} {} @ {}x{}",
        "Screenshot:".cyan().bold(),
        args.url.yellow(),
        viewport.0, viewport.1);

    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;
    if args.delay > 0 {
        std::thread::sleep(std::time::Duration::from_millis(args.delay * 1000));
    }

    let fmt = if args.format.eq_ignore_ascii_case("jpeg") || args.format.eq_ignore_ascii_case("jpg") {
        CaptureScreenshotFormatOption::Jpeg
    } else {
        CaptureScreenshotFormatOption::Png
    };
    let quality = if matches!(fmt, CaptureScreenshotFormatOption::Jpeg) { Some(args.quality as u32) } else { None };

    let bytes = if let Some(sel) = &args.selector {
        let el = tab.wait_for_element(sel)?;
        el.capture_screenshot(fmt)?
    } else {
        tab.capture_screenshot(fmt, quality, None, args.full_page)?
    };

    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&args.output, &bytes)?;
    let sz = std::fs::metadata(&args.output).map(|m| m.len()).unwrap_or(0);
    println!("{} {}  ({})",
        "Wrote:".green().bold(),
        args.output.display().to_string().cyan(),
        fmt_bytes(sz).yellow());
    Ok(())
}
