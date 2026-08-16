use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Modifier},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, Focus, Mode, View};
use crate::tui::theme;

const BANNER: &[&str] = &[
    "  ██████╗ ██████╗ ███████╗",
    " ██╔═══██╗██╔══██╗██╔════╝",
    " ██║   ██║██████╔╝█████╗  ",
    " ██║   ██║██╔══██╗██╔══╝  ",
    " ╚██████╔╝██║  ██║███████╗",
    "  ╚═════╝ ╚═╝  ╚═╝╚══════╝",
];

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Root layout: banner+status | main | commandbar
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // top: banner + top bar
            Constraint::Min(5),     // middle: tree + main
            Constraint::Length(3),  // bottom: command bar + status
        ])
        .split(size);

    draw_top(f, app, root[0]);
    draw_body(f, app, root[1]);
    draw_bottom(f, app, root[2]);

    if app.show_help_overlay {
        draw_help_overlay(f, size);
    }
}

fn draw_top(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(10)])
        .split(area);

    // Banner
    let banner_lines: Vec<Line> = BANNER.iter().map(|l| Line::from(Span::styled(*l, theme::accent()))).collect();
    let banner = Paragraph::new(banner_lines).style(theme::base());
    f.render_widget(banner, cols[0]);

    // Info panel
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Focus  ", theme::dim()),
            Span::styled(app.root.display().to_string(), theme::info()),
        ]),
        Line::from(vec![
            Span::styled("Branch ", theme::dim()),
            Span::styled(if app.git_branch.is_empty() { "(none)".to_string() } else { app.git_branch.clone() }, theme::accent()),
            Span::raw("   "),
            Span::styled("Dirty ", theme::dim()),
            Span::styled(format!("{}", app.git_dirty), if app.git_dirty > 0 { theme::error() } else { theme::accent() }),
        ]),
        Line::from(vec![
            Span::styled("View  ", theme::dim()),
            Span::styled(format!("{:?}", app.view), theme::accent_dim()),
            Span::raw("   "),
            Span::styled("Focus ", theme::dim()),
            Span::styled(format!("{:?}", app.focus), theme::accent_dim()),
            Span::raw("   "),
            Span::styled("Mode ", theme::dim()),
            Span::styled(format!("{:?}", app.mode), theme::accent_dim()),
        ]),
        Line::from(vec![
            Span::styled("Files ", theme::dim()),
            Span::styled(format!("{}", app.tree.len()), theme::info()),
            Span::raw("   "),
            Span::styled(if app.status_msg.is_empty() { "".to_string() } else { app.status_msg.clone() }, theme::accent_dim()),
        ]),
    ];
    let info = Paragraph::new(info_lines).style(theme::base()).wrap(Wrap { trim: false });
    f.render_widget(info, cols[1]);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Min(20)])
        .split(area);

    draw_tree(f, app, cols[0]);
    draw_main(f, app, cols[1]);
}

