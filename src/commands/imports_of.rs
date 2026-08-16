use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_imports, resolve_ts_import};

#[derive(Args)]
pub struct ImportsOfArgs {
    file: PathBuf,

    /// Resolve relative imports to real files
    #[arg(short = 'r', long)]
    resolve: bool,

    /// JSON output
    #[arg(short = 'j', long)]
    json: bool,
}

pub fn run(args: ImportsOfArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let imports = extract_imports(&content, &args.file);

    if args.json { println!("{}", serde_json::to_string_pretty(&imports)?); return Ok(()); }

    println!("{} {}  ({} imports)", "Imports of:".cyan().bold(), args.file.display().to_string().yellow(), imports.len().to_string().dimmed());
    for i in &imports {
        let named = if i.named.is_empty() { String::new() } else { format!("{{{}}} ", i.named.join(", ")) };
        let def = i.default.as_ref().map(|s| format!("{} ", s)).unwrap_or_default();
        let ns = i.namespace.as_ref().map(|s| format!("* as {} ", s)).unwrap_or_default();
        let resolved = if args.resolve {
            match resolve_ts_import(&args.file, &i.source) {
                Some(p) => format!("  {} {}", "→".dimmed(), p.display().to_string().green()),
                None => format!("  {} {}", "→".dimmed(), "(external)".dimmed()),
            }
        } else { String::new() };
        println!("  L{:<4} {}{}{}from '{}'{}", i.line.to_string().dimmed(), def, named, ns, i.source.yellow(), resolved);
    }
    Ok(())
}
