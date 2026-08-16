use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ReportHealthArgs {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(short = 'e', long)]
    ext: Option<String>,
}

pub fn run(args: ReportHealthArgs) -> Result<()> {
    // Invoke our own binary directly (no cmd shell) so embedded-quote mangling
    // and PATH lookup can't break it. Same binary, subprocess isolation.
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("health").arg(&args.path);
    if let Some(e) = &args.ext {
        cmd.arg("-e").arg(e);
    }
    let out = String::from_utf8_lossy(&cmd.output()?.stdout).to_string();
    let mut md = String::new();
    md.push_str("# Codebase Health Report\n\n");
    md.push_str(&format!("_Generated: {}_\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    md.push_str("```\n");
    md.push_str(&out);
    md.push_str("```\n");
    write_out(&md, args.output)
}

pub(crate) fn write_out(text: &str, out: Option<PathBuf>) -> Result<()> {
    match out {
        Some(p) => {
            std::fs::write(&p, text)?;
            eprintln!("Wrote: {}", p.display());
        }
        None => print!("{}", text),
    }
    Ok(())
}
