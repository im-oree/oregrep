use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_imports, extract_symbols, SymbolKind};

#[derive(Args)]
pub struct PluckArgs {
    file: PathBuf,

    #[arg(long)]
    exports: bool,
    #[arg(long)]
    imports: bool,
    #[arg(long)]
    types: bool,
    #[arg(long)]
    interfaces: bool,
    #[arg(long)]
    signatures: bool,
    #[arg(long)]
    hooks: bool,
    #[arg(long)]
    components: bool,

    /// Include line numbers
    #[arg(short = 'N', long)]
    number: bool,
}

pub fn run(args: PluckArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let syms = extract_symbols(&content, &args.file);

    // If no filter flags, show everything (like outline)
    let any_flag = args.exports || args.imports || args.types || args.interfaces
                || args.signatures || args.hooks || args.components;

    if !any_flag || args.imports {
        let imps = extract_imports(&content, &args.file);
        if !imps.is_empty() {
            println!("{}", "Imports:".cyan().bold());
            for i in &imps {
                let named = if i.named.is_empty() { String::new() } else { format!("{{{}}}", i.named.join(", ")) };
                let def = i.default.as_ref().map(|s| format!("{} ", s)).unwrap_or_default();
                let ns = i.namespace.as_ref().map(|s| format!("* as {} ", s)).unwrap_or_default();
                if args.number {
                    println!("  L{:<4} {}{}{}from '{}'", i.line.to_string().dimmed(), def, named, ns, i.source.yellow());
                } else {
                    println!("  {}{}{}from '{}'", def, named, ns, i.source.yellow());
                }
            }
        }
    }

    if !any_flag || args.exports {
        let exp: Vec<_> = syms.iter().filter(|s| s.exported).collect();
        if !exp.is_empty() {
            println!("\n{}", "Exports:".cyan().bold());
            for s in exp {
                if args.number {
                    println!("  L{:<4} {:<6} {}", s.line.to_string().dimmed(), s.kind.short().magenta(), s.name.yellow());
                } else {
                    println!("  {:<6} {}", s.kind.short().magenta(), s.name.yellow());
                }
            }
        }
    }

    if args.types {
        let ts: Vec<_> = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Type)).collect();
        println!("\n{}", "Types:".cyan().bold());
        for s in ts { println!("  {}", s.name.yellow()); }
    }
    if args.interfaces {
        let is: Vec<_> = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Interface)).collect();
        println!("\n{}", "Interfaces:".cyan().bold());
        for s in is { println!("  {}", s.name.yellow()); }
    }
    if args.hooks {
        let hs: Vec<_> = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Hook)).collect();
        println!("\n{}", "Hooks:".cyan().bold());
        for s in hs { println!("  {}", s.name.yellow()); }
    }
    if args.components {
        let cs: Vec<_> = syms.iter().filter(|s| matches!(s.kind, SymbolKind::Component)).collect();
        println!("\n{}", "Components:".cyan().bold());
        for s in cs { println!("  {}", s.name.yellow()); }
    }
    if args.signatures {
        println!("\n{}", "Signatures:".cyan().bold());
        let re = regex::Regex::new(r"(?m)^\s*(?:export\s+(?:default\s+)?)?(?:async\s+)?function\s+\w+[^{]+").unwrap();
        for cap in re.find_iter(&content) {
            let sig = cap.as_str().trim().trim_end_matches('{').trim().to_string();
            println!("  {}", sig.yellow());
        }
    }
    Ok(())
}
