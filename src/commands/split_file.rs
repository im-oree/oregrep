use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::symbols::{extract_symbols, extract_body, language_of, Symbol, SymbolKind};

#[derive(Args)]
pub struct SplitFileArgs {
    file: PathBuf,

    /// Output directory (default: same dir, "<file-stem>/")
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,

    /// Also keep the original file as a barrel hub re-exporting each split (so imports don't break)
    #[arg(short = 'k', long)]
    keep_hub: bool,

    /// What to split by: fn | class | export | all (default all-exported)
    #[arg(short = 'b', long, default_value = "export")]
    by: String,

    /// File extension for output files (default: same as input)
    #[arg(short = 'e', long)]
    ext: Option<String>,

    /// Naming: kebab (default) or exact
    #[arg(short = 'n', long, default_value = "kebab")]
    naming: String,

    /// Include imports from source file in each output
    #[arg(short = 'i', long, default_value = "true")]
    carry_imports: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: SplitFileArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let all_symbols = extract_symbols(&content, &args.file);
    let lang = language_of(&args.file);
    if lang == "other" { anyhow::bail!("Unsupported language for split: {}", args.file.display()); }

    // Filter symbols
    let symbols: Vec<&Symbol> = all_symbols.iter().filter(|s| match args.by.as_str() {
        "fn" => matches!(s.kind, SymbolKind::Function | SymbolKind::Hook | SymbolKind::Component),
        "class" => matches!(s.kind, SymbolKind::Class),
        "export" => s.exported,
        "all" => true,
        _ => s.exported,
    }).collect();

    if symbols.is_empty() {
        anyhow::bail!("No matching symbols in {} (by={})", args.file.display(), args.by);
    }

    // Extract imports section (everything before first symbol)
    let imports_header = if args.carry_imports {
        let first_line = symbols.iter().map(|s| s.line).min().unwrap_or(1);
        content.lines().take(first_line.saturating_sub(1))
            .filter(|l| {
                let t = l.trim_start();
                t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") || t.starts_with("//") || t.starts_with("/*") || t.trim().is_empty()
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else { String::new() };

    // Determine output dir
    let ext = args.ext.clone().unwrap_or_else(|| args.file.extension().and_then(|e| e.to_str()).unwrap_or("txt").to_string());
    let stem = args.file.file_stem().and_then(|s| s.to_str()).unwrap_or("split").to_string();
    let out_dir = args.output_dir.clone().unwrap_or_else(|| {
        args.file.parent().map(|p| p.join(&stem)).unwrap_or_else(|| PathBuf::from(&stem))
    });

    // For each symbol, extract body
    let mut planned: Vec<(String, String)> = Vec::new(); // (file-name, content)
    let mut exports_for_hub: Vec<(String, String)> = Vec::new(); // (symbol-name, file-name-no-ext)

    for sym in &symbols {
        let (start, end, body) = match extract_body(&content, &sym.name) {
            Some(x) => x,
            None => continue,
        };
        let _ = (start, end);
        let fname = match args.naming.as_str() {
            "exact" => format!("{}.{}", sym.name, ext),
            _ => format!("{}.{}", to_kebab(&sym.name), ext),
        };
        // Reconstruct with export keyword prefix if missing (for exportable symbols)
        let final_body = if sym.exported && !body.trim_start().starts_with("export") {
            match lang {
                "ts" | "js" => format!("export {}", body.trim_start()),
                _ => body,
            }
        } else { body };

        let full = if imports_header.is_empty() {
            final_body
        } else {
            format!("{}\n\n{}", imports_header.trim_end(), final_body)
        };
        planned.push((fname.clone(), full));
        let fname_no_ext = fname.trim_end_matches(&format!(".{}", ext)).to_string();
        exports_for_hub.push((sym.name.clone(), fname_no_ext));
    }

    println!("{} {} → {} files",
        "Splitting:".cyan().bold(),
        args.file.display().to_string().yellow(),
        planned.len().to_string().green());
    for (f, _) in &planned {
        println!("  {} {}", "+".green(), out_dir.join(f).display().to_string().cyan());
    }
    if args.keep_hub {
        println!("  {} {} (hub barrel)", "→".magenta(), args.file.display().to_string().yellow());
    }

    if args.dry_run {
        println!("\n{}", "[DRY RUN — nothing written]".yellow().bold());
        return Ok(());
    }

    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }

    // Create out_dir
    std::fs::create_dir_all(&out_dir)?;
    for (fname, body) in &planned {
        let target = out_dir.join(fname);
        std::fs::write(&target, body)?;
    }

    // Hub barrel
    if args.keep_hub {
        let mut hub = String::new();
        if !imports_header.is_empty() {
            // We won't re-include imports in barrel — they belong in leaves
        }
        // Build "export { X } from './stem/x-name'" for each
        let hub_prefix = format!("./{}", stem);
        for (sym_name, fname_no_ext) in &exports_for_hub {
            match lang {
                "ts" | "js" => {
                    hub.push_str(&format!("export {{ {} }} from '{}/{}';\n", sym_name, hub_prefix, fname_no_ext));
                }
                "rs" => {
                    hub.push_str(&format!("pub use {}::{};\n", fname_no_ext.replace('-', "_"), sym_name));
                }
                "py" => {
                    hub.push_str(&format!("from .{} import {}\n", fname_no_ext.replace('-', "_"), sym_name));
                }
                _ => {}
            }
        }
        std::fs::write(&args.file, hub)?;
    } else {
        // Original file untouched — user should decide what to do with it
    }

    println!("\n{} {} files written to {}",
        "Done:".green().bold(),
        planned.len().to_string().green(),
        out_dir.display().to_string().cyan());
    Ok(())
}

fn to_kebab(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    let mut prev_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 && !prev_upper { out.push('-'); }
            out.push(c.to_ascii_lowercase());
            prev_upper = true;
        } else {
            out.push(c);
            prev_upper = false;
        }
    }
    out
}
