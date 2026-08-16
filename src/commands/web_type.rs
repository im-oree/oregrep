use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebTypeArgs {
    url: String,
    selector: String,
    text: String,

    /// Press Enter after typing
    #[arg(long)]
    submit: bool,

    /// Clear existing value before typing
    #[arg(short = 'c', long)]
    clear: bool,

    #[arg(short = 'V', long)]
    visible: bool,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,
}

pub fn run(args: WebTypeArgs) -> Result<()> {
    let session = WebSession::launch(!args.visible, None)?;
    let tab = session.open(&args.url, None, args.timeout)?;
    let el = tab.wait_for_element(&args.selector)?;
    el.click()?;
    if args.clear {
        // Select all + delete
        tab.press_key("Control+a")?;
        tab.press_key("Delete")?;
    }
    tab.type_str(&args.text)?;
    if args.submit {
        tab.press_key("Enter")?;
        // Give the page a moment to navigate/react
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    println!("{} typed into {}", "OK".green().bold(), args.selector.yellow());
    Ok(())
}
