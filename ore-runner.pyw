# Ore Runner - Simple GUI for executing ore commands.
# No encoding corruption. No shell interpretation.
# Calls ore.exe directly via subprocess with argument array.

import tkinter as tk
from tkinter import ttk, scrolledtext, filedialog, messagebox
import subprocess
import threading
import os
import sys
import shlex
import re
import time
from pathlib import Path

# ─── Config ───────────────────────────────────────────────────────

ORE_EXE = None  # Auto-detected below
DEFAULT_CWD = os.getcwd()
FONT_UI = ("Segoe UI", 10)
FONT_CODE = ("Cascadia Code", 10)
FONT_CODE_SMALL = ("Cascadia Code", 9)
BG = "#1a1a1d"
BG_PANEL = "#222225"
BG_INPUT = "#2a2a2e"
BG_OUTPUT = "#1e1e21"
FG = "#e8e8ea"
FG_DIM = "#8b8b90"
ACCENT = "#6be398"
ACCENT_DIM = "#4a9e6b"
ERROR_COLOR = "#ef4444"
WARN_COLOR = "#f5a623"
INFO_COLOR = "#4a90e2"
BORDER = "#3a3a3e"
BTN_BG = "#6be398"
BTN_FG = "#0a0a0b"
BTN_HOVER = "#8aeeb0"
COPY_BG = "#4a9e6b"