fn draw_tree(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == Focus::Tree { theme::accent() } else { theme::dim() };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(" files ", theme::accent()));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let visible = inner.height as usize;
    // Scroll to keep selected in view
    let scroll = if app.tree_selected >= app.tree_scroll + visible {
        app.tree_selected + 1 - visible
    } else if app.tree_selected < app.tree_scroll {
        app.tree_selected
    } else { app.tree_scroll };

    let items: Vec<ListItem> = app.tree.iter().enumerate().skip(scroll).take(visible).map(|(i, e)| {
        let indent = "  ".repeat(e.depth);
        let marker = if e.is_dir {
            if e.expanded { "▾ " } else { "▸ " }
        } else { "  " };
        let name = e.path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let display = format!("{}{}{}", indent, marker, name);
        let style = if i == app.tree_selected {
            theme::selected()
        } else if e.is_dir {
            theme::info()
        } else {
            theme::base()
        };
        ListItem::new(display).style(style)
    }).collect();
    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.focus == Focus::Main { theme::accent() } else { theme::dim() };
    let title = match app.view {
        View::Preview => {
            let name = app.preview_path.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "output".to_string());
            format!(" {} ", name)
        }
        View::Search => format!(" search ({} results) ", app.search_results.len()),
        View::GitStatus => " git status ".to_string(),
        View::Hex => " hex view ".to_string(),
        View::Help => " help ".to_string(),
        View::Health => " health ".to_string(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .title(Span::styled(title, theme::accent()));
    let inner = block.inner(area);
    f.render_widget(block, area);

    match app.view {
        View::Preview => draw_preview(f, app, inner),
        View::Search => draw_search_results(f, app, inner),
        View::GitStatus => draw_git_status(f, app, inner),
        View::Hex => draw_hex(f, app, inner),
        View::Help => draw_help(f, inner),
        View::Health => draw_health(f, inner),
    }
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let visible = area.height as usize;
    let lines: Vec<Line> = app.preview_lines.iter().enumerate().skip(app.preview_scroll).take(visible).map(|(i, l)| {
        Line::from(vec![
            Span::styled(format!("{:>5} ", i + 1), theme::dim()),
            Span::styled(l.clone(), theme::base()),
        ])
    }).collect();
    let p = Paragraph::new(lines).style(theme::base());
    f.render_widget(p, area);
}

fn draw_search_results(f: &mut Frame, app: &App, area: Rect) {
    let visible = area.height as usize;
    let items: Vec<ListItem> = app.search_results.iter().enumerate().skip(0).take(visible).map(|(i, (p, ln, text))| {
        let style = if i == app.search_selected { theme::selected() } else { theme::base() };
        let line = Line::from(vec![
            Span::styled(format!("{}:{}", p.display(), ln), theme::info()),
            Span::raw("  "),
            Span::styled(text.clone(), theme::dim()),
        ]);
        ListItem::new(line).style(style)
    }).collect();
    let list = List::new(items);
    f.render_widget(list, area);
}

fn draw_git_status(f: &mut Frame, _app: &App, area: Rect) {
    let files = crate::engine::git::changed_files().unwrap_or_default();
    if files.is_empty() {
        let p = Paragraph::new("Clean working tree.").style(theme::accent());
        f.render_widget(p, area);
        return;
    }
    let items: Vec<ListItem> = files.iter().take(area.height as usize).map(|(status, path)| {
        let style = if status.starts_with('?') { theme::error() }
            else if status.chars().next().unwrap_or(' ') != ' ' { theme::accent() }
            else { theme::info() };
        ListItem::new(format!("{} {}", status, path)).style(style)
    }).collect();
    let list = List::new(items);
    f.render_widget(list, area);
}

