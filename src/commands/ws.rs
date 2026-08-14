use anyhow::Result;
use clap::Args;
use colored::*;
use tungstenite::{connect, Message};
use url::Url;

#[derive(Args)]
pub struct WsArgs {
    /// WebSocket URL (ws:// or wss://)
    url: String,

    /// Message to send after connecting
    #[arg(short = 'm', long)]
    message: Option<String>,

    /// Send N messages then close
    #[arg(short = 'n', long)]
    count: Option<usize>,

    /// Read this many messages then exit
    #[arg(short = 'r', long)]
    read: Option<usize>,

    /// Read forever (Ctrl+C to stop)
    #[arg(long)]
    listen: bool,
}

pub fn run(args: WsArgs) -> Result<()> {
    let url = Url::parse(&args.url)?;
    println!("{} {}", "Connecting to".cyan(), args.url.yellow());
    let (mut socket, response) = connect(url.as_str())?;
    println!("{} HTTP {}", "Connected".green().bold(), response.status().as_u16().to_string().green());

    if let Some(msg) = &args.message {
        let n = args.count.unwrap_or(1);
        for i in 0..n {
            let content = if n > 1 { format!("{} #{}", msg, i + 1) } else { msg.clone() };
            socket.send(Message::Text(content.clone()))?;
            println!("  {} {}", "→".cyan(), content);
        }
    }

    let read_count = args.read.unwrap_or(if args.listen { usize::MAX } else if args.message.is_some() { 1 } else { 0 });
    for i in 0..read_count {
        let msg = socket.read()?;
        match msg {
            Message::Text(t) => println!("  {} {}", "←".green(), t),
            Message::Binary(b) => println!("  {} {} bytes binary", "←".green(), b.len()),
            Message::Ping(_) => println!("  {} PING", "←".dimmed()),
            Message::Pong(_) => println!("  {} PONG", "←".dimmed()),
            Message::Close(f) => {
                println!("  {} CLOSE {}", "←".dimmed(), f.map(|c| c.reason.to_string()).unwrap_or_default().dimmed());
                break;
            }
            Message::Frame(_) => {}
        }
        if !args.listen && i + 1 >= read_count { break; }
    }
    let _ = socket.close(None);
    println!("{}", "Closed.".dimmed());
    Ok(())
}
