use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct BinStatsArgs {
    file: PathBuf,

    /// Show byte-frequency histogram (top 16)
    #[arg(short = 'H', long)]
    histogram: bool,
}

pub fn run(args: BinStatsArgs) -> Result<()> {
    if !args.file.exists() { anyhow::bail!("File not found: {}", args.file.display()); }
    let data = std::fs::read(&args.file)?;
    let n = data.len() as f64;

    let mut freq = [0u64; 256];
    let mut printable = 0u64;
    let mut zeros = 0u64;
    let mut high = 0u64;
    for b in &data {
        freq[*b as usize] += 1;
        if *b == 0 { zeros += 1; }
        if *b >= 0x80 { high += 1; }
        if (*b >= 0x20 && *b < 0x7f) || matches!(*b, b'\t' | b'\r' | b'\n') { printable += 1; }
    }
    let entropy: f64 = freq.iter().filter(|&&c| c > 0).map(|&c| {
        let p = c as f64 / n;
        -p * p.log2()
    }).sum();

    println!("{} {}", "File:".dimmed(), args.file.display().to_string().cyan());
    println!("{} {} bytes", "Size:".dimmed(), data.len().to_string().yellow());
    println!("{} {:.3} / 8.0  {}", "Entropy:".dimmed(), entropy,
        if entropy > 7.5 { "(likely compressed / encrypted)".red().to_string() }
        else if entropy < 4.0 { "(low variance, likely text/simple)".green().to_string() }
        else { "".normal().to_string() }
    );
    println!("{} {} ({:.1}%)", "Printable:".dimmed(), printable.to_string().yellow(), 100.0 * printable as f64 / n);
    println!("{} {} ({:.1}%)", "Zeros:".dimmed(), zeros.to_string().yellow(), 100.0 * zeros as f64 / n);
    println!("{} {} ({:.1}%)", "High bit:".dimmed(), high.to_string().yellow(), 100.0 * high as f64 / n);

    if args.histogram {
        let mut idx: Vec<usize> = (0..256).collect();
        idx.sort_by(|a, b| freq[*b].cmp(&freq[*a]));
        println!("\n{}", "Top 16 bytes:".bold());
        for i in idx.iter().take(16) {
            let c = *i as u8;
            let ch = if c.is_ascii_graphic() { format!("'{}'", c as char) } else { "".to_string() };
            let bar_len = ((freq[*i] as f64 / freq[idx[0]] as f64) * 40.0) as usize;
            let bar: String = "█".repeat(bar_len);
            println!("  {:#04x} {:<4}  {:>10}  {}", *i, ch.dimmed(), freq[*i].to_string().yellow(), bar.cyan());
        }
    }
    Ok(())
}
