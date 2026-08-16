use anyhow::Result;
use clap::Args;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

use crate::tui::{app::App, events, ui};

#[derive(Args)]
pub struct TuiArgs {
    /// Root path (default: focus setting, or current dir)
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

pub fn run(args: TuiArgs) -> Result<()> {
    let root = args.path.unwrap_or_else(crate::tui::app::detect_root);
    if !root.exists() { anyhow::bail!("Path not found: {}", root.display()); }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(root)?;

    let result: Result<()> = (|| {
        loop {
            terminal.draw(|f| ui::draw(f, &app))?;
            events::handle(&mut app)?;
            if app.quit { break; }
        }
        Ok(())
    })();

    // Cleanup no matter what
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).ok();
    terminal.show_cursor().ok();

    result
}
