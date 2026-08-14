use anyhow::Result;
use clap::Args;
use colored::*;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct ShowArgs {
    /// File to show (omit to read from stdin)
    file: Option<PathBuf>,

    /// Editor to open with (default: notepad)
    #[arg(short = 'e', long, default_value = "notepad")]
    editor: String,

    /// Custom filename prefix for the temp file
    #[arg(short = 'p', long, default_value = "ore")]
    prefix: String,

    /// File extension for temp file
    #[arg(short = 'x', long, default_value = "txt")]
    ext: String,

    /// Detach (don't wait)
    #[arg(short = 'd', long)]
    detached: bool,

    /// Print temp file path
    #[arg(short = 'P', long)]
    print_path: bool,
}

pub fn run(args: ShowArgs) -> Result<()> {
    let content = if let Some(f) = &args.file {
        if !f.exists() { anyhow::bail!("File not found: {}", f.display()); }
        std::fs::read(f)?
    } else {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        buf
    };

    if content.is_empty() {
        eprintln!("{} No content to show.", "!".yellow());
        return Ok(());
    }

    let clean = strip_ansi(&content);
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
    let fname = format!("{}-{}.{}", args.prefix, ts, args.ext);
    let temp_path: PathBuf = std::env::temp_dir().join(fname);
    std::fs::write(&temp_path, &clean)?;

    if args.print_path {
        println!("{}", temp_path.display());
    } else {
        eprintln!("{} {} ({} bytes)",
            "Opening:".cyan(),
            temp_path.display().to_string().yellow(),
            clean.len().to_string().dimmed());
    }

    launch_editor(&args.editor, &temp_path, args.detached)?;
    Ok(())
}

/// Launch an editor. On Windows, `code`/`nano`/etc. may be .cmd/.bat shims that
/// std::process::Command doesn't auto-resolve. We try direct spawn first, then
/// fall back to `cmd /C <editor>`.
pub fn launch_editor(editor: &str, path: &std::path::Path, _detached: bool) -> Result<()> {
    match std::process::Command::new(editor).arg(path).spawn() {
        Ok(_) => return Ok(()),
        Err(_) => {}
    }
    #[cfg(windows)]
    {
        // Try common shim suffixes explicitly
        for ext in &["cmd", "bat", "exe"] {
            let with_ext = format!("{}.{}", editor, ext);
            if std::process::Command::new(&with_ext).arg(path).spawn().is_ok() {
                return Ok(());
            }
        }
        // Last resort: shell out via cmd
        let arg = format!("{} \"{}\"", editor, path.display());
        std::process::Command::new("cmd").args(["/C", &arg]).spawn()?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        anyhow::bail!("Editor '{}' not found", editor);
    }
}

fn strip_ansi(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0x1B && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            i += 2;
            while i < bytes.len() {
                let c = bytes[i];
                i += 1;
                if (0x40..=0x7E).contains(&c) { break; }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}
