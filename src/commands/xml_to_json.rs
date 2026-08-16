use anyhow::Result;
use clap::Args;
use colored::*;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct XmlToJsonArgs {
    file: PathBuf,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(short = 'c', long)]
    compact: bool,
}

/// Simple XML → JSON: each element becomes { "@attr": {..}, "#text": "..", "<child>": [..] }
pub fn run(args: XmlToJsonArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    let mut stack: Vec<serde_json::Map<String, serde_json::Value>> = vec![serde_json::Map::new()];
    let mut name_stack: Vec<String> = vec!["#root".to_string()];
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut obj = serde_json::Map::new();
                for a in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                    let v = String::from_utf8_lossy(&a.value).to_string();
                    obj.entry(format!("@{}", k)).or_insert(serde_json::Value::String(v));
                }
                stack.push(obj);
                name_stack.push(name);
            }
            Ok(Event::End(_)) => {
                let child = stack.pop().unwrap_or_default();
                let name = name_stack.pop().unwrap_or_default();
                let parent = stack.last_mut().unwrap();
                let val = serde_json::Value::Object(child);
                match parent.get_mut(&name) {
                    Some(serde_json::Value::Array(a)) => a.push(val),
                    Some(existing) => {
                        let old = std::mem::replace(existing, serde_json::Value::Null);
                        *existing = serde_json::Value::Array(vec![old, val]);
                    }
                    None => { parent.insert(name, val); }
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut obj = serde_json::Map::new();
                for a in e.attributes().flatten() {
                    let k = String::from_utf8_lossy(a.key.as_ref()).to_string();
                    let v = String::from_utf8_lossy(&a.value).to_string();
                    obj.insert(format!("@{}", k), serde_json::Value::String(v));
                }
                let parent = stack.last_mut().unwrap();
                let val = serde_json::Value::Object(obj);
                match parent.get_mut(&name) {
                    Some(serde_json::Value::Array(a)) => a.push(val),
                    Some(existing) => {
                        let old = std::mem::replace(existing, serde_json::Value::Null);
                        *existing = serde_json::Value::Array(vec![old, val]);
                    }
                    None => { parent.insert(name, val); }
                }
            }
            Ok(Event::Text(e)) => {
                let txt = e.unescape().unwrap_or_default().to_string();
                if !txt.trim().is_empty() {
                    if let Some(top) = stack.last_mut() {
                        top.insert("#text".to_string(), serde_json::Value::String(txt));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => anyhow::bail!("XML parse: {}", e),
            _ => {}
        }
        buf.clear();
    }
    let root = stack.pop().unwrap_or_default();
    let json = serde_json::Value::Object(root);
    let out = if args.compact { serde_json::to_string(&json)? } else { serde_json::to_string_pretty(&json)? };
    match args.output {
        Some(p) => {
            std::fs::write(&p, out)?;
            println!("{} {}", "Wrote:".green().bold(), p.display().to_string().cyan());
        }
        None => println!("{}", out),
    }
    Ok(())
}
