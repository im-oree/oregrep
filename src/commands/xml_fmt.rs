use anyhow::Result;
use clap::Args;
use colored::*;
use quick_xml::events::Event;
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;

#[derive(Args)]
pub struct XmlFmtArgs {
    file: PathBuf,
    /// Indent width
    #[arg(short = 'w', long, default_value = "2")]
    width: usize,
    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: XmlFmtArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', args.width);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e)?,
            Err(e) => anyhow::bail!("XML parse: {}", e),
        }
        buf.clear();
    }
    let out_bytes = writer.into_inner().into_inner();
    let out = String::from_utf8(out_bytes)?;
    let target = args.output.clone().unwrap_or_else(|| args.file.clone());
    if target == args.file && !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    std::fs::write(&target, out)?;
    println!("{} {}", "Formatted:".green().bold(), target.display().to_string().cyan());
    Ok(())
}
