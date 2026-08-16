use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::backup::create_backup;
use crate::engine::encoding::read_file_smart;
use crate::engine::patch::write_atomic;

#[derive(Args)]
pub struct RemoveImportArgs {
    file: PathBuf,

    /// Named import to remove
    #[arg(short = 'n', long)]
    name: Option<String>,

    /// Remove entire import line for this source
    #[arg(short = 's', long)]
    from: Option<String>,

    #[arg(long)]
    no_backup: bool,
    #[arg(short = 'l', long)]
    label: Option<String>,
}

pub fn run(args: RemoveImportArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    if args.name.is_none() && args.from.is_none() {
        anyhow::bail!("Provide --name or --from");
    }
    let content = read_file_smart(&args.file)?;

    let new_content = if let Some(from) = &args.from {
        if args.name.is_none() {
            // Delete whole line
            let re = regex::Regex::new(&format!(
                r#"(?m)^\s*import\s+[^;]+\s+from\s+['"]{}['"];?\s*\n?"#,
                regex::escape(from)
            ))?;
            re.replace_all(&content, "").to_string()
        } else {
            // Remove specific name from ALL import lines matching that source
            let name = args.name.as_ref().unwrap();
            let re = regex::Regex::new(&format!(
                r#"(?m)^(\s*import\s+)([^;]+?)(\s+from\s+['"]{}['"];?\s*)$"#,
                regex::escape(from)
            ))?;
            let mut new = String::with_capacity(content.len());
            let mut last = 0;
            for cap in re.captures_iter(&content) {
                let full = cap.get(0).unwrap();
                new.push_str(&content[last..full.start()]);
                let clauses = &cap[2];
                let updated = drop_name(clauses, name);
                if updated.trim().is_empty() {
                    // skip this line entirely (also swallow trailing newline)
                    last = full.end();
                    if last < content.len() && content.as_bytes()[last] == b'\n' { last += 1; }
                } else {
                    new.push_str(&cap[1]);
                    new.push_str(&updated);
                    new.push_str(&cap[3]);
                    last = full.end();
                }
            }
            new.push_str(&content[last..]);
            new
        }
    } else {
        // Only --name: remove from any import line
        let name = args.name.as_ref().unwrap();
        let re = regex::Regex::new(r#"(?m)^(\s*import\s+)([^;]+?)(\s+from\s+['"][^'"]+['"];?\s*)$"#)?;
        let mut new = String::with_capacity(content.len());
        let mut last = 0;
        for cap in re.captures_iter(&content) {
            let full = cap.get(0).unwrap();
            new.push_str(&content[last..full.start()]);
            let clauses = &cap[2];
            let updated = drop_name(clauses, name);
            if updated.trim().is_empty() {
                // skip this line entirely
            } else {
                new.push_str(&cap[1]);
                new.push_str(&updated);
                new.push_str(&cap[3]);
            }
            last = full.end();
        }
        new.push_str(&content[last..]);
        new
    };

    if new_content == content {
        println!("{} No changes", "!".yellow());
        return Ok(());
    }
    if !args.no_backup {
        let label = args.label.clone().unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let bak = create_backup(&args.file, &label)?;
        println!("{} {}", "Backup:".dimmed(), bak.display().to_string().dimmed());
    }
    write_atomic(&args.file, &new_content, content.starts_with('\u{FEFF}'))?;
    println!("{} import removed", "OK:".green().bold());
    Ok(())
}

fn drop_name(clauses: &str, name: &str) -> String {
    let mut default: Option<String> = None;
    let mut named: Vec<String> = Vec::new();
    if let Some(start) = clauses.find('{') {
        let before = clauses[..start].trim().trim_end_matches(',').trim();
        if !before.is_empty() { default = Some(before.to_string()); }
        let end = clauses.rfind('}').unwrap_or(clauses.len());
        for n in clauses[start + 1..end].split(',') {
            let n = n.trim();
            if n.is_empty() { continue; }
            if n == name { continue; }
            named.push(n.to_string());
        }
    } else {
        // Only default
        let d = clauses.trim();
        if d != name { default = Some(d.to_string()); }
    }

    if default.is_none() && named.is_empty() { return String::new(); }
    let mut result = String::new();
    if let Some(d) = default {
        result.push_str(&d);
        if !named.is_empty() { result.push_str(", "); }
    }
    if !named.is_empty() {
        result.push('{');
        result.push_str(&format!(" {} ", named.join(", ")));
        result.push('}');
    }
    result
}
