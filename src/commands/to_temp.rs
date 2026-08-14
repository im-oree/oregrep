use anyhow::Result;
use clap::Args;
use std::io::Read;
use std::path::PathBuf;

#[derive(Args)]
pub struct ToTempArgs {
    /// File extension for the temp file
    #[arg(short = 'x', long, default_value = "txt")]
    ext: String,

    /// Custom filename prefix
    #[arg(short = 'p', long, default_value = "ore")]
    prefix: String,

    /// Strip ANSI color codes before writing
    #[arg(short = 's', long, default_value = "true")]
    strip: bool,
}

pub fn run(args: ToTempArgs) -> Result<()> {
    let mut content = Vec::new();
    std::io::stdin().read_to_end(&mut content)?;

    let payload = if args.strip { strip_ansi(&content) } else { content };

    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
    let fname = format!("{}-{}.{}", args.prefix, ts, args.ext);
    let temp_path: PathBuf = std::env::temp_dir().join(fname);

    std::fs::write(&temp_path, &payload)?;
    println!("{}", temp_path.display());
    Ok(())
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
