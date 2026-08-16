use anyhow::Result;
use base64::Engine as _;
use clap::Args;
use colored::*;

#[derive(Args)]
pub struct NotifyArgs {
    message: String,

    #[arg(short = 't', long, default_value = "ore")]
    title: String,

    #[arg(short = 'e', long)]
    echo: bool,
}

pub fn run(args: NotifyArgs) -> Result<()> {
    #[cfg(windows)]
    {
        // Build a proper PowerShell script and pass via -EncodedCommand
        // to avoid all the cmd /C quoting/echo issues.
        let ps_script = format!(
            r#"$ErrorActionPreference='SilentlyContinue';Add-Type -AssemblyName System.Windows.Forms;$n=New-Object System.Windows.Forms.NotifyIcon;$n.Icon=[System.Drawing.SystemIcons]::Information;$n.BalloonTipTitle=[string]'{}';$n.BalloonTipText=[string]'{}';$n.Visible=$true;$n.ShowBalloonTip(5000);Start-Sleep -Seconds 6;$n.Dispose()"#,
            args.title.replace('\'', "''"),
            args.message.replace('\'', "''"),
        );
        // PowerShell -EncodedCommand expects UTF-16 LE base64
        let utf16: Vec<u8> = ps_script.encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&utf16);

        let _ = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-EncodedCommand", &b64])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!("display notification \"{}\" with title \"{}\"",
            args.message.replace('"', "\\\""),
            args.title.replace('"', "\\\""));
        let _ = std::process::Command::new("osascript").args(["-e", &script])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("notify-send").args([&args.title, &args.message])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
    if args.echo {
        println!("{} {}", args.title.cyan().bold(), args.message);
    }
    eprintln!("{} {}", "Notified:".green(), args.message.dimmed());
    Ok(())
}
