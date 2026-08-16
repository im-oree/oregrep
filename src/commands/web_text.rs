use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebTextArgs {
    url: String,

    /// CSS selector to limit extraction (default: body)
    #[arg(short = 's', long, default_value = "body")]
    selector: String,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    /// Write to file
    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,

    #[arg(short = 'w', long)]
    wait_selector: Option<String>,
}

pub fn run(args: WebTextArgs) -> Result<()> {
    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;
    let el = tab.wait_for_element(&args.selector)?;
    let text = el.get_inner_text()?;
    match args.output {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&p, &text)?;
            println!("{} {}  ({} chars)",
                "Wrote:".green().bold(),
                p.display().to_string().cyan(),
                text.chars().count().to_string().yellow());
        }
        None => print!("{}", text),
    }
    Ok(())
}
