use anyhow::Result;
use clap::Args;
use colored::*;
use headless_chrome::types::PrintToPdfOptions;
use std::path::PathBuf;

use crate::engine::web::{fmt_bytes, WebSession};

#[derive(Args)]
pub struct WebPdfArgs {
    url: String,

    #[arg(short = 'o', long, default_value = "page.pdf")]
    output: PathBuf,

    /// Landscape orientation
    #[arg(short = 'L', long)]
    landscape: bool,

    /// Print backgrounds (CSS colors + images)
    #[arg(short = 'b', long, default_value = "true")]
    background: bool,

    /// Margin in inches (all sides)
    #[arg(short = 'm', long, default_value = "0.4")]
    margin: f64,

    #[arg(short = 't', long, default_value = "60")]
    timeout: u64,

    /// Wait for selector before printing
    #[arg(short = 'w', long)]
    wait_selector: Option<String>,
}

pub fn run(args: WebPdfArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    println!("{} {}", "PDF:".cyan().bold(), args.url.yellow());

    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;
    let opts = PrintToPdfOptions {
        landscape: Some(args.landscape),
        display_header_footer: Some(false),
        print_background: Some(args.background),
        scale: Some(1.0),
        paper_width: None,
        paper_height: None,
        margin_top: Some(args.margin),
        margin_bottom: Some(args.margin),
        margin_left: Some(args.margin),
        margin_right: Some(args.margin),
        page_ranges: None,
        header_template: None,
        footer_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
        ignore_invalid_page_ranges: None,
        generate_document_outline: None,
        generate_tagged_pdf: None,
    };
    let pdf_bytes = tab.print_to_pdf(Some(opts))?;
    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&args.output, &pdf_bytes)?;
    let sz = std::fs::metadata(&args.output).map(|m| m.len()).unwrap_or(0);
    println!("{} {}  ({})",
        "Wrote:".green().bold(),
        args.output.display().to_string().cyan(),
        fmt_bytes(sz).yellow());
    Ok(())
}
