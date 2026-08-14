use anyhow::Result;
use clap::Args;
use colored::*;
use encoding_rs::Encoding;
use std::fs;
use std::path::PathBuf;

use crate::engine::backup::create_backup;

#[derive(Args)]
pub struct EncodingArgs {
    /// File to inspect or convert
    file: PathBuf,

    /// Convert to this encoding (utf-8, utf-16le, utf-16be, windows-1252, etc.)
    #[arg(short = 't', long)]
    to: Option<String>,

    /// Skip backup on conversion
    #[arg(long)]
    no_backup: bool,

    /// Backup label
    #[arg(short = 'l', long)]
    label: Option<String>,

    /// Add BOM after conversion (only meaningful for utf-8, utf-16)
    #[arg(long)]
    bom: bool,

    /// Strip BOM after conversion
    #[arg(long)]
    strip_bom: bool,
}

pub fn run(args: EncodingArgs) -> Result<()> {
    if !args.file.exists() {
        anyhow::bail!("File not found: {}", args.file.display());
    }

    let bytes = fs::read(&args.file)?;

    // Detection
    let (bom_name, bom_len) = detect_bom(&bytes);
    let (guessed, confidence) = if bom_len > 0 {
        (bom_name.to_string(), "BOM".to_string())
    } else {
        let mut detector = chardetng::EncodingDetector::new();
        detector.feed(&bytes, true);
        let enc = detector.guess(None, true);
        (enc.name().to_string(), "detected".to_string())
    };

    if args.to.is_none() {
        // Just report
        println!("{} {}",
            "File:".dimmed(),
            args.file.display().to_string().cyan()
        );
        println!("{} {} bytes", "Size:".dimmed(), bytes.len().to_string().yellow());
        println!("{} {} ({})",
            "Encoding:".dimmed(),
            guessed.green(),
            confidence.dimmed()
        );
        if bom_len > 0 {
            println!("{} {} ({} bytes)", "BOM:".dimmed(), "present".green(), bom_len);
        } else {
            println!("{} {}", "BOM:".dimmed(), "absent".yellow());
        }
        // Simple line ending scan
        let has_crlf = bytes.windows(2).any(|w| w == b"\r\n");
        let has_lf_only = !has_crlf && bytes.contains(&b'\n');
        let nl = if has_crlf { "CRLF" } else if has_lf_only { "LF" } else { "none" };
        println!("{} {}", "Newlines:".dimmed(), nl.green());
        return Ok(());
    }

    // Conversion
    let target_name = args.to.as_ref().unwrap().to_lowercase();
    let target_enc: &'static Encoding = Encoding::for_label(target_name.as_bytes())
        .ok_or_else(|| anyhow::anyhow!("Unknown encoding: {}", target_name))?;

    // Decode source using detected encoding
    let source_enc: &'static Encoding = Encoding::for_label(guessed.as_bytes()).unwrap_or(encoding_rs::UTF_8);
    let (decoded, _, _) = source_enc.decode(&bytes[bom_len..]);

    // Encode to target. UTF-16LE/BE are decoder-only in encoding_rs
    // (their encoder silently returns a UTF-8 encoder), so encode those
    // manually via encode_utf16.
    let encoded: Vec<u8> = match target_enc.name() {
        "UTF-16LE" => encode_utf16(&decoded, true),
        "UTF-16BE" => encode_utf16(&decoded, false),
        _ => target_enc.encode(&decoded).0.into_owned(),
    };
    let mut out_bytes: Vec<u8> = Vec::with_capacity(encoded.len() + 3);

    if args.bom && !args.strip_bom {
        match target_enc.name() {
            "UTF-8" => out_bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]),
            "UTF-16LE" => out_bytes.extend_from_slice(&[0xFF, 0xFE]),
            "UTF-16BE" => out_bytes.extend_from_slice(&[0xFE, 0xFF]),
            _ => {}
        }
    }
    out_bytes.extend_from_slice(&encoded);

    if !args.no_backup {
        let label = args
            .label
            .clone()
            .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d_%H%M%S").to_string());
        let backup_path = create_backup(&args.file, &label)?;
        println!("{} {}",
            "Backup:".dimmed(),
            backup_path.display().to_string().dimmed()
        );
    }

    fs::write(&args.file, &out_bytes)?;
    println!("{} {} -> {}",
        "Converted:".green().bold(),
        guessed.yellow(),
        target_enc.name().green()
    );

    Ok(())
}

/// Encode a &str as UTF-16 code units in either byte order.
fn encode_utf16(s: &str, little_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for unit in s.encode_utf16() {
        let bytes = if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        };
        out.extend_from_slice(&bytes);
    }
    out
}

fn detect_bom(bytes: &[u8]) -> (&'static str, usize) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        ("UTF-8", 3)
    } else if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        ("UTF-32LE", 4)
    } else if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        ("UTF-32BE", 4)
    } else if bytes.starts_with(&[0xFF, 0xFE]) {
        ("UTF-16LE", 2)
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        ("UTF-16BE", 2)
    } else {
        ("", 0)
    }
}
