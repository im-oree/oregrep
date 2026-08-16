use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct CsvFilterArgs {
    file: PathBuf,
    /// column=value (exact). Repeatable, all must match.
    #[arg(short = 'w', long = "where")]
    filters: Vec<String>,
    #[arg(long)]
    no_header: bool,
    #[arg(short = 'd', long, default_value = ",")]
    delim: String,
    /// Output file (default: stdout)
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: CsvFilterArgs) -> Result<()> {
    let delim = args.delim.chars().next().unwrap_or(',') as u8;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .delimiter(delim)
        .from_path(&args.file)?;
    let headers = rdr.headers()?.clone();
    let header_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    let mut where_pairs: Vec<(usize, String)> = Vec::new();
    for f in &args.filters {
        let (k, v) = f.split_once('=').ok_or_else(|| anyhow::anyhow!("Bad --where (expected col=val): {}", f))?;
        let idx = if let Ok(i) = k.parse::<usize>() { i }
            else { header_vec.iter().position(|h| h == k).ok_or_else(|| anyhow::anyhow!("Filter column '{}' not found", k))? };
        where_pairs.push((idx, v.to_string()));
    }

    let mut wtr: Box<dyn std::io::Write> = match &args.output {
        Some(p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(std::io::stdout()),
    };
    let mut csvw = csv::WriterBuilder::new().delimiter(delim).from_writer(&mut wtr);
    if !args.no_header { csvw.write_record(&headers)?; }
    let mut kept = 0usize;
    for record in rdr.records() {
        let rec = record?;
        let mut keep = true;
        for (idx, val) in &where_pairs {
            if rec.get(*idx).map(|v| v != val).unwrap_or(true) { keep = false; break; }
        }
        if keep {
            csvw.write_record(&rec)?;
            kept += 1;
        }
    }
    csvw.flush()?;
    drop(csvw);
    if args.output.is_some() {
        eprintln!("{} {} rows kept", "OK:".green().bold(), kept.to_string().yellow());
    } else {
        eprintln!("\n{} {} rows kept", "Total:".dimmed(), kept.to_string().yellow());
    }
    Ok(())
}