def find_ore():
    """Find ore.exe — check PATH, then common locations."""
    # Check PATH
    for p in os.environ.get("PATH", "").split(os.pathsep):
        candidate = os.path.join(p, "ore.exe")
        if os.path.isfile(candidate):
            return candidate
    # Common locations
    candidates = [
        os.path.join(os.path.dirname(__file__), "target", "debug", "ore.exe"),
        os.path.join(os.path.dirname(__file__), "target", "release", "ore.exe"),
        r"C:\Users\ORE\Documents\Dev\oregrep\target\debug\ore.exe",
        r"C:\Users\ORE\Documents\Dev\oregrep\target\release\ore.exe",
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    return "ore"  # Hope it's on PATH

ORE_EXE = find_ore()


# ─── Command Parser ──────────────────────────────────────────────

def is_skippable(line):
    """Lines to skip: empty, markdown headers, code fences, comments, prose."""
    s = line.strip()
    if not s:
        return True
    if s.startswith("```"):
        return True
    if s.startswith("##"):
        return True
    if s.startswith("#") and not s.startswith("#!"):
        return True
    # Skip obvious prose (starts with capital letter + contains spaces + no ore-like tokens)
    if (s[0].isupper() and " " in s and
        not any(s.startswith(kw) for kw in [
            "cd ", "ore ", "find ", "cat ", "patch ", "tree ", "replace ",
            "search", "parallel", "sequence", "on-", "wait ", "run ",
            "mkfile", "mkdir", "mv ", "cp ", "rm ", "touch ",
            "git-", "ai-", "web-", "hex-", "csv-", "json-", "yaml-",
            "toml-", "env-", "xml-", "compile", "verify", "health",
            "symbols", "outline", "refs ", "trace ", "impact ",
            "digest", "condense", "chunk ", "scaffold", "index-",
            "history", "undo ", "redo ", "snip ", "template ",
            "macro ", "convert", "image-", "analyze", "report-",
            "benchmark", "timer ", "monitor ", "schedule ",
            "notify ", "setup ", "check-", "install-",
            "blast-", "related ", "route ", "trim-", "consolidate",
            "rename-", "split-", "merge-", "extract-", "move-",
            "hub ", "flatten-", "organize", "explain ",
            "hot-", "stale-", "since ", "diff-", "workspace-",
            "pack ", "slice ", "map ", "stats ", "count ",
            "wc ", "head ", "tail ", "line ", "sort-", "dedup-",
            "strip-", "collapse-", "purge-", "backup ", "restore ",
            "encoding ", "newlines ", "insert ", "delete-",
            "replace-", "before ", "after ", "surround ",
            "fetch", "post ", "download", "upload ", "ping ",
            "dns ", "crawl ", "status ", "headers ", "filesize ",
            "base64", "xxd ", "bin-", "strings ", "magic ",
            "checksum", "bench-", "retry ", "watch",
            "show ", "copy ", "open-", "focus ", "config ",
            "alias ", "session ", "lock ", "unlock ", "locks",
        ])):
        return True
    return False


def tokenize_command(line):
    """Parse a command line into [program, arg1, arg2, ...] handling quotes properly."""
    tokens = []
    current = []
    in_dq = False
    in_sq = False
    chars = iter(line)
    for c in chars:
        if in_sq:
            if c == "'":
                in_sq = False
            else:
                current.append(c)
            continue
        if in_dq:
            if c == "\\":
                nc = next(chars, None)
                if nc == '"':
                    current.append('"')
                elif nc == "\\":
                    current.append("\\")
                elif nc == "n":
                    current.append("\n")
                elif nc == "t":
                    current.append("\t")
                elif nc is not None:
                    current.append("\\")
                    current.append(nc)
                else:
                    current.append("\\")
            elif c == '"':
                in_dq = False
            else:
                current.append(c)
            continue
        # Outside quotes
        if c == '"':
            in_dq = True
        elif c == "'":
            in_sq = True
        elif c in (" ", "\t"):
            if current:
                tokens.append("".join(current))
                current = []
        elif c == "\\":
            nc = next(chars, None)
            if nc is not None:
                current.append(nc)
            else:
                current.append("\\")
        else:
            current.append(c)
    if current:
        tokens.append("".join(current))
    return tokens


def prepare_command(line, cwd):
    """Convert a user-typed line into (args_list, new_cwd_or_None)."""
    s = line.strip()

    # Handle "cd" specially
    if s == "cd" or s == "cd ~":
        home = os.environ.get("USERPROFILE", os.environ.get("HOME", "."))
        return None, home
    if s.startswith("cd "):
        target = s[3:].strip().strip('"').strip("'")
        if target == "~":
            target = os.environ.get("USERPROFILE", os.environ.get("HOME", "."))
        if not os.path.isabs(target):
            target = os.path.join(cwd, target)
        target = os.path.normpath(target)
        if os.path.isdir(target):
            return None, target
        else:
            return None, f"ERROR: Directory not found: {target}"

    # Strip "ore " prefix if present
    if s.startswith("ore "):
        s = s[4:]

    tokens = tokenize_command(s)
    if not tokens:
        return None, None

    # Check if first token is an ore subcommand
    args = [ORE_EXE] + tokens
    return args, None


# ─── GUI ──────────────────────────────────────────────────────────

class OreRunner(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Ore Runner")
        self.geometry("1400x800")
        self.configure(bg=BG)
        self.cwd = DEFAULT_CWD
        self.running = False
        self.cancel_flag = False

        # Try to set icon
        try:
            self.iconbitmap(default="")
        except:
            pass

        self._build_ui()
        self._bind_keys()

    def _build_ui(self):
        # ── Top bar ──
        top = tk.Frame(self, bg=BG, height=40)
        top.pack(fill="x", padx=0, pady=0)
        top.pack_propagate(False)

        tk.Label(top, text=" ◆ ORE RUNNER", font=("Cascadia Code", 11, "bold"),
                 fg=ACCENT, bg=BG).pack(side="left", padx=10)

        self.cwd_label = tk.Label(top, text=self.cwd, font=FONT_CODE_SMALL,
                                   fg=FG_DIM, bg=BG, cursor="hand2")
        self.cwd_label.pack(side="left", padx=10)
        self.cwd_label.bind("<Button-1>", self._change_dir)

        self.status_label = tk.Label(top, text="Ready", font=FONT_CODE_SMALL,
                                      fg=ACCENT_DIM, bg=BG)
        self.status_label.pack(side="right", padx=10)

        # ── Separator ──
        tk.Frame(self, bg=BORDER, height=1).pack(fill="x")

        # ── Main split ──
        main = tk.PanedWindow(self, orient="horizontal", bg=BG,
                               sashwidth=4, sashrelief="flat",
                               borderwidth=0, opaqueresize=True)
        main.pack(fill="both", expand=True, padx=0, pady=0)

        # ── Left panel: commands ──
        left_frame = tk.Frame(main, bg=BG_PANEL)
        main.add(left_frame, minsize=400, width=600)

        # Left header
        left_header = tk.Frame(left_frame, bg=BG_PANEL, height=36)
        left_header.pack(fill="x")
        left_header.pack_propagate(False)
        tk.Label(left_header, text=" COMMANDS", font=(FONT_UI[0], 9, "bold"),
                 fg=FG_DIM, bg=BG_PANEL).pack(side="left", padx=8)

        btn_clear_left = tk.Label(left_header, text="Clear", font=FONT_CODE_SMALL,
                                   fg=INFO_COLOR, bg=BG_PANEL, cursor="hand2")
        btn_clear_left.pack(side="right", padx=10)
        btn_clear_left.bind("<Button-1>", lambda e: self.cmd_text.delete("1.0", "end"))

        # Command text area
        self.cmd_text = scrolledtext.ScrolledText(
            left_frame, font=FONT_CODE, bg=BG_INPUT, fg=FG,
            insertbackground=ACCENT, selectbackground="#3a5a4a",
            borderwidth=0, highlightthickness=0, wrap="word",
            undo=True, padx=12, pady=10
        )
        self.cmd_text.pack(fill="both", expand=True, padx=4, pady=(0, 4))

        # Buttons bar
        btn_bar = tk.Frame(left_frame, bg=BG_PANEL, height=48)
        btn_bar.pack(fill="x", padx=4, pady=(0, 4))
        btn_bar.pack_propagate(False)

        self.run_btn = tk.Button(
            btn_bar, text="▶  RUN", font=(FONT_UI[0], 11, "bold"),
            bg=BTN_BG, fg=BTN_FG, activebackground=BTN_HOVER,
            activeforeground=BTN_FG, borderwidth=0, cursor="hand2",
            command=self._run_commands, padx=24, pady=6
        )
        self.run_btn.pack(side="left", padx=8, pady=6)

        self.cancel_btn = tk.Button(
            btn_bar, text="■  STOP", font=(FONT_UI[0], 10),
            bg="#444", fg=FG, activebackground="#666",
            borderwidth=0, cursor="hand2",
            command=self._cancel, state="disabled", padx=16, pady=6
        )
        self.cancel_btn.pack(side="left", padx=4, pady=6)

        self.run_selected_btn = tk.Button(
            btn_bar, text="▶ Run Selected", font=FONT_CODE_SMALL,
            bg="#333", fg=FG, activebackground="#555",
            borderwidth=0, cursor="hand2",
            command=self._run_selected, padx=12, pady=6
        )
        self.run_selected_btn.pack(side="left", padx=4, pady=6)

        # ── Right panel: output ──
        right_frame = tk.Frame(main, bg=BG_PANEL)
        main.add(right_frame, minsize=400)

        # Right header
        right_header = tk.Frame(right_frame, bg=BG_PANEL, height=36)
        right_header.pack(fill="x")
        right_header.pack_propagate(False)
        tk.Label(right_header, text=" OUTPUT", font=(FONT_UI[0], 9, "bold"),
                 fg=FG_DIM, bg=BG_PANEL).pack(side="left", padx=8)

        btn_clear_right = tk.Label(right_header, text="Clear", font=FONT_CODE_SMALL,
                                    fg=INFO_COLOR, bg=BG_PANEL, cursor="hand2")
        btn_clear_right.pack(side="right", padx=10)
        btn_clear_right.bind("<Button-1>", lambda e: self._clear_output())

        # Output text area
        self.out_text = scrolledtext.ScrolledText(
            right_frame, font=FONT_CODE, bg=BG_OUTPUT, fg=FG,
            insertbackground=FG, selectbackground="#3a5a4a",
            borderwidth=0, highlightthickness=0, wrap="word",
            state="disabled", padx=12, pady=10
        )
        self.out_text.pack(fill="both", expand=True, padx=4, pady=(0, 4))

        # Configure output tags
        self.out_text.tag_configure("command", foreground=ACCENT, font=(FONT_CODE[0], FONT_CODE[1], "bold"))
        self.out_text.tag_configure("error", foreground=ERROR_COLOR)
        self.out_text.tag_configure("warn", foreground=WARN_COLOR)
        self.out_text.tag_configure("info", foreground=INFO_COLOR)
        self.out_text.tag_configure("dim", foreground=FG_DIM)
        self.out_text.tag_configure("success", foreground=ACCENT)
        self.out_text.tag_configure("separator", foreground="#3a3a3e")

        # Copy button
        copy_bar = tk.Frame(right_frame, bg=BG_PANEL, height=48)
        copy_bar.pack(fill="x", padx=4, pady=(0, 4))
        copy_bar.pack_propagate(False)

        self.copy_btn = tk.Button(
            copy_bar, text="📋  COPY ALL", font=(FONT_UI[0], 11, "bold"),
            bg=COPY_BG, fg="#fff", activebackground=ACCENT,
            activeforeground=BTN_FG, borderwidth=0, cursor="hand2",
            command=self._copy_output, padx=24, pady=6
        )
        self.copy_btn.pack(side="right", padx=8, pady=6)

        self.notepad_btn = tk.Button(
            copy_bar, text="📝  Notepad", font=FONT_CODE_SMALL,
            bg="#333", fg=FG, activebackground="#555",
            borderwidth=0, cursor="hand2",
            command=self._open_notepad, padx=12, pady=6
        )
        self.notepad_btn.pack(side="right", padx=4, pady=6)

        self.save_btn = tk.Button(
            copy_bar, text="💾  Save", font=FONT_CODE_SMALL,
            bg="#333", fg=FG, activebackground="#555",
            borderwidth=0, cursor="hand2",
            command=self._save_output, padx=12, pady=6
        )
        self.save_btn.pack(side="right", padx=4, pady=6)

    def _bind_keys(self):
        self.bind("<F5>", lambda e: self._run_commands())
        self.bind("<Control-Return>", lambda e: self._run_commands())
        self.bind("<Escape>", lambda e: self._cancel())
        self.bind("<Control-l>", lambda e: self._clear_output())
        self.bind("<Control-Shift-C>", lambda e: self._copy_output())

    def _change_dir(self, event=None):
        d = filedialog.askdirectory(initialdir=self.cwd, title="Select working directory")
        if d:
            self.cwd = os.path.normpath(d)
            self.cwd_label.config(text=self.cwd)

    def _clear_output(self):
        self.out_text.config(state="normal")
        self.out_text.delete("1.0", "end")
        self.out_text.config(state="disabled")

    def _append_output(self, text, tag=None):
        self.out_text.config(state="normal")
        if tag:
            self.out_text.insert("end", text, tag)
        else:
            self.out_text.insert("end", text)
        self.out_text.see("end")
        self.out_text.config(state="disabled")

    def _copy_output(self):
        content = self.out_text.get("1.0", "end-1c")
        # Strip ANSI codes
        content = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', content)
        self.clipboard_clear()
        self.clipboard_append(content)
        self.status_label.config(text=f"Copied {len(content)} chars", fg=ACCENT)
        self.after(3000, lambda: self.status_label.config(text="Ready", fg=ACCENT_DIM))

    def _open_notepad(self):
        content = self.out_text.get("1.0", "end-1c")
        content = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', content)
        tmp = os.path.join(os.environ.get("TEMP", "."),
                           f"ore-runner-{int(time.time())}.txt")
        with open(tmp, "w", encoding="utf-8") as f:
            f.write(content)
        subprocess.Popen(["notepad", tmp])

    def _save_output(self):
        content = self.out_text.get("1.0", "end-1c")
        content = re.sub(r'\x1b\[[0-9;]*[a-zA-Z]', '', content)
        path = filedialog.asksaveasfilename(
            defaultextension=".txt",
            filetypes=[("Text", "*.txt"), ("Markdown", "*.md"), ("All", "*.*")],
            initialdir=self.cwd
        )
        if path:
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            self.status_label.config(text=f"Saved to {os.path.basename(path)}", fg=ACCENT)

    def _get_command_lines(self):
        """Get all lines from the command text area."""
        return self.cmd_text.get("1.0", "end").splitlines()

    def _get_selected_lines(self):
        """Get selected text or all if nothing selected."""
        try:
            return self.cmd_text.get("sel.first", "sel.last").splitlines()
        except tk.TclError:
            return self._get_command_lines()

    def _run_commands(self):
        if self.running:
            return
        lines = self._get_command_lines()
        self._execute_lines(lines)

    def _run_selected(self):
        if self.running:
            return
        lines = self._get_selected_lines()
        self._execute_lines(lines)

    def _execute_lines(self, lines):
        self.running = True
        self.cancel_flag = False
        self.run_btn.config(state="disabled", bg="#555")
        self.cancel_btn.config(state="normal")
        self.status_label.config(text="Running...", fg=WARN_COLOR)

        # Filter out skippable lines
        commands = [(i, line) for i, line in enumerate(lines) if not is_skippable(line)]

        def worker():
            for idx, (line_num, line) in enumerate(commands):
                if self.cancel_flag:
                    self._append_output("\n--- CANCELLED ---\n", "error")
                    break

                self.after(0, lambda l=line_num: self.status_label.config(
                    text=f"Running {idx+1}/{len(commands)}...", fg=WARN_COLOR))

                self._append_output(f"$ {line.strip()}\n", "command")

                result = self._execute_one(line.strip())
                if result is not None:
                    self._append_output(result + "\n")

                self._append_output("─" * 60 + "\n", "separator")

            self.after(0, self._done)

        threading.Thread(target=worker, daemon=True).start()

    def _execute_one(self, line):
        """Execute a single command line. Returns output string."""
        args, new_cwd = prepare_command(line, self.cwd)

        # Handle cd
        if args is None:
            if new_cwd and new_cwd.startswith("ERROR:"):
                self._append_output(f"{new_cwd}\n", "error")
                return None
            elif new_cwd:
                self.cwd = new_cwd
                self.after(0, lambda: self.cwd_label.config(text=self.cwd))
                self._append_output(f"→ {self.cwd}\n", "success")
                return None
            return None

        try:
            proc = subprocess.run(
                args,
                capture_output=True,
                cwd=self.cwd,
                timeout=300,
                # No shell=True! Direct process spawn. No cmd.exe.
                # This is the key: special characters are NEVER interpreted.
            )
            out = proc.stdout.decode("utf-8", errors="replace")
            err = proc.stderr.decode("utf-8", errors="replace")

            result = ""
            if out:
                result += out
            if err:
                if result and not result.endswith("\n"):
                    result += "\n"
                result += err

            if proc.returncode != 0:
                self._append_output(f"✗ exit {proc.returncode}\n", "error")

            return result

        except subprocess.TimeoutExpired:
            self._append_output("✗ TIMEOUT (300s)\n", "error")
            return None
        except FileNotFoundError:
            self._append_output(f"✗ ore not found at: {ORE_EXE}\n", "error")
            return None
        except Exception as e:
            self._append_output(f"✗ {str(e)}\n", "error")
            return None

    def _cancel(self):
        self.cancel_flag = True
        self.status_label.config(text="Cancelling...", fg=ERROR_COLOR)

    def _done(self):
        self.running = False
        self.run_btn.config(state="normal", bg=BTN_BG)
        self.cancel_btn.config(state="disabled")
        self.status_label.config(text="Done", fg=ACCENT)
        self.after(5000, lambda: self.status_label.config(text="Ready", fg=ACCENT_DIM))


# ─── Main ─────────────────────────────────────────────────────────

if __name__ == "__main__":
    # Set UTF-8 console codepage on Windows
    if sys.platform == "win32":
        os.system("")  # Enable ANSI on Windows 10+
        try:
            import ctypes
            ctypes.windll.kernel32.SetConsoleOutputCP(65001)
            ctypes.windll.kernel32.SetConsoleCP(65001)
        except:
            pass

    app = OreRunner()
    app.mainloop()