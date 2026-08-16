use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct CsvSelectArgs {
    file: PathBuf,
    /// Comma-separated columns (names or indexes)
    columns: String,
    #[arg(long)]
    no_header: bool,
    #[arg(short = 'd', long, default_value = ",")]
    delim: String,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
}

pub fn run(args: CsvSelectArgs) -> Result<()> {
    let delim = args.delim.chars().next().unwrap_or(',') as u8;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .delimiter(delim)
        .from_path(&args.file)?;
    let headers = rdr.headers()?.clone();
    let header_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
    let mut indexes: Vec<usize> = Vec::new();
    for name in args.columns.split(',') {
        let n = name.trim();
        let idx = if let Ok(i) = n.parse::<usize>() { i }
            else { header_vec.iter().position(|h| h == n).ok_or_else(|| anyhow::anyhow!("Column '{}' not found", n))? };
        indexes.push(idx);
    }

    let mut wtr: Box<dyn std::io::Write> = match &args.output {
        Some(p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(std::io::stdout()),
    };
    let mut csvw = csv::WriterBuilder::new().delimiter(delim).from_writer(&mut wtr);
    if !args.no_header {
        let hdr: Vec<&str> = indexes.iter().map(|i| headers.get(*i).unwrap_or("")).collect();
        csvw.write_record(&hdr)?;
    }
    let mut rows = 0usize;
    for record in rdr.records() {
        let rec = record?;
        let out: Vec<&str> = indexes.iter().map(|i| rec.get(*i).unwrap_or("")).collect();
        csvw.write_record(&out)?;
        rows += 1;
    }
    csvw.flush()?;
    drop(csvw);
    eprintln!("\n{} {} rows, {} columns selected",
        "Total:".dimmed(), rows.to_string().yellow(), indexes.len().to_string().yellow());
    Ok(())
}
