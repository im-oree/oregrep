use anyhow::Result;
use base64::Engine as _;
use clap::Args;
use colored::*;
use std::time::{Duration, Instant};

#[derive(Args)]
pub struct TimerArgs {
    /// Duration: 30s, 5m, 1h, or plain seconds
    duration: String,

    /// Message to show when done
    #[arg(short = 'm', long, default_value = "Timer done")]
    message: String,

    /// Also fire notification when done (pass `-n` or `-n false` to disable)
    #[arg(short = 'n', long, default_value = "true", num_args = 0..=1, default_missing_value = "false")]
    notify: bool,

    /// Command to run when done
    #[arg(short = 'c', long)]
    command: Option<String>,

    /// Silent (no ticks printed)
    #[arg(short = 's', long)]
    silent: bool,
}

pub fn run(args: TimerArgs) -> Result<()> {
    let secs = parse_duration(&args.duration)?;
    println!("{} {}", "Timer:".cyan().bold(), format_secs(secs).yellow());
    let start = Instant::now();
    let total = Duration::from_secs(secs);

    while start.elapsed() < total {
        if !args.silent {
            let remaining = total.saturating_sub(start.elapsed());
            let r_secs = remaining.as_secs();
            print!("\r  {} remaining      ", format_secs(r_secs).yellow());
            use std::io::Write;
            std::io::stdout().flush().ok();
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if !args.silent {
        println!("\r  {}         ", "Done.".green().bold());
    }

    if args.notify {
        #[cfg(windows)]
        {
            let ps_script = format!(
                r#"$ErrorActionPreference='SilentlyContinue';Add-Type -AssemblyName System.Windows.Forms;$n=New-Object System.Windows.Forms.NotifyIcon;$n.Icon=[System.Drawing.SystemIcons]::Information;$n.BalloonTipTitle=[string]'ore timer';$n.BalloonTipText=[string]'{}';$n.Visible=$true;$n.ShowBalloonTip(5000);Start-Sleep -Seconds 6;$n.Dispose()"#,
                args.message.replace('\'', "''"));
            let utf16: Vec<u8> = ps_script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
            let b64 = base64::engine::general_purpose::STANDARD.encode(&utf16);
            let _ = std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-EncodedCommand", &b64])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }

    println!("{} {}", "▶".green().bold(), args.message.cyan());
    if let Some(cmd) = &args.command {
        let _ = crate::engine::proc::run_cmd(cmd, true, false);
    }
    Ok(())
}

fn parse_duration(s: &str) -> Result<u64> {
    let s = s.trim();
    let (num_part, mul) = if let Some(n) = s.strip_suffix('h') { (n, 3600u64) }
        else if let Some(n) = s.strip_suffix('m') { (n, 60u64) }
        else if let Some(n) = s.strip_suffix('s') { (n, 1u64) }
        else { (s, 1u64) };
    let n: u64 = num_part.parse().map_err(|_| anyhow::anyhow!("Bad duration: {}", s))?;
    Ok(n * mul)
}

fn format_secs(s: u64) -> String {
    if s >= 3600 { format!("{}h{}m{}s", s / 3600, (s / 60) % 60, s % 60) }
    else if s >= 60 { format!("{}m{}s", s / 60, s % 60) }
    else { format!("{}s", s) }
}
