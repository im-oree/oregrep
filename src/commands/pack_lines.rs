use anyhow::Result;
use clap::Args;
use colored::*;

use crate::engine::encoding::read_file_smart;
use crate::engine::edit::parse_line_range;

#[derive(Args)]
pub struct PackLinesArgs {
    /// File specs: path, path:N, path:N-M, or path:N:M
    /// Examples: src/foo.ts:80-120  src/bar.ts:1-50  src/baz.ts:200
    #[arg(required = true)]
    specs: Vec<String>,

    /// Output format: tag (default), md, plain
    #[arg(long, default_value = "tag")]
    format: String,

    /// Show line numbers in output
    #[arg(short = 'n', long)]
    numbers: bool,

    /// Show file+range label above each block (always on for tag/md)
    #[arg(long)]
    label: bool,
}

struct Spec {
    file: String,
    range: Option<String>,
}

fn parse_spec(s: &str) -> Spec {
    // Split on last colon that looks like a range (digit after it)
    // Handle Windows paths: C:\foo\bar.ts:80-120
    // Strategy: find the LAST colon where the remainder matches \d
    let bytes = s.as_bytes();
    let mut colon_pos: Option<usize> = None;
    for i in (0..s.len()).rev() {
        if bytes[i] == b':' {
            let after = &s[i + 1..];
            // Valid range: starts with digit, or N-M, or N:M
            if after.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                colon_pos = Some(i);
                break;
            }
        }
    }
    match colon_pos {
        Some(pos) => Spec {
            file: s[..pos].to_string(),
            range: Some(s[pos + 1..].to_string()),
        },
        None => Spec {
            file: s.to_string(),
            range: None,
        },
    }
}

pub fn run(args: PackLinesArgs) -> Result<()> {
    let fmt = args.format.to_lowercase();
    if !["tag", "md", "plain"].contains(&fmt.as_str()) {
        anyhow::bail!("Unknown format {:?}. Use: tag, md, plain", args.format);
    }

    let mut blocks: Vec<(String, String, String)> = Vec::new(); // (label, lang, content)

    for spec_str in &args.specs {
        let spec = parse_spec(spec_str);
        let path = std::path::Path::new(&spec.file);

        if !path.exists() {
            anyhow::bail!("File not found: {}", spec.file);
        }

        let content = read_file_smart(path)?;
        let all_lines: Vec<&str> = content.lines().collect();
        let total = all_lines.len();

        let (from, to, range_label) = if let Some(ref range_str) = spec.range {
            // Handle N:M as a range (parse_line_range accepts both : and -)
            let (f, t) = parse_line_range(range_str, total)?;
            (f, t, format!("{}:{}", f, t))
        } else {
            (1, total, format!("1:{}", total))
        };

        let selected: Vec<&str> = all_lines[(from - 1)..to].to_vec();

        let block_label = format!("{}:{}", spec.file, range_label);

        // Detect language from extension for code fences
        let lang = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let mut block_content = String::new();
        for (i, line) in selected.iter().enumerate() {
            if args.numbers {
                let line_num = from + i;
                block_content.push_str(&format!("{:>5} │ {}\n", line_num, line));
            } else {
                block_content.push_str(line);
                block_content.push('\n');
            }
        }

        blocks.push((block_label, lang, block_content));
    }

    // Render
    for (label, lang, content) in &blocks {
        match fmt.as_str() {
            "tag" => {
                println!("<file path=\"{}\">", label);
                print!("{}", content);
                println!("</file>");
            }
            "md" => {
                println!("### `{}`", label);
                println!("```{}", lang);
                print!("{}", content);
                println!("```");
                println!();
            }
            "plain" => {
                if args.label {
                    println!("=== {} ===", label);
                }
                print!("{}", content);
            }
            _ => unreachable!(),
        }
    }

    // Summary to stderr so it doesn't pollute stdout pipe
    let total_lines: usize = blocks.iter().map(|(_, _, c)| c.lines().count()).sum();
    eprintln!(
        "{} {} spec{}, {} lines",
        "Packed:".dimmed(),
        blocks.len().to_string().yellow(),
        if blocks.len() == 1 { "" } else { "s" },
        total_lines.to_string().yellow()
    );

    Ok(())
}
