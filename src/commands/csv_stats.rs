use anyhow::Result;
use clap::Args;
use colored::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args)]
pub struct CsvStatsArgs {
    file: PathBuf,
    #[arg(long)]
    no_header: bool,
    #[arg(short = 'd', long, default_value = ",")]
    delim: String,
}

pub fn run(args: CsvStatsArgs) -> Result<()> {
    let delim = args.delim.chars().next().unwrap_or(',') as u8;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .delimiter(delim)
        .from_path(&args.file)?;
    let headers: Vec<String> = if args.no_header {
        Vec::new()
    } else {
        rdr.headers()?.iter().map(|s| s.to_string()).collect()
    };

    let mut col_data: Vec<(usize, HashMap<String, usize>, usize)> = Vec::new(); // (unique count-map, empty count)
    let mut rows = 0usize;
    for record in rdr.records() {
        let rec = record?;
        rows += 1;
        for (i, field) in rec.iter().enumerate() {
            if col_data.len() <= i { col_data.push((i, HashMap::new(), 0)); }
            let entry = &mut col_data[i];
            if field.is_empty() { entry.2 += 1; }
            else { *entry.1.entry(field.to_string()).or_insert(0) += 1; }
        }
    }

    println!("{} {}", "File:".dimmed(), args.file.display().to_string().cyan());
    println!("{} {} rows, {} columns", "Shape:".dimmed(), rows.to_string().yellow(), col_data.len().to_string().yellow());
    println!("\n{}", "Columns:".bold());
    for (i, uniques, empties) in &col_data {
        let name = headers.get(*i).cloned().unwrap_or_else(|| format!("col{}", i));
        let numeric = uniques.keys().all(|k| k.parse::<f64>().is_ok()) && !uniques.is_empty();
        let mut kind = if numeric { "numeric".green().to_string() } else { "text".cyan().to_string() };
        if uniques.len() == 1 { kind = format!("{} (single value)", kind); }
        println!("  {:<20} {:>6} unique  {:>4} empty  {}",
            name.cyan(),
            uniques.len().to_string().yellow(),
            empties.to_string().dimmed(),
            kind);
    }
    Ok(())
}
