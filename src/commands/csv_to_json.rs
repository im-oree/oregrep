use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct CsvToJsonArgs {
    file: PathBuf,
    #[arg(long)]
    no_header: bool,
    #[arg(short = 'd', long, default_value = ",")]
    delim: String,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(short = 'c', long)]
    compact: bool,
}

pub fn run(args: CsvToJsonArgs) -> Result<()> {
    let delim = args.delim.chars().next().unwrap_or(',') as u8;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .delimiter(delim)
        .from_path(&args.file)?;
    let headers: Vec<String> = if args.no_header {
        // Peek first record for length; use 0..n as names
        Vec::new()
    } else {
        rdr.headers()?.iter().map(|s| s.to_string()).collect()
    };

    let mut rows: Vec<serde_json::Value> = Vec::new();
    for record in rdr.records() {
        let rec = record?;
        let mut obj = serde_json::Map::new();
        for (i, field) in rec.iter().enumerate() {
            let key = if args.no_header || headers.is_empty() { i.to_string() } else { headers.get(i).cloned().unwrap_or_else(|| i.to_string()) };
            obj.insert(key, serde_json::Value::String(field.to_string()));
        }
        rows.push(serde_json::Value::Object(obj));
    }
    let arr = serde_json::Value::Array(rows);
    let out = if args.compact { serde_json::to_string(&arr)? } else { serde_json::to_string_pretty(&arr)? };
    match args.output {
        Some(p) => {
            std::fs::write(&p, out)?;
            println!("{} {}", "Wrote:".green().bold(), p.display().to_string().cyan());
        }
        None => println!("{}", out),
    }
    Ok(())
}
