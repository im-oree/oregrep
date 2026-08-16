use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_imports, extract_symbols, SymbolKind};

#[derive(Args)]
pub struct ExplainArgs {
    file: PathBuf,
}

pub fn run(args: ExplainArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let lines = content.lines().count();
    let bytes = content.len();
    let imports = extract_imports(&content, &args.file);
    let symbols = extract_symbols(&content, &args.file);

    let external: Vec<&str> = imports.iter()
        .map(|i| i.source.as_str())
        .filter(|s| !s.starts_with('.') && !s.starts_with('/'))
        .collect();
    let internal: Vec<&str> = imports.iter()
        .map(|i| i.source.as_str())
        .filter(|s| s.starts_with('.') || s.starts_with('/'))
        .collect();

    let hooks: Vec<&crate::engine::symbols::Symbol> = symbols.iter().filter(|s| matches!(s.kind, SymbolKind::Hook)).collect();
    let comps: Vec<&crate::engine::symbols::Symbol> = symbols.iter().filter(|s| matches!(s.kind, SymbolKind::Component)).collect();
    let classes: Vec<&crate::engine::symbols::Symbol> = symbols.iter().filter(|s| matches!(s.kind, SymbolKind::Class)).collect();
    let types: Vec<&crate::engine::symbols::Symbol> = symbols.iter().filter(|s| matches!(s.kind, SymbolKind::Type | SymbolKind::Interface)).collect();
    let fns: Vec<&crate::engine::symbols::Symbol> = symbols.iter().filter(|s| matches!(s.kind, SymbolKind::Function)).collect();
    let exported = symbols.iter().filter(|s| s.exported).count();

    println!("{} {}", "Explain:".cyan().bold(), args.file.display().to_string().yellow());
    println!("  Size: {} lines, {} bytes", lines.to_string().yellow(), bytes.to_string().dimmed());
    println!("  Symbols: {} total, {} exported", symbols.len().to_string().yellow(), exported.to_string().green());

    let mut summary_parts: Vec<String> = Vec::new();
    if !comps.is_empty() { summary_parts.push(format!("{} React component{}", comps.len(), if comps.len() == 1 { "" } else { "s" })); }
    if !hooks.is_empty() { summary_parts.push(format!("{} custom hook{}", hooks.len(), if hooks.len() == 1 { "" } else { "s" })); }
    if !classes.is_empty() { summary_parts.push(format!("{} class{}", classes.len(), if classes.len() == 1 { "" } else { "es" })); }
    if !fns.is_empty() { summary_parts.push(format!("{} function{}", fns.len(), if fns.len() == 1 { "" } else { "s" })); }
    if !types.is_empty() { summary_parts.push(format!("{} type{}/interface{}", types.len(), if types.len() == 1 { "" } else { "s" }, if types.len() == 1 { "" } else { "s" })); }

    println!("\n{}", "Summary:".bold());
    if summary_parts.is_empty() {
        println!("  This file has no top-level definitions detected.");
    } else {
        println!("  Contains: {}", summary_parts.join(", "));
    }

    if !imports.is_empty() {
        println!("\n{} {} imports ({} external, {} local)",
            "Dependencies:".bold(),
            imports.len().to_string().yellow(),
            external.len().to_string().yellow(),
            internal.len().to_string().yellow());
        if !external.is_empty() {
            let deps: Vec<String> = external.iter().take(10).map(|s| s.to_string()).collect();
            println!("  external: {}", deps.join(", ").magenta());
        }
    }

    if !comps.is_empty() {
        println!("\n{}", "Components:".bold());
        for c in comps.iter().take(10) { println!("  • {}  (L{})", c.name.cyan(), c.line.to_string().dimmed()); }
    }
    if !hooks.is_empty() {
        println!("\n{}", "Hooks:".bold());
        for h in hooks.iter().take(10) { println!("  • {}  (L{})", h.name.cyan(), h.line.to_string().dimmed()); }
    }
    if !classes.is_empty() {
        println!("\n{}", "Classes:".bold());
        for c in classes.iter().take(10) { println!("  • {}  (L{})", c.name.cyan(), c.line.to_string().dimmed()); }
    }

    // Heuristic role
    println!("\n{}", "Likely role:".bold());
    let path_str = args.file.to_string_lossy().to_lowercase();
    let role = if path_str.contains("test") || path_str.contains("spec") { "Test file" }
        else if !hooks.is_empty() && comps.is_empty() { "Custom hooks module" }
        else if !comps.is_empty() && !hooks.is_empty() { "React component with local hooks" }
        else if !comps.is_empty() { "React component module" }
        else if !classes.is_empty() { "Class-based module" }
        else if types.len() > fns.len() + classes.len() { "Type definitions" }
        else if fns.len() > 5 && comps.is_empty() { "Utility / library module" }
        else if imports.is_empty() && exported == 0 { "Script / entry" }
        else { "General module" };
    println!("  {}", role.green());

    Ok(())
}
