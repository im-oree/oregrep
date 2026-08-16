use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::analysis::{build_graph, short_path};

#[derive(Args)]
pub struct DigestArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'e', long)]
    ext: Option<String>,
    #[arg(short = 'x', long)]
    exclude: Option<String>,
    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    /// Include imports section per file
    #[arg(long, default_value = "true")]
    with_imports: bool,
    /// Include tree overview at the top
    #[arg(long, default_value = "true")]
    with_tree: bool,
    /// Include per-file size/lines
    #[arg(long, default_value = "true")]
    with_stats: bool,
    /// Cap: skip files with more than N exports (usually barrel files)
    #[arg(long, default_value = "0")]
    max_exports: usize,
    /// Only include files matching this substring
    #[arg(long)]
    only: Option<String>,
}

pub fn run(args: DigestArgs) -> Result<()> {
    let g = build_graph(&args.path, args.ext.as_deref(), args.exclude.as_deref())?;
    let mut files: Vec<&PathBuf> = g.symbols.keys().collect();
    if let Some(only) = &args.only {
        files.retain(|p| short_path(&args.path, p).contains(only));
    }
    files.sort();

    let mut out = String::new();
    out.push_str(&format!("# Codebase Digest: {}\n\n", args.path.display()));
    out.push_str(&format!("_Generated {} — {} files, {} total symbols_\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M"),
        files.len(),
        g.symbols.values().map(|v| v.len()).sum::<usize>()));

    if args.with_tree {
        out.push_str("## Structure\n\n```\n");
        for f in &files {
            out.push_str(&format!("{}\n", short_path(&args.path, f)));
        }
        out.push_str("```\n\n");
    }

    out.push_str("## Files\n\n");
    for f in &files {
        let syms = g.symbols.get(*f).cloned().unwrap_or_default();
        let exports: Vec<_> = syms.iter().filter(|s| s.exported).collect();
        if args.max_exports > 0 && exports.len() > args.max_exports { continue; }
        if exports.is_empty() { continue; }

        out.push_str(&format!("### `{}`\n\n", short_path(&args.path, f)));

        if args.with_stats {
            if let Ok(meta) = std::fs::metadata(f) {
                let content = std::fs::read_to_string(f).unwrap_or_default();
                out.push_str(&format!("_{} lines, {} bytes_\n\n",
                    content.lines().count(), meta.len()));
            }
        }

        // Group exports by kind
        let mut by_kind: std::collections::BTreeMap<String, Vec<&crate::engine::symbols::Symbol>> = std::collections::BTreeMap::new();
        for s in &exports {
            by_kind.entry(format!("{:?}", s.kind)).or_default().push(*s);
        }
        for (kind, list) in &by_kind {
            out.push_str(&format!("**{}**: ", kind_pretty(kind)));
            let names: Vec<String> = list.iter().map(|s| format!("`{}`", s.name)).collect();
            out.push_str(&names.join(", "));
            out.push_str("\n\n");
        }

        if args.with_imports {
            if let Ok(content) = crate::engine::encoding::read_file_smart(f) {
                let imps = crate::engine::symbols::extract_imports(&content, f);
                let sources: Vec<&str> = imps.iter()
                    .map(|i| i.source.as_str())
                    .filter(|s| !s.starts_with('.') && !s.starts_with('/'))
                    .collect();
                if !sources.is_empty() {
                    out.push_str("_imports:_ ");
                    let dedup: std::collections::BTreeSet<&&str> = sources.iter().collect();
                    let v: Vec<String> = dedup.iter().map(|s| format!("`{}`", s)).collect();
                    out.push_str(&v.join(", "));
                    out.push_str("\n\n");
                }
            }
        }
    }

    match args.output {
        Some(p) => {
            std::fs::write(&p, &out)?;
            println!("{} {}  ({} bytes)", "Wrote:".green().bold(), p.display().to_string().cyan(), out.len().to_string().yellow());
        }
        None => print!("{}", out),
    }
    Ok(())
}

fn kind_pretty(k: &str) -> &str {
    match k {
        "Function" => "functions",
        "Class" => "classes",
        "Interface" => "interfaces",
        "Type" => "types",
        "Enum" => "enums",
        "Const" => "constants",
        "Struct" => "structs",
        "Trait" => "traits",
        "Impl" => "impls",
        "Module" => "modules",
        "Hook" => "hooks",
        "Component" => "components",
        _ => k,
    }
}
