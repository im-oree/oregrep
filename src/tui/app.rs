use anyhow::Result;
use std::path::PathBuf;

use crate::engine::state::read_focus;

/// Which main pane is currently visible
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum View {
    Preview,
    Search,
    GitStatus,
    Hex,
    Help,
    Health,
}

/// Which pane owns the keyboard cursor
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Focus {
    Tree,
    Main,
    #[allow(dead_code)] // reserved: explicit command-bar focus in a later polish pass
    CommandBar,
}

/// Input mode for the command bar
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Mode {
    Normal,
    Command,   // ":" — raw ore subcommand
    Search,    // "/" — fuzzy search everything
    #[allow(dead_code)] // reserved: in-tree filename filter (future)
    Filter,    // in-tree filename filter
}

pub struct TreeEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct App {
    pub root: PathBuf,
    pub tree: Vec<TreeEntry>,
    pub tree_selected: usize,
    pub tree_scroll: usize,
    pub focus: Focus,
    pub view: View,
    pub mode: Mode,
    pub input: String,
    pub status_msg: String,
    pub preview_lines: Vec<String>,
    pub preview_scroll: usize,
    pub preview_path: Option<PathBuf>,
    pub search_results: Vec<(PathBuf, usize, String)>, // (file, line, text)
    pub search_selected: usize,
    pub git_branch: String,
    pub git_dirty: usize,
    pub show_help_overlay: bool,
    pub quit: bool,
}

impl App {
    pub fn new(root: PathBuf) -> Result<Self> {
        let tree = build_tree(&root, 0, 2)?;
        let mut app = Self {
            root,
            tree,
            tree_selected: 0,
            tree_scroll: 0,
            focus: Focus::Tree,
            view: View::Preview,
            mode: Mode::Normal,
            input: String::new(),
            status_msg: String::new(),
            preview_lines: vec![],
            preview_scroll: 0,
            preview_path: None,
            search_results: vec![],
            search_selected: 0,
            git_branch: String::new(),
            git_dirty: 0,
            show_help_overlay: false,
            quit: false,
        };
        app.refresh_git_status();
        Ok(app)
    }

    pub fn refresh_git_status(&mut self) {
        self.git_branch = crate::engine::git::git(&["branch", "--show-current"])
            .ok().map(|s| s.trim().to_string()).unwrap_or_default();
        self.git_dirty = crate::engine::git::changed_files().map(|v| v.len()).unwrap_or(0);
    }

    pub fn selected_entry(&self) -> Option<&TreeEntry> {
        self.tree.get(self.tree_selected)
    }

