use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

use crate::engine::encoding::read_file_smart;
use crate::engine::formats::nav_get;

#[derive(Args)]
pub struct JsonKeysArgs {
    file: PathBuf,
    /// Path to the object whose keys to list (default: root)
    #[arg(default_value = "")]
    path: String,
    /// Include type of each value
    #[arg(short = 't', long)]
    types: bool,
    /// Recursive: dump full key tree with dot-notation
    #[arg(short = 'r', long)]
    recursive: bool,
}

pub fn run(args: JsonKeysArgs) -> Result<()> {
    let content = read_file_smart(&args.file)?;
    let value: serde_json::Value = serde_json::from_str(&content)?;
    let target = nav_get(&value, &args.path).ok_or_else(|| anyhow::anyhow!("Path not found: {}", args.path))?;
    if args.recursive {
        let mut out: Vec<String> = Vec::new();
        collect_keys(target, "", &mut out, args.types);
        for k in out { println!("{}", k); }
        return Ok(());
    }
    match target {
        serde_json::Value::Object(m) => {
            for (k, v) in m {
                if args.types {
                    println!("{}  {}", k.cyan(), type_name(v).dimmed());
                } else {
                    println!("{}", k.cyan());
                }
            }
            eprintln!("\n{} {} keys", "Total:".dimmed(), m.len().to_string().yellow());
        }
        serde_json::Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                if args.types {
                    println!("[{}]  {}", i.to_string().cyan(), type_name(v).dimmed());
                } else {
                    println!("[{}]", i.to_string().cyan());
                }
            }
            eprintln!("\n{} {} items", "Total:".dimmed(), a.len().to_string().yellow());
        }
        _ => println!("{}: {}", type_name(target).cyan(), target),
    }
    Ok(())
}

fn type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

fn collect_keys(v: &serde_json::Value, prefix: &str, out: &mut Vec<String>, with_types: bool) {
    match v {
        serde_json::Value::Object(m) => {
            for (k, val) in m {
                let path = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                if with_types { out.push(format!("{}  ({})", path, type_name(val))); } else { out.push(path.clone()); }
                collect_keys(val, &path, out, with_types);
            }
        }
        serde_json::Value::Array(a) => {
            for (i, val) in a.iter().enumerate() {
                let path = format!("{}[{}]", prefix, i);
                if with_types { out.push(format!("{}  ({})", path, type_name(val))); } else { out.push(path.clone()); }
                collect_keys(val, &path, out, with_types);
            }
        }
        _ => {}
    }
}
