use anyhow::Result;
use clap::Args;
use colored::*;
use std::path::PathBuf;

#[derive(Args)]
pub struct MagicArgs {
    /// File(s) to identify
    files: Vec<PathBuf>,

    /// Quiet (just type name)
    #[arg(short = 'q', long)]
    quiet: bool,
}

pub fn run(args: MagicArgs) -> Result<()> {
    if args.files.is_empty() { anyhow::bail!("Provide at least one file"); }
    for f in &args.files {
        if !f.exists() { eprintln!("  {} {}", "MISSING".red(), f.display()); continue; }
        let head = read_head(f, 512)?;
        let id = identify(&head, f);
        if args.quiet {
            println!("{}", id);
        } else {
            println!("{}  {}", id.green().bold(), f.display().to_string().cyan());
        }
    }
    Ok(())
}

fn read_head(path: &std::path::Path, n: usize) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut buf = vec![0u8; n];
    let read = f.read(&mut buf)?;
    buf.truncate(read);
    Ok(buf)
}

fn identify(b: &[u8], path: &std::path::Path) -> String {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") { return "PNG image".to_string(); }
    if b.starts_with(b"\xff\xd8\xff") { return "JPEG image".to_string(); }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") { return "GIF image".to_string(); }
    if b.len() >= 12 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" { return "WebP image".to_string(); }
    if b.starts_with(b"%PDF-") { return "PDF document".to_string(); }
    if b.starts_with(b"PK\x03\x04") { return "ZIP archive (or docx/xlsx/jar)".to_string(); }
    if b.starts_with(b"\x1f\x8b") { return "gzip archive".to_string(); }
    if b.starts_with(b"7z\xbc\xaf\x27\x1c") { return "7-Zip archive".to_string(); }
    if b.starts_with(b"Rar!\x1a\x07") { return "RAR archive".to_string(); }
    if b.starts_with(b"MZ") { return "PE executable (EXE/DLL)".to_string(); }
    if b.starts_with(b"\x7fELF") { return "ELF binary".to_string(); }
    if b.starts_with(b"#!") {
        let first = b.iter().position(|&c| c == b'\n').unwrap_or(b.len().min(64));
        return format!("Script ({})", String::from_utf8_lossy(&b[..first]).trim());
    }
    if b.starts_with(b"\xef\xbb\xbf") { return "UTF-8 text (with BOM)".to_string(); }
    if b.starts_with(b"\xff\xfe") { return "UTF-16 LE text".to_string(); }
    if b.starts_with(b"\xfe\xff") { return "UTF-16 BE text".to_string(); }
    if b.starts_with(b"ID3") { return "MP3 audio (ID3v2)".to_string(); }
    if b.len() >= 12 && &b[4..8] == b"ftyp" { return "MP4 / MOV video".to_string(); }
    if b.starts_with(b"OggS") { return "Ogg container".to_string(); }
    if b.starts_with(b"fLaC") { return "FLAC audio".to_string(); }
    if b.starts_with(b"BM") { return "BMP image".to_string(); }
    if b.starts_with(b"II*\x00") || b.starts_with(b"MM\x00*") { return "TIFF image".to_string(); }
    if b.starts_with(b"\x00\x00\x01\x00") { return "ICO icon".to_string(); }
    // Sniff for text
    let text_like = b.iter().take(512).all(|c| c.is_ascii_graphic() || matches!(*c, b' ' | b'\t' | b'\r' | b'\n'));
    if text_like && !b.is_empty() {
        // Match by extension for hint
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        return match ext.as_str() {
            "json" => "JSON text".to_string(),
            "xml" => "XML text".to_string(),
            "html" | "htm" => "HTML text".to_string(),
            "md" => "Markdown text".to_string(),
            "" => "ASCII text".to_string(),
            _ => format!("Text ({})", ext),
        };
    }
    "unknown / binary".to_string()
}
