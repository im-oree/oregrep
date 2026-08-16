use anyhow::Result;
use clap::Args;
use colored::*;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use std::path::PathBuf;

use crate::engine::web::{fmt_bytes, parse_size_list, WebSession};

#[derive(Args)]
pub struct WebScreenshotSetArgs {
    url: String,

    /// Comma-separated widths (heights auto-scale via viewport aspect ratio)
    #[arg(short = 's', long, default_value = "375,768,1024,1440,1920")]
    sizes: String,

    /// Aspect ratio for viewport height (height = width / ratio). Default 16/9.
    #[arg(short = 'a', long, default_value = "1.7777")]
    aspect: f32,

    #[arg(short = 'o', long, default_value = "./screenshots")]
    out_dir: PathBuf,

    #[arg(short = 'F', long)]
    full_page: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: WebScreenshotSetArgs) -> Result<()> {
    let sizes = parse_size_list(&args.sizes)?;
    std::fs::create_dir_all(&args.out_dir)?;

    println!("{} {} @ {} widths → {}",
        "Screenshot set:".cyan().bold(),
        args.url.yellow(),
        sizes.len().to_string().yellow(),
        args.out_dir.display().to_string().cyan());

    for w in &sizes {
        let h = ((*w as f32) / args.aspect).round() as u32;
        let session = WebSession::launch(true, Some((*w, h)))?;
        let tab = session.open(&args.url, None, args.timeout)?;
        let bytes = tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, args.full_page)?;
        let out = args.out_dir.join(format!("shot-{}.png", w));
        std::fs::write(&out, &bytes)?;
        println!("  {} {}x{}  {}  ({})",
            "+".green(),
            w, h,
            out.display().to_string().cyan(),
            fmt_bytes(bytes.len() as u64).dimmed());
        let _ = tab.close(false);
        drop(session);
    }
    Ok(())
}
