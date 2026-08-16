use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;
use crate::engine::symbols::{extract_body, language_of};

#[derive(Args)]
pub struct ExtractFnArgs {
    /// Source file
    file: PathBuf,

    /// Symbol name to extract
    symbol: String,

    /// Output file (new file where the symbol will live)
    #[arg(short = 'o', long, required = true)]
    output: PathBuf,

    /// Add re-export from source to output ("export { foo } from './output'")
    #[arg(short = 'r', long, default_value = "true")]
    reexport: bool,

    /// Include imports from source file in the new output
    #[arg(short = 'i', long, default_value = "true")]
    carry_imports: bool,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
    #[arg(long)]
    dry_run: bool,
}

pub fn run(args: ExtractFnArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let content = read_file_smart(&args.file)?;
    let (start, end, body) = extract_body(&content, &args.symbol)
        .ok_or_else(|| anyhow::anyhow!("Symbol '{}' not found in {}", args.symbol, args.file.display()))?;
    let lang = language_of(&args.file);

    // Build imports header
    let imports_header = if args.carry_imports {
        content.lines().take_while(|l| {
            let t = l.trim_start();
            t.starts_with("import ") || t.starts_with("use ") || t.starts_with("from ") || t.starts_with("//") || t.starts_with("/*") || t.trim().is_empty()
        }).collect::<Vec<_>>().join("\n")
    } else { String::new() };

    // Build the new file body
    let final_body = if !body.trim_start().starts_with("export") && !body.trim_start().starts_with("pub ") {
        match lang {
            "ts" | "js" => format!("export {}", body.trim_start()),
            "rs" => format!("pub {}", body.trim_start()),
            _ => body,
        }
    } else { body.clone() };

    let new_output = if imports_header.is_empty() {
        final_body.clone()
    } else {
        format!("{}\n\n{}", imports_header.trim_end(), final_body)
    };

    // Compute the source file with symbol removed
    let mut modified_source = String::with_capacity(content.len());
    modified_source.push_str(&content[..start]);
    if args.reexport {
        // Insert a re-export line where the symbol used to be
        let out_stem = args.output.file_stem().and_then(|s| s.to_str()).unwrap_or("extracted");
        let rel = compute_relative_import_path(&args.file, &args.output).unwrap_or_else(|| format!("./{}", out_stem));
        match lang {
            "ts" | "js" => modified_source.push_str(&format!("export {{ {} }} from '{}';\n", args.symbol, rel)),
            "rs" => modified_source.push_str(&format!("pub use crate::{}::{};\n", rel.replace('-', "_"), args.symbol)),
            "py" => modified_source.push_str(&format!("from .{} import {}\n", rel.trim_start_matches("./").replace('-', "_"), args.symbol)),
            _ => {}
        }
    }
    modified_source.push_str(&content[end..]);

    println!("{} {} from {}",
        "Extracting:".cyan().bold(),
        args.symbol.yellow(),
        args.file.display().to_string().dimmed());
    println!("  {} new file: {}", "+".green(), args.output.display().to_string().cyan());
    println!("  {} source modified: {} ({} bytes → {} bytes)",
        "~".yellow(),
        args.file.display().to_string().cyan(),
        content.len().to_string().dimmed(),
        modified_source.len().to_string().dimmed());
    if args.reexport {
        println!("  {} re-export added in source", "→".magenta());
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
    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&args.output, new_output)?;
    write_atomic(&args.file, &modified_source, content.starts_with('\u{FEFF}'))?;
    println!("\n{}", "Done.".green().bold());
    Ok(())
}

fn compute_relative_import_path(from: &std::path::Path, to: &std::path::Path) -> Option<String> {
    let from_dir = from.parent()?;
    let to_stem = to.with_extension("");
    let rel = pathdiff(&from_dir.to_path_buf(), &to_stem)?;
    let mut s = rel.to_string_lossy().replace('\\', "/");
    if !s.starts_with('.') { s = format!("./{}", s); }
    Some(s)
}

fn pathdiff(from: &PathBuf, to: &PathBuf) -> Option<PathBuf> {
    let f = from.components().collect::<Vec<_>>();
    let t = to.components().collect::<Vec<_>>();
    let mut i = 0;
    while i < f.len() && i < t.len() && f[i] == t[i] { i += 1; }
    let ups = f.len() - i;
    let mut result = PathBuf::new();
    for _ in 0..ups { result.push(".."); }
    for c in &t[i..] { result.push(c); }
    Some(result)
}
