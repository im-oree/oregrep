use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct CsvQueryArgs {
    file: PathBuf,
    /// Column name to show
    column: String,
    /// Filter: column=value (exact match). Repeatable.
    #[arg(short = 'w', long = "where")]
    filters: Vec<String>,
    /// No header row (columns are 0,1,2,...)
    #[arg(long)]
    no_header: bool,
    /// Delimiter (default comma)
    #[arg(short = 'd', long, default_value = ",")]
    delim: String,
}

pub fn run(args: CsvQueryArgs) -> Result<()> {
    let delim = args.delim.chars().next().unwrap_or(',') as u8;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .delimiter(delim)
        .from_path(&args.file)?;
    let headers: Vec<String> = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    let col_idx = if let Ok(i) = args.column.parse::<usize>() { i }
        else {
            headers.iter().position(|h| h == &args.column)
                .ok_or_else(|| anyhow::anyhow!("Column '{}' not found. Headers: {:?}", args.column, headers))?
        };
    // Parse filters
    let mut where_pairs: Vec<(usize, String)> = Vec::new();
    for f in &args.filters {
        let (k, v) = f.split_once('=').ok_or_else(|| anyhow::anyhow!("Bad --where (expected col=val): {}", f))?;
        let idx = if let Ok(i) = k.parse::<usize>() { i }
            else { headers.iter().position(|h| h == k).ok_or_else(|| anyhow::anyhow!("Filter column '{}' not found", k))? };
        where_pairs.push((idx, v.to_string()));
    }

    let mut count = 0usize;
    for result in rdr.records() {
        let record = result?;
        let mut keep = true;
        for (idx, val) in &where_pairs {
            if record.get(*idx).map(|v| v != val).unwrap_or(true) { keep = false; break; }
        }
        if !keep { continue; }
        if let Some(v) = record.get(col_idx) {
            println!("{}", v);
            count += 1;
        }
    }
    eprintln!("\n{} {} rows", "Total:".dimmed(), count.to_string().yellow());
    Ok(())
}