    pub fn move_tree(&mut self, delta: i32) {
        let len = self.tree.len();
        if len == 0 { return; }
        let sel = self.tree_selected as i32 + delta;
        self.tree_selected = sel.clamp(0, len as i32 - 1) as usize;
    }

    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.tree.get(self.tree_selected) {
            if !entry.is_dir { return; }
            let path = entry.path.clone();
            let depth = entry.depth;
            let currently_expanded = entry.expanded;
            if currently_expanded {
                // Collapse: remove all children below
                self.tree[self.tree_selected].expanded = false;
                let cutoff = self.tree_selected + 1;
                let mut end = cutoff;
                while end < self.tree.len() && self.tree[end].depth > depth {
                    end += 1;
                }
                self.tree.drain(cutoff..end);
            } else {
                // Expand: insert children below
                self.tree[self.tree_selected].expanded = true;
                if let Ok(children) = build_tree(&path, depth + 1, 1) {
                    let insert_at = self.tree_selected + 1;
                    for (i, child) in children.into_iter().enumerate() {
                        self.tree.insert(insert_at + i, child);
                    }
                }
            }
        }
    }

    pub fn open_selected(&mut self) {
        let entry_opt = self.selected_entry().map(|e| (e.path.clone(), e.is_dir));
        if let Some((path, is_dir)) = entry_opt {
            if is_dir {
                self.toggle_expand();
            } else {
                self.load_preview(&path);
            }
        }
    }

    pub fn load_preview(&mut self, path: &std::path::Path) {
        match crate::engine::encoding::is_binary(path).unwrap_or(false) {
            true => {
                self.preview_lines = vec![format!("(binary file: {} bytes)",
                    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0))];
            }
            false => match crate::engine::encoding::read_file_smart(path) {
                Ok(content) => {
                    self.preview_lines = content.lines().map(|s| s.to_string()).collect();
                }
                Err(e) => {
                    self.preview_lines = vec![format!("(error reading: {})", e)];
                }
            }
        }
        self.preview_scroll = 0;
        self.preview_path = Some(path.to_path_buf());
        self.view = View::Preview;
        self.status_msg = format!("Opened {}", path.display());
    }

    pub fn scroll_preview(&mut self, delta: i32) {
        let len = self.preview_lines.len();
        if len == 0 { return; }
        let sel = self.preview_scroll as i32 + delta;
        self.preview_scroll = sel.clamp(0, (len as i32 - 1).max(0)) as usize;
    }

    pub fn execute_command(&mut self, cmd: &str) {
        let cmd = cmd.trim();
        if cmd.is_empty() { return; }
        // Match well-known short commands
        match cmd {
            "q" | "quit" | "exit" => { self.quit = true; return; }
            "help" | "?" => { self.view = View::Help; return; }
            "health" => { self.view = View::Health; return; }
            "git" | "status" => { self.view = View::GitStatus; return; }
            "hex" => { self.view = View::Hex; return; }
            "refresh" => {
                self.tree = build_tree(&self.root, 0, 2).unwrap_or_default();
                self.refresh_git_status();
                self.status_msg = "Refreshed.".to_string();
                return;
            }
            _ => {}
        }
        // Fall through: execute as ore <cmd> and stash the output
        let full = format!("ore {}", cmd);
        match crate::engine::proc::run_cmd(&full, false, true) {
            Ok(r) => {
                let mut lines: Vec<String> = Vec::new();
                lines.push(format!("$ {}", full));
                lines.push(format!("[exit {}, {}ms]", r.exit_code, r.duration_ms));
                for l in r.stdout.lines() { lines.push(l.to_string()); }
                if !r.stderr.is_empty() {
                    lines.push("--- stderr ---".to_string());
                    for l in r.stderr.lines() { lines.push(l.to_string()); }
                }
                self.preview_lines = lines;
                self.preview_scroll = 0;
                self.preview_path = None;
                self.view = View::Preview;
                self.status_msg = format!("Ran: {}", cmd);
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
    }

    pub fn run_search(&mut self, query: &str) {
        let q = query.trim();
        if q.is_empty() { self.search_results.clear(); return; }
        let re = match regex::RegexBuilder::new(q).case_insensitive(true).build() {
            Ok(r) => r,
            Err(_) => return,
        };
        let cfg = crate::engine::walker::WalkConfig {
            root: self.root.clone(),
            skip_backups: true,
            ..Default::default()
        };
        let files = crate::engine::walker::collect_files(&cfg).unwrap_or_default();
        let mut results: Vec<(PathBuf, usize, String)> = Vec::new();
        for f in files {
            if let Ok(content) = crate::engine::encoding::read_file_smart(&f) {
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        results.push((f.clone(), i + 1, line.trim().to_string()));
                        if results.len() >= 500 { break; }
                    }
                }
            }
            if results.len() >= 500 { break; }
        }
        self.search_results = results;
        self.search_selected = 0;
        self.view = View::Search;
        self.status_msg = format!("Search: {} results", self.search_results.len());
    }
}

pub fn build_tree(root: &std::path::Path, depth: usize, max_expand_depth: usize) -> Result<Vec<TreeEntry>> {
    let mut out = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .max_depth(Some(1))
        .build();
    let mut entries: Vec<_> = walker.flatten().collect();
    entries.sort_by(|a, b| {
        let ad = a.path().is_dir();
        let bd = b.path().is_dir();
        if ad != bd { return bd.cmp(&ad); }
        a.path().cmp(b.path())
    });
    for entry in entries {
        let p = entry.path().to_path_buf();
        if p == root { continue; }
        let is_dir = p.is_dir();
        out.push(TreeEntry { path: p, is_dir, depth, expanded: false });
    }
    // Recursion managed by depth arg — this only expands one level; expansion done on toggle
    let _ = max_expand_depth; // kept for future auto-expand
    Ok(out)
}

pub fn detect_root() -> PathBuf {
    if let Ok(Some(f)) = read_focus() { return f; }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