fn draw_hex(f: &mut Frame, app: &App, area: Rect) {
    let msg = match &app.preview_path {
        Some(p) => match std::fs::read(p) {
            Ok(bytes) => {
                let slice = &bytes[..bytes.len().min(4096)];
                crate::engine::hex::format_hex_dump(slice, 0, 16)
            }
            Err(_) => "(read error)".to_string(),
        },
        None => "(no file selected — open one in the tree first)".to_string(),
    };
    let lines: Vec<Line> = msg.lines().take(area.height as usize).map(|l| Line::from(Span::styled(l.to_string(), theme::base()))).collect();
    let p = Paragraph::new(lines).style(theme::base());
    f.render_widget(p, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = vec![
        Line::from(Span::styled("Ore TUI — keybindings", theme::accent())),
        Line::from(""),
        Line::from(vec![Span::styled("j/k or ↓/↑", theme::key_hint()), Span::raw("      move cursor")]),
        Line::from(vec![Span::styled("h/l or ←/→", theme::key_hint()), Span::raw("      collapse / expand dir")]),
        Line::from(vec![Span::styled("Enter", theme::key_hint()),      Span::raw("           open file / toggle dir")]),
        Line::from(vec![Span::styled("Tab", theme::key_hint()),        Span::raw("             switch panel focus")]),
        Line::from(vec![Span::styled("/", theme::key_hint()),          Span::raw("               search across codebase")]),
        Line::from(vec![Span::styled(":", theme::key_hint()),          Span::raw("               run raw ore command (e.g. :git-log -n 5)")]),
        Line::from(vec![Span::styled("g", theme::key_hint()),          Span::raw("               show git status")]),
        Line::from(vec![Span::styled("x", theme::key_hint()),          Span::raw("               hex view current file")]),
        Line::from(vec![Span::styled("H", theme::key_hint()),          Span::raw("               health report")]),
        Line::from(vec![Span::styled("r", theme::key_hint()),          Span::raw("               refresh tree + git")]),
        Line::from(vec![Span::styled("?", theme::key_hint()),          Span::raw("               toggle this help")]),
        Line::from(vec![Span::styled("q or Esc", theme::key_hint()),   Span::raw("        quit")]),
        Line::from(""),
        Line::from(Span::styled("Command mode (after :)", theme::accent())),
        Line::from(vec![Span::styled("  :q, :quit, :exit", theme::info()), Span::raw("     quit")]),
        Line::from(vec![Span::styled("  :help", theme::info()),             Span::raw("               show help")]),
        Line::from(vec![Span::styled("  :health", theme::info()),           Span::raw("             show codebase health")]),
        Line::from(vec![Span::styled("  :git", theme::info()),              Span::raw("                git status view")]),
        Line::from(vec![Span::styled("  :refresh", theme::info()),          Span::raw("            re-scan tree + git")]),
        Line::from(vec![Span::styled("  :<any ore cmd>", theme::info()),    Span::raw("      run it, show output in preview")]),
    ];
    let p = Paragraph::new(help).style(theme::base());
    f.render_widget(p, area);
}

fn draw_health(f: &mut Frame, area: Rect) {
    let content = crate::engine::proc::run_cmd("ore health .", false, true)
        .map(|r| r.stdout).unwrap_or_else(|_| "(health failed)".to_string());
    let lines: Vec<Line> = content.lines().take(area.height as usize).map(|l| Line::from(Span::styled(l.to_string(), theme::base()))).collect();
    let p = Paragraph::new(lines).style(theme::base());
    f.render_widget(p, area);
}

fn draw_bottom(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(2)])
        .split(area);

    // Separator line
    let sep = Paragraph::new(Span::styled("─".repeat(area.width as usize), theme::dim())).style(theme::base());
    f.render_widget(sep, rows[0]);

    // Input line
    let prompt = match app.mode {
        Mode::Normal => "?".to_string(),
        Mode::Command => ":".to_string(),
        Mode::Search => "/".to_string(),
        Mode::Filter => "filter>".to_string(),
    };
    let hint = match app.mode {
        Mode::Normal => "Enter a coding task or / for search, : for command, ? for help",
        Mode::Command => "Type an ore command and press Enter (Esc to cancel)",
        Mode::Search => "Type a search pattern and press Enter (Esc to cancel)",
        Mode::Filter => "Type to filter tree, Esc to clear",
    };
    let line = if matches!(app.mode, Mode::Normal) {
        Line::from(vec![
            Span::styled(prompt, theme::key_hint()),
            Span::raw(" "),
            Span::styled(hint, theme::dim()),
            Span::raw("     "),
            Span::styled("? End session", theme::accent_dim()),
        ])
    } else {
        Line::from(vec![
            Span::styled(prompt, theme::key_hint()),
            Span::raw(" "),
            Span::styled(app.input.clone(), Style::default().fg(theme::FG).add_modifier(Modifier::BOLD)),
            Span::styled("█", theme::accent()),
        ])
    };
    let input = Paragraph::new(vec![Line::from(""), line]).style(theme::base());
    f.render_widget(input, rows[1]);
}

fn draw_help_overlay(f: &mut Frame, area: Rect) {
    let w = 60.min(area.width.saturating_sub(4));
    let h = 20.min(area.height.saturating_sub(4));
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    let rect = Rect { x, y, width: w, height: h };
    f.render_widget(Clear, rect);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(theme::accent())
        .title(Span::styled(" help (press ? or Esc to close) ", theme::accent()));
    let inner = block.inner(rect);
    f.render_widget(block, rect);
    draw_help(f, inner);
}
