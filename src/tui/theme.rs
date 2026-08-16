use ratatui::style::{Color, Modifier, Style};

// Dark theme with green accents (inspired by the freebuff design language)
pub const BG: Color = Color::Black;
pub const FG: Color = Color::White;
pub const ACCENT: Color = Color::Green;
pub const ACCENT_DIM: Color = Color::Rgb(90, 160, 90);
pub const MUTED: Color = Color::Rgb(140, 140, 140);
pub const HIGHLIGHT: Color = Color::Rgb(20, 60, 20);
pub const ERROR: Color = Color::Red;
#[allow(dead_code)] // reserved for status-warning styling in a later pass
pub const WARN: Color = Color::Yellow;
pub const INFO: Color = Color::Cyan;
pub const KEY: Color = Color::Magenta;

pub fn base() -> Style { Style::default().fg(FG).bg(BG) }
pub fn dim() -> Style { Style::default().fg(MUTED).bg(BG) }
pub fn accent() -> Style { Style::default().fg(ACCENT).bg(BG).add_modifier(Modifier::BOLD) }
pub fn accent_dim() -> Style { Style::default().fg(ACCENT_DIM).bg(BG) }
pub fn selected() -> Style { Style::default().fg(ACCENT).bg(HIGHLIGHT).add_modifier(Modifier::BOLD) }
pub fn error() -> Style { Style::default().fg(ERROR).add_modifier(Modifier::BOLD) }
pub fn info() -> Style { Style::default().fg(INFO) }
pub fn key_hint() -> Style { Style::default().fg(KEY).add_modifier(Modifier::BOLD) }
