use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;

#[derive(Args)]
pub struct AddImportArgs {
    file: PathBuf,

    /// Named import to add, e.g. "useState"
    #[arg(short = 'n', long)]
    name: Option<String>,

    /// Default import, e.g. "React"
    #[arg(short = 'D', long)]
    default: Option<String>,

    /// Source module, e.g. "react"
    #[arg(short = 's', long)]
    from: String,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: AddImportArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    if args.name.is_none() && args.default.is_none() {
        anyhow::bail!("Provide --name or --default");
    }
    let content = read_file_smart(&args.file)?;

    // Find any existing import from this source
    let source_re = regex::Regex::new(&format!(
        r#"(?m)^\s*import\s+([^;]+?)\s+from\s+['"]{}['"];?\s*$"#,
        regex::escape(&args.from)
    ))?;

    let new_content = if let Some(mat) = source_re.find(&content) {
        // Existing import — merge in
        let line = mat.as_str();
        let before = &content[..mat.start()];
        let after = &content[mat.end()..];
        let updated = merge_into_existing(line, args.name.as_deref(), args.default.as_deref())?;
        format!("{}{}{}", before, updated, after)
    } else {
        // No existing — insert new import at top (after any leading comments)
        let stmt = build_new_import(args.name.as_deref(), args.default.as_deref(), &args.from);
        insert_import(&content, &stmt)
    };

    if new_content == content {
        println!("{} No changes (already present)", "!".yellow());
        return Ok(());
    }
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    write_atomic(&args.file, &new_content, content.starts_with('\u{FEFF}'))?;
    println!("{} added imports from '{}'", "OK:".green().bold(), args.from.yellow());
    Ok(())
}

fn merge_into_existing(line: &str, name: Option<&str>, default: Option<&str>) -> Result<String> {
    // Line looks like: import Foo from 'x'  |  import { a, b } from 'x'  |  import Foo, { a, b } from 'x'
    // Extract clauses and rebuild.
    let re = regex::Regex::new(r#"^\s*import\s+([^;]+?)\s+from\s+(['"][^'"]+['"]);?\s*$"#).unwrap();
    let cap = re.captures(line).ok_or_else(|| anyhow::anyhow!("Unparseable import line"))?;
    let clauses = cap[1].trim().to_string();
    let source = cap[2].to_string();

    let mut existing_default: Option<String> = None;
    let mut existing_named: Vec<String> = Vec::new();

    // Split by { } or comma
    if let Some(start) = clauses.find('{') {
        if start > 0 {
            let before = clauses[..start].trim().trim_end_matches(',').trim();
            if !before.is_empty() { existing_default = Some(before.to_string()); }
        }
        let end = clauses.rfind('}').unwrap_or(clauses.len());
        let inner = &clauses[start + 1..end];
        for n in inner.split(',') {
            let n = n.trim().to_string();
            if !n.is_empty() { existing_named.push(n); }
        }
    } else {
        // Only default
        existing_default = Some(clauses.clone());
    }

    if let Some(d) = default {
        if existing_default.is_none() { existing_default = Some(d.to_string()); }
    }
    if let Some(n) = name {
        if !existing_named.iter().any(|x| x == n) { existing_named.push(n.to_string()); }
    }

    existing_named.sort();
    existing_named.dedup();

    let mut result = String::from("import ");
    if let Some(d) = &existing_default {
        result.push_str(d);
        if !existing_named.is_empty() { result.push_str(", "); }
    }
    if !existing_named.is_empty() {
        result.push('{');
        result.push_str(&format!(" {} ", existing_named.join(", ")));
        result.push('}');
    }
    result.push_str(" from ");
    result.push_str(&source);
    result.push(';');
    Ok(result)
}

fn build_new_import(name: Option<&str>, default: Option<&str>, from: &str) -> String {
    let mut s = String::from("import ");
    if let Some(d) = default { s.push_str(d); if name.is_some() { s.push_str(", "); } }
    if let Some(n) = name { s.push_str(&format!("{{ {} }}", n)); }
    s.push_str(&format!(" from '{}';", from));
    s
}

fn insert_import(content: &str, stmt: &str) -> String {
    // Insert after last existing import at the top, or at very top otherwise.
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut last_import = None;
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if t.starts_with("import ") { last_import = Some(i); }
        else if !t.is_empty() && !t.starts_with("//") && !t.starts_with("/*") && last_import.is_some() { break; }
    }
    let insert_at = last_import.map(|i| i + 1).unwrap_or(0);
    lines.insert(insert_at, stmt.to_string());
    let joined = lines.join("\n");
    if content.ends_with('\n') && !joined.ends_with('\n') { format!("{}\n", joined) } else { joined }
}
