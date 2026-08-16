use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::BTreeMap;

use crate::engine::web::WebSession;

#[derive(Args)]
pub struct WebScrapeArgs {
    url: String,

    /// Field spec: name=selector. Repeat.
    /// e.g. -f "title=h1" -f "price=.price"
    #[arg(short = 'f', long = "field", required = true, num_args = 1..)]
    fields: Vec<String>,

    /// If set, treat this selector as a repeating container; extract fields relative to each match.
    #[arg(short = 'r', long)]
    repeat: Option<String>,

    /// Extract this attribute (default: innerText). Applies to all fields.
    #[arg(short = 'a', long)]
    attr: Option<String>,

    #[arg(short = 't', long, default_value = "30")]
    timeout: u64,

    #[arg(short = 'w', long)]
    wait_selector: Option<String>,

    #[arg(short = 'o', long)]
    output: Option<std::path::PathBuf>,

    /// Output format: json (default) or csv
    #[arg(short = 'F', long, default_value = "json")]
    format: String,
}

pub fn run(args: WebScrapeArgs) -> Result<()> {
    let field_pairs: Vec<(String, String)> = args.fields.iter()
        .map(|f| {
            let (name, sel) = f.split_once('=').ok_or_else(|| anyhow::anyhow!("Bad field (want name=selector): {}", f))?;
            Ok((name.trim().to_string(), sel.trim().to_string()))
        })
        .collect::<Result<Vec<_>>>()?;

    let session = WebSession::launch(true, None)?;
    let tab = session.open(&args.url, args.wait_selector.as_deref(), args.timeout)?;

    // Build a JS snippet that returns JSON
    let attr_expr = match &args.attr {
        Some(a) => format!("e.getAttribute({}) || ''", json_str(a)),
        None => "(e.innerText || '').trim()".to_string(),
    };

    let js = if let Some(repeat_sel) = &args.repeat {
        let fields_json: String = field_pairs.iter()
            .map(|(k, sel)| format!("{}: (function(){{const e = root.querySelector({}); return e ? {} : null;}})()", json_str(k), json_str(sel), attr_expr))
            .collect::<Vec<_>>().join(", ");
        format!(
            "JSON.stringify(Array.from(document.querySelectorAll({})).map(root => ({{ {} }})))",
            json_str(repeat_sel), fields_json
        )
    } else {
        let fields_json: String = field_pairs.iter()
            .map(|(k, sel)| format!("{}: (function(){{const e = document.querySelector({}); return e ? {} : null;}})()", json_str(k), json_str(sel), attr_expr))
            .collect::<Vec<_>>().join(", ");
        format!("JSON.stringify([{{ {} }}])", fields_json)
    };

    let result = tab.evaluate(&js, true)?;
    let raw = result.value.and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or("[]".to_string());
    let parsed: Vec<BTreeMap<String, serde_json::Value>> = serde_json::from_str(&raw).unwrap_or_default();

    let output_text = match args.format.to_lowercase().as_str() {
        "csv" => to_csv(&parsed, &field_pairs.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>()),
        _ => serde_json::to_string_pretty(&parsed)?,
    };

    match args.output {
        Some(p) => {
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&p, &output_text)?;
            eprintln!("{} {}  ({} rows)", "Wrote:".green().bold(), p.display().to_string().cyan(), parsed.len().to_string().yellow());
        }
        None => println!("{}", output_text),
    }
    Ok(())
}

fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s.replace('"', "\\\"")))
}

fn to_csv(rows: &[BTreeMap<String, serde_json::Value>], headers: &[String]) -> String {
    let mut out = String::new();
    out.push_str(&headers.join(","));
    out.push('\n');
    for row in rows {
        let values: Vec<String> = headers.iter().map(|h| {
            let raw = row.get(h).map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            }).unwrap_or_default();
            csv_escape(&raw)
        }).collect();
        out.push_str(&values.join(","));
        out.push('\n');
    }
    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else { s.to_string() }
}
