use anyhow::Result;
use clap::{Args, Subcommand};
use colored::*;
use std::io::Read;
use std::path::PathBuf;

use crate::engine::storage::{extract_vars, interpolate, parse_kv_pairs, template_path, templates_dir};

#[derive(Args)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub action: TemplateAction,
}

#[derive(Subcommand)]
pub enum TemplateAction {
    /// Save a template from a file or stdin
    Save {
        name: String,
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
        #[arg(long)]
        force: bool,
    },
    /// Load and render a template with variables
    Load {
        name: String,
        /// key=value pairs (repeatable)
        #[arg(short = 'v', long = "var", num_args = 1..)]
        vars: Vec<String>,
        /// Write output to this file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    /// List all templates
    List,
    /// Delete a template
    Rm {
        name: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Show template file path
    Path { name: String },
    /// List variables required by a template
    Vars { name: String },
    /// Test-render a template (shows what would be produced)
    Test {
        name: String,
        #[arg(short = 'v', long = "var", num_args = 1..)]
        vars: Vec<String>,
    },
}

pub fn run(args: TemplateArgs) -> Result<()> {
    match args.action {
        TemplateAction::Save { name, file, force } => {
            let path = template_path(&name)?;
            if path.exists() && !force { anyhow::bail!("Template exists: {} (use --force)", name); }
            let content = if let Some(f) = file {
                std::fs::read_to_string(f)?
            } else {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            };
            if content.trim().is_empty() { anyhow::bail!("Empty content"); }
            std::fs::write(&path, &content)?;
            let v = extract_vars(&content);
            println!("{} {} ({} bytes, {} vars)", "Saved:".green().bold(), name.cyan(), content.len().to_string().yellow(), v.len().to_string().yellow());
            if !v.is_empty() {
                println!("  vars: {}", v.join(", ").magenta());
            }
        }
        TemplateAction::Load { name, vars, output } => {
            let path = template_path(&name)?;
            if !path.exists() { anyhow::bail!("Template not found: {}", name); }
            let tmpl = std::fs::read_to_string(&path)?;
            let map = parse_kv_pairs(&vars)?;
            let rendered = interpolate(&tmpl, &map);
            match output {
                Some(p) => {
                    if let Some(parent) = p.parent() {
                        if !parent.as_os_str().is_empty() && !parent.exists() {
                            std::fs::create_dir_all(parent)?;
                        }
                    }
                    std::fs::write(&p, &rendered)?;
                    println!("{} {} ({} bytes)", "Wrote:".green().bold(), p.display().to_string().cyan(), rendered.len().to_string().yellow());
                }
                None => print!("{}", rendered),
            }
        }
        TemplateAction::List => {
            let d = templates_dir()?;
            let mut count = 0;
            for entry in std::fs::read_dir(&d)? {
                let e = entry?;
                if e.path().extension().and_then(|x| x.to_str()) == Some("tmpl") {
                    let name = e.path().file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let content = std::fs::read_to_string(e.path()).unwrap_or_default();
                    let v = extract_vars(&content);
                    println!("  {} ({} bytes, {} vars)", name.cyan(), content.len().to_string().dimmed(), v.len().to_string().yellow());
                    count += 1;
                }
            }
            if count == 0 { println!("{}", "(no templates)".dimmed()); }
        }
        TemplateAction::Rm { name, yes } => {
            let path = template_path(&name)?;
            if !path.exists() { anyhow::bail!("Template not found: {}", name); }
            if !yes {
                let ok = crate::engine::confirm::confirm(&format!("Delete template '{}'?", name), false)?;
                if !ok { println!("{}", "Aborted.".yellow()); return Ok(()); }
            }
            std::fs::remove_file(&path)?;
            println!("{} {}", "Deleted:".green(), name.cyan());
        }
        TemplateAction::Path { name } => {
            println!("{}", template_path(&name)?.display());
        }
        TemplateAction::Vars { name } => {
            let path = template_path(&name)?;
            if !path.exists() { anyhow::bail!("Template not found: {}", name); }
            let content = std::fs::read_to_string(&path)?;
            let v = extract_vars(&content);
            if v.is_empty() { println!("{}", "(no variables)".dimmed()); }
            else { for var in &v { println!("  {}", var.cyan()); } }
        }
        TemplateAction::Test { name, vars } => {
            let path = template_path(&name)?;
            if !path.exists() { anyhow::bail!("Template not found: {}", name); }
            let tmpl = std::fs::read_to_string(&path)?;
            let required = extract_vars(&tmpl);
            let map = parse_kv_pairs(&vars)?;
            let missing: Vec<&String> = required.iter().filter(|v| !map.contains_key(*v)).collect();
            if !missing.is_empty() {
                println!("{} missing vars: {}", "!".yellow(), missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ").red());
            }
            let rendered = interpolate(&tmpl, &map);
            println!("{}", "─".repeat(60).dimmed());
            print!("{}", rendered);
            println!("{}", "─".repeat(60).dimmed());
        }
    }
    Ok(())
}
