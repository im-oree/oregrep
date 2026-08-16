use anyhow::Result;
use clap::Args;
use colored::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct XmlGetArgs {
    file: PathBuf,
    /// Element name to extract text from (all occurrences)
    element: String,
    /// Optional attribute name — print only this attribute value instead of text
    #[arg(short = 'a', long)]
    attr: Option<String>,
}

pub fn run(args: XmlGetArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_target = false;
    let mut count = 0usize;
    let mut current_text = String::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == args.element {
                    if let Some(attr_name) = &args.attr {
                        for a in e.attributes().flatten() {
                            let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                            if &k == attr_name {
                                let v = String::from_utf8_lossy(&a.value).to_string();
                                println!("{}", v);
                                count += 1;
                            }
                        }
                    } else {
                        in_target = true;
                        current_text.clear();
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_target {
                    current_text.push_str(&e.unescape().unwrap_or_default());
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == args.element && in_target {
                    println!("{}", current_text);
                    count += 1;
                    in_target = false;
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == args.element {
                    if let Some(attr_name) = &args.attr {
                        for a in e.attributes().flatten() {
                            let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                            if &k == attr_name {
                                let v = String::from_utf8_lossy(&a.value).to_string();
                                println!("{}", v);
                                count += 1;
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse error: {}", e),
            _ => {}
        }
        buf.clear();
    }
    eprintln!("\n{} {} matches", "Total:".dimmed(), count.to_string().yellow());
    Ok(())
}
