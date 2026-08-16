use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebHtmlArgs {
    url: String,

    /// CSS selector to extract (default: entire document)
    #[arg(short = 's', long)]
    selector: Option<String>,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,

    #[arg(short = 'w', long)]
    wait_selector: Option<String>,
}

pub fn run(args: WebHtmlArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;

    let html = if let Some(sel) = &args.selector {
        let el = tab.wait_for_element(sel)?;
        el.get_content()?
    } else {
        tab.get_content()?
    };

    match args.output {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&p, &html)?;
            println!("{} {}  ({} bytes)",
                "Wrote:".green().bold(),
                p.display().to_string().cyan(),
                html.len().to_string().yellow());
        }
        None => print!("{}", html),
    }
    Ok(())
}
