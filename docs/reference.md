# ore — Complete Command Reference

**ore** (built from the `oregrep` project) is a powerful all-in-one file, code, and codebase
manipulation CLI. It ships **310 commands** covering text search, safe file editing with an undo
safety net, git workflows, process automation, HTTP/API testing, binary/hex tooling, structured
data (JSON/YAML/TOML/CSV/XML/.env), code analysis and refactoring, scaffolding, a SQLite index,
browser automation, web search, and AI-powered assistance (chat, agents, review, fix, refactor).

This is the complete reference. It documents **every command** — what it does, the syntax, the
key options, concrete use cases, an example, and — explicitly — **what it cannot do**. The
"Can't do" lines are as important as the "use cases": they tell you when to reach for a
different command, or when a limitation is by design.

---

## How to read this reference

Every command entry follows the same shape:

- **What it does** — one or two sentences.
- **Syntax** — `ore <command> [options] <arguments>`. Brackets `[ ]` mean optional; angle
  brackets `< >` mean required.
- **Options** — the flags that matter most. Every command also accepts `-h, --help`, which
  prints the full flag list (run `ore <command> --help` for the authoritative list — this
  document summarizes, it does not replace `--help`).
- **Use cases** — realistic scenarios where the command earns its keep.
- **Example** — a runnable invocation.
- **Can't do** — the honest limitations and "not possible" notes.

---

## Core concepts you need before the commands

### The safety net (backups, history, undo, redo)

Editing commands (`patch`, `replace`, `insert`, `delete-lines`, `replace-line`,
`replace-range`, `before`, `after`, `surround`, `rename-bulk`, `mv`, `cp`, `rm`, `encoding`,
`newlines`, and friends) create **backups** automatically before they touch a file, unless you
pass `--no-backup`. Backups are sibling files like `file.txt.bak20260814-091233` or
`file.txt.bakLABEL` when you pass `-l LABEL`.

- `ore backup <file>` — make a backup on demand. `ore backup <file> --list` — list backups.
- `ore restore <file> [-l LABEL]` — restore from the most recent (or a labeled) backup.
- `ore diff <file> --backup` — diff the current file against its latest backup.
- `ore history` — every recorded operation (backups, patches, deletes), newest first.
- `ore undo [-y] [N]` — undo the last N recorded operations by restoring from their backups.
- `ore redo [-y]` — clears the undone mark. **It does not replay changes** — `redo` only marks
  an undone operation as redone so `undo` won't re-apply it.
- `ore purge-backups` — remove old backup files.

This is the model behind `ai-fix` / `ai-refactor`: the agent backs up, edits, verifies, and can
`restore` + try again if verification fails. **Any destructive edit without a backup is your own
choice (`--no-backup`)** — the default is always safe.

### The index (fast reuse across commands)

`ore index-build` builds a SQLite database (at `<workspace-root>/.ore-index/index.db`) of files,
symbols, and imports. `index-update` refreshes only changed files; `index-search` answers symbol
lookups from the index instead of rescanning. `index-status`, `index-gc` (remove orphans +
vacuum), `index-clear`, and `index-locate` (print the DB path) manage it. The index is
**opt-in** (`--from-index` is off by default) — commands work without it, just slower.

### State and configuration locations

| What | Where |
|---|---|
| Global config (`ai.toml`, search config, etc.) | Windows `%APPDATA%\ore\` · Unix `~/.config/ore/` |
| AI API keys (`secrets.toml`) | Same state directory |
| Index DB + AI usage/session/model tables | `<current-workspace>/.ore-index/index.db` |
| Backups | Next to the file, `.<name>.bak…` |
| Op history | Index DB (operation log) |

Because the AI tables live in the workspace `.ore-index`, `ai-usage`, `ai-history`,
`ai-session`, and `ai-recall` are **per-workspace** by design — switch directories and you see
that directory's history.

### Exit codes

- `0` — success.
- `1` — command failed (validation, network, HTTP error, budget exceeded, etc.). Some commands
  use non-zero exit codes as *signals*: `web-ws-status` exits 1 when a page isn't ready;
  `ai-keys test` exits 1 when a provider test fails; `undo`/`redo` require `-y` or confirmation.
- `2` — clap usage error (bad flags/arguments).

### Output conventions

- Colored output by default; most commands respect `--quiet`/`-q` where they offer it.
- `-j` / `--json` on commands that support it emits machine-readable JSON.
- AI commands can emit a structured event stream with `--events-json` (one JSON object per
  line, on stderr) for GUIs and tooling.
- `ai-ask -q` = result only, no event chatter. The final answer is always printed once.

---

## Command index by category

| Section | Commands |
|---|---|
| Files & basic I/O | find cat cat-around line tree head tail count stats wc mv cp rm touch mkdir mkfile mkfile-from checksum find-dupes verify-checksum extract pack pack-lines pack-changed slice map dedup-lines sort-lines trim strip-blank-lines collapse-blank-lines show copy to-temp open-file purge-backups |
| Editing & safety | backup restore patch patch-lines patch-insert patch-preview patch-regex patch-batch replace insert delete-lines replace-line replace-range before after surround replace-project replace-ext replace-dir patch-project rename-bulk diff encoding newlines |
| Diffs & merges | diff-word diff-semantic diff-ignore diff-dirs merge3 apply-patch revert-patch diff-summary |
| Search | search-and search-or search-negative search-multiline search-fuzzy search-changed search-history |
| Git | git-status git-changed git-diff git-history git-blame git-search git-who git-stage git-commit git-log git-auto-commit git-auto-message git-suggest-commit git-commit-body git-changelog git-release-notes git-undo-commit git-amend git-fixup git-cleanup-branches git-stash-named |
| Process automation | run wait retry parallel sequence watch watch-multi on-error on-success monitor notify schedule timer benchmark |
| HTTP & networking | fetch post download headers status ping dns api-test filesize upload fetch-many download-many check-urls resume-download bench-url ws crawl |
| Binary & hex | hex-view hex-find hex-replace hex-patch hex-diff hex-extract hex-insert hex-delete strings magic bin-stats base64-encode base64-decode xxd bin-slice bin-cat |
| Data formats | json-get json-set json-merge json-fmt json-query json-keys yaml-get yaml-set yaml-fmt yaml-to-json toml-get toml-set toml-fmt toml-to-json csv-query csv-filter csv-select csv-to-json csv-stats env-get env-set env-diff xml-get xml-fmt xml-to-json |
| Code analysis | symbols outline snippet pluck refs used-by imports-of neighbors add-import remove-import split-file merge-files extract-fn move-with-imports hub flatten-hub rename-symbol organize analyze-imports analyze-exports analyze-coupling analyze-churn analyze-hotspot analyze-complexity analyze-dead-exports analyze-circular analyze-type-coverage analyze-duplication impact trace blast-radius related route trim-dead consolidate rename-safe since hot-files stale-files explain digest condense chunk ai-prompt |
| Reports | report-health report-todos report-imports report-api report-contributors report-coverage report-changes report-errors workspace-report |
| Compile & verify | compile-ts compile-rust compile-node errors-last verify health verify-json verify-syntax verify-encoding verify-imports verify-anchor |
| Scaffolding & tooling | scaffold scaffold-add scaffold-component scaffold-hook scaffold-store scaffold-context scaffold-api scaffold-test setup check-deps install-if-missing snip template macro |
| Workspace & config | session focus notes bookmark tag session-export lock unlock locks config alias tui |
| Index & history | index-build index-update index-status index-clear index-locate index-gc index-search history undo redo |
| Browser automation | web-open web-screenshot web-pdf web-text web-html web-title web-links web-click web-type web-eval web-wait web-scrape web-screenshot-many web-screenshot-set web-cookies web-ws-status web-check |
| Web search & AI | web-search web-search-config web-search-instances web-fetch-clean ai-search-test ai-keys ai-config ai-models ai-providers ai-usage ai-history ai-budget ai-prompts ai-session ai-recall ai-ask ai-chat ai-agent ai-explain ai-review ai-fix ai-refactor ai-commit-message |

---

## What ore is **not** (the honest overview)

- **Not an IDE.** `tui` gives an interactive file tree/search/palette, but there is no editor
  with undo in the UI sense — editing happens via commands, and safety comes from backups/undo.
- **No image/SVG conversion commands yet.** The `convert-*` family was scaffolded but never
  shipped — don't look for `ore convert-png-to-webp`; it doesn't exist.
- **Not a package manager.** `install-if-missing` can install tools via winget/choco/npm/cargo/
  scoop, but ore is not a dependency manager for your project.
- **Browser automation is stateless per command.** Every `web-*` command launches a fresh
  headless browser profile; nothing persists between commands (cookies, sessions, typed text).
- **`redo` does not replay.** It only clears the undone mark — see the safety-net section.
- **AI needs a backend.** Without a registered key, AI commands fall back to local
  Ollama/LM Studio; with neither, they fail with a clear error. Free tiers have rate limits
  (HTTP 429) and payload limits (HTTP 413) that ore retries/advises on but cannot remove.
- **Some commands are Windows-specific** (`schedule` wraps Windows Task Scheduler; `show`
  defaults to notepad).
# Files & basic I/O

### `ore find`
**What it does:** Search files for a pattern (regex by default) and print matching lines with file paths and line numbers.
**Syntax:** `ore find <PATTERN> [PATH]`
**Options:** `-F` literal (not regex) · `-i` ignore case · `-w` whole word · `-H` include hidden · `--no-ignore` ignore `.gitignore` · `--binary` include binaries · `-l` files-only · `-c` count-only · `-B/-A` before/after context · `-e` extension filter (e.g. `ts,tsx,rs`)
**Use cases:** The everyday grep — "where is `foo()` defined/used?"; count matches per file (`-c`); list files that mention a term (`-l`); search only Rust files (`-e rs`).
**Example:** `ore find "TODO|FIXME" src -i -c` — case-insensitive TODO/FIXME counts per file.
**Can't do:** Not fuzzy — a typo means no match (use `search-fuzzy`). Skips hidden and binary files unless you opt in. No ranking by relevance. For symbol-level questions prefer `ore refs`/`ore symbols`.

### `ore cat`
**What it does:** Print a file with smart encoding detection.
**Syntax:** `ore cat <FILE>`
**Options:** `-n` line numbers · `-g` show only lines matching a pattern · `--binary` force-print binaries · `--raw` raw bytes without decoding
**Use cases:** Read any file with correct encoding; scan a file for lines matching a pattern without a full search; inspect a file that isn't UTF-8.
**Example:** `ore cat src/main.rs -n -g "pub fn"` — numbered list of public functions.
**Can't do:** Not a pager (no scrollback) — pipe to `more`/`less` for long files. It doesn't decode truly binary content (use `hex-view`/`strings`).

### `ore cat-around`
**What it does:** Print N lines of context around every match of a pattern in a file.
**Syntax:** `ore cat-around <FILE> <PATTERN>`
**Options:** `-C <n>` context lines each side (default 5) · `-n` line numbers · `-i` ignore case · `-x` regex
**Use cases:** Read the function body around a symbol you're investigating; get surgical context for an AI prompt without a full `cat`.
**Example:** `ore cat-around src/main.rs "fn run" -C 5 -n`
**Can't do:** One file per call — for multiple files use `pack-lines` or `find -C`. Substring match by default; use `-x` for regex.

### `ore line`
**What it does:** Print a specific line or range from a file.
**Syntax:** `ore line <FILE> <RANGE>` — ranges like `42`, `10:20`, or `10-20`
**Options:** `-N` suppress line numbers · `-C` context lines before/after
**Use cases:** Jump straight to a line a compiler error mentioned; grab a range to share with a colleague or paste into an AI prompt.
**Example:** `ore line src/main.rs 100-120 -C 2`
**Can't do:** No regex — exact line numbers only. For pattern-based extraction use `extract`/`slice`.

### `ore tree`
**What it does:** Print a directory tree.
**Syntax:** `ore tree [PATH]`
**Options:** `-d` max depth · `-H` hidden · `--no-ignore` · `-s` sizes · `-e` extension filter · `-D` dirs-only
**Use cases:** Orient yourself in an unfamiliar codebase; verify a scaffold's shape; show folder structure in a report.
**Example:** `ore tree src -d 2 -e ts,tsx -s`
**Can't do:** Not interactive (that's `tui`). No file-count summary by default.

### `ore head` / `ore tail`
**What they do:** Print the first / last N lines of a file.
**Syntax:** `ore head <FILE> [-n N]` · `ore tail <FILE> [-n N]` · `-N` line numbers
**Use cases:** Peek at a log's beginning/end; preview a file before editing; check the tail of a long CSV.
**Example:** `ore tail error.log -n 50`
**Can't do:** No `-f` follow mode (use `watch` + `tail`, or `monitor` for streaming).

### `ore count`
**What it does:** Count occurrences of a pattern across files.
**Syntax:** `ore count <PATTERN> [PATH]`
**Options:** `-F -i -w -e <ext> -x <exclude>` · `-v` per-file breakdown
**Use cases:** "How many times is `useEffect` imported in this repo?"; count a TODO debt; quick metric before a refactor.
**Example:** `ore count "console\.log" src -i`
**Can't do:** Only counts — no line context (use `find`).

### `ore stats`
**What it does:** File/line/size statistics across a path.
**Syntax:** `ore stats [PATH]`
**Options:** `-e -x -H --no-ignore` · `-n` top-N largest by size
**Use cases:** Codebase size overview; find the biggest files; spot a giant generated file.
**Example:** `ore stats . -n 10`
**Can't do:** Not per-extension breakdown — for that use `wc`/`map` together with filters.

### `ore wc`
**What it does:** Word/line/byte/character counts.
**Syntax:** `ore wc [FILES]...`
**Options:** `-l -w -c -m` show only one metric
**Use cases:** LOC reports; token-budget estimation (chars/4 ≈ tokens); log size checks.
**Example:** `ore wc src/**/*.rs -l`
**Can't do:** No recursive directory walk (pass files explicitly or use `stats`).

### `ore mv`
**What it does:** Move/rename a file or directory. **Auto-backs up the target on overwrite.**
**Syntax:** `ore mv <SRC> <DST>`
**Options:** `-y` bypass confirmation · `--force` overwrite without backup · `--no-backup` · `-l <label>` · `--dry-run`
**Use cases:** Safe rename — the undo net catches mistakes; rename a file that another file imports (though for code, `move-with-imports` fixes importers too).
**Example:** `ore mv src/a.ts src/b.ts`
**Can't do:** Doesn't update importers (see `move-with-imports`). On overwrite, the *target's* previous content is what gets backed up.

### `ore cp`
**What it does:** Copy a file or directory (auto-backup on overwrite).
**Syntax:** `ore cp <SRC> <DST>`
**Options:** `-r` recursive · `-y` · `--force` · `--no-backup` · `-l` · `--dry-run`
**Use cases:** Duplicate a config file before experimenting; copy a template file.
**Example:** `ore cp .env .env.bak-local -y`
**Can't do:** No symlink handling guarantees; copying into an existing directory uses the same backup semantics as `mv`.

### `ore rm`
**What it does:** Delete files/directories **with confirmation and a backup first**.
**Syntax:** `ore rm [PATHS]...`
**Options:** `-r` recursive (required for dirs) · `-y` bypass confirm · `-f` force (no backup) · `--no-backup` · `-l` · `--dry-run`
**Use cases:** Delete with a safety net — `undo` can restore; clean up a scratch file without fear.
**Example:** `ore rm build/ -r -y`
**Can't do:** With `-f`/`--no-backup` the safety net is off — deleted means gone (well, until your file system's recycle bin, which ore does not manage).

### `ore touch`
**What it does:** Create empty files or update mtime.
**Syntax:** `ore touch [FILES]...` · `-p` create parent dirs
**Use cases:** Create placeholder files; bump timestamps to trigger watchers.
**Example:** `ore touch src/types.ts -p`
**Can't do:** Nothing fancy — no mode/chown flags.

### `ore mkdir`
**What it does:** Create directories (recursive).
**Syntax:** `ore mkdir [PATHS]...`
**Use cases:** The obvious one. Recursive by default, so `ore mkdir a/b/c` works.
**Example:** `ore mkdir docs/images/screenshots`
**Can't do:** No `-v` verbose or `-p` flag needed — it's always recursive and quiet on success.

### `ore mkfile`
**What it does:** Create a file with optional initial content.
**Syntax:** `ore mkfile <FILE>`
**Options:** `-c <content>` initial content (`\n` for newlines) · `-p` parents · `--force` overwrite · `-y`
**Use cases:** Script the creation of files with a starting skeleton.
**Example:** `ore mkfile .gitignore -c "node_modules\ntarget"`
**Can't do:** Content is a single string — for templates with variables use `template`; for scaffolds use `scaffold-*`.

### `ore mkfile-from`
**What it does:** Create a file whose content comes from the clipboard, stdin, or another file.
**Syntax:** `ore mkfile-from <FILE>` — one source required: `--clipboard`, `--stdin`, or `--file <SOURCE>`
**Options:** `-f` overwrite without prompting · `--no-backup` · `-l <label>` · `--strip-bom`
**Use cases:** Save what you just copied to disk; materialize piped output as a file; clone an existing file to a new path.
**Example:** `ore mkfile-from notes.md --clipboard`
**Can't do:** One source per call; overwriting prompts unless `-f` (and backs up the target first by default).

### `ore checksum`
**What it does:** Compute file checksums.
**Syntax:** `ore checksum [FILES]...` · `-a sha256|md5|crc32|all`
**Use cases:** Verify a downloaded artifact; dedupe detection; integrity checks.
**Example:** `ore checksum package.zip -a md5`
**Can't do:** One-shot only — to *verify* against an expected hash use `verify-checksum`.

### `ore verify-checksum`
**What it does:** Verify a file against an expected checksum.
**Syntax:** `ore verify-checksum <FILE> <EXPECTED>`
**Use cases:** Confirm a download matches the published hash.
**Example:** `ore verify-checksum app.exe 4f2a…`
**Can't do:** No algorithm autodetect — it compares the expected string against sha256 by default (use `checksum` first to see what you're comparing).

### `ore find-dupes`
**What it does:** Find duplicate files by content hash.
**Syntax:** `ore find-dupes <PATHS>...`
**Options:** `-e <ext>` · `-x <exclude>` · `-H` · `--no-ignore` · `-s <min-size>`
**Use cases:** Reclaim disk space; find accidentally duplicated assets; detect vendored copies.
**Example:** `ore find-dupes . -e png,jpg`
**Can't do:** Reports only — it never deletes. Exact-content match only (no near-dupes — see `consolidate` for code).

### `ore extract`
**What it does:** Extract line ranges from one or more files — multi-range, multi-file.
**Syntax:** `ore extract [FILE] [RANGES]` — also `--spec "file1:10-30,file2:5-15"` or `--spec-file`
**Options:** `-L` labels · `-C` context · `-n` numbers (`--line-numbers`) · `-m` merge overlapping ranges · `-o` output file · `--plain`
**Use cases:** Pull the relevant hunks of several files into one prompt or review file; build a focused context bundle for AI.
**Example:** `ore extract src/app.ts 40-60,120-140 -L -o ctx.txt`
**Can't do:** Line numbers only — for pattern-based slicing use `slice`.

### `ore pack`
**What it does:** Pack files into an AI-ready blob (md/xml/tag/plain) with tree, strip, and truncate options.
**Syntax:** `ore pack [INPUTS]...`
**Options:** `-e <ext>` · `-x <exclude>` · `-f <format>` · `-o <output>` · `--copy` · `--max-lines-per-file` · `--strip-blanks` · `--strip-comments` · `--include-tree`
**Use cases:** Assemble a codebase snapshot for an LLM prompt; share a review bundle; archive a module.
**Example:** `ore pack src --format md --strip-comments -o snapshot.md`
**Can't do:** It packs text files — binary assets are skipped or garbled. For a structural digest (not full contents) use `digest`.

### `ore pack-lines`
**What it does:** Pack specific line ranges from multiple files into one AI-ready blob.
**Syntax:** `ore pack-lines <SPECS>...` — specs like `path`, `path:N`, `path:N-M`, `path:N:M`
**Options:** `--format tag|md|plain` (default tag) · `-n` numbers · `--label` show file+range labels (always on for tag/md)
**Use cases:** Grab exactly the hunks you need across several files for a review or LLM context; build a focused bundle without `extract` gymnastics.
**Example:** `ore pack-lines src/a.ts:80-120 src/b.ts:1-50 --format md`
**Can't do:** Line numbers only — for pattern-based pulls use `slice`/`extract`.

### `ore pack-changed`
**What it does:** Pack all files changed since a git ref (default HEAD).
**Syntax:** `ore pack-changed [SINCE]`
**Options:** `--format tag|md|plain` · `-n` numbers · `--dir <DIR>` · `--untracked` include untracked · `-e <ext>` filter
**Use cases:** Bundle your uncommitted work for review; hand an LLM everything you changed today; assemble PR context.
**Example:** `ore pack-changed HEAD -e ts --format md`
**Can't do:** Needs a git repo; only *changed* files are included (new/modified — deleted files are skipped).

### `ore slice`
**What it does:** Slice content between pattern markers (start/end regex).
**Syntax:** `ore slice --start <START> [--end <END>] <FILE>`
**Options:** `--include-start/--include-end` · `-a` all occurrences · `-L` labels · `-N` numbers · `-i` ignore case · `-o` output
**Use cases:** Extract a function body or a section between `<!-- BEGIN -->` markers; pull the `# Changelog` section out of a README.
**Example:** `ore slice README.md -s "^## Changelog" -e "^## " -a`
**Can't do:** Requires at least a start pattern; no line-number ranges (use `extract`).

### `ore map`
**What it does:** Codebase map: per-file lines/size/exports/imports overview.
**Syntax:** `ore map [PATH]`
**Options:** `-e -x -H --no-ignore` · `-s name|lines|size|exports|imports` sort · `-r` reverse · `-n` top-N
**Use cases:** One-glance project layout; find the file with the most exports; list the biggest files.
**Example:** `ore map src -s lines -r -n 15`
**Can't do:** Summarizes, doesn't analyze — for coupling/import graphs use `analyze-*` and `neighbors`.

### `ore dedup-lines`
**What it does:** Remove duplicate lines from a file.
**Syntax:** `ore dedup-lines <FILE>`
**Options:** `-a` adjacent-only (like `uniq`) · `-i` ignore case · `-t` trim-ignore · `--no-backup -l --dry-run`
**Use cases:** Clean a generated list; dedupe a hosts file; tidy a whitelist.
**Example:** `ore dedup-lines allowlist.txt -i -t`
**Can't do:** Order of first occurrence is kept for non-adjacent mode; it's line-based, not token-based.

### `ore sort-lines`
**What it does:** Sort lines in a file.
**Syntax:** `ore sort-lines <FILE>`
**Options:** `-r` reverse · `-i` case-insensitive · `-n` numeric · `-u` unique · `--no-backup -l --dry-run`
**Use cases:** Alphabetize imports lists, keyword lists, or a glossary.
**Example:** `ore sort-lines keywords.txt -i -u`
**Can't do:** In-place only (no stdout mode); numeric sort treats each line as one number.

### `ore trim`
**What it does:** Trim whitespace from lines.
**Syntax:** `ore trim <FILE>`
**Options:** `-t` trailing (default) · `-L` leading · `-b` both · `--no-backup -l --dry-run`
**Use cases:** Clean up files with trailing spaces (common lint complaint).
**Example:** `ore trim src/main.rs -b`
**Can't do:** Not a full formatter — tabs/spaces conversion is out of scope.

### `ore strip-blank-lines`
**What it does:** Remove empty lines.
**Syntax:** `ore strip-blank-lines <FILE>` · `--no-backup -l --dry-run`
**Use cases:** Compact a config or data file.
**Example:** `ore strip-blank-lines config.yml`
**Can't do:** Removes all blank lines — if you want to keep *some* separation, use `collapse-blank-lines`.

### `ore collapse-blank-lines`
**What it does:** Cap consecutive blank lines.
**Syntax:** `ore collapse-blank-lines <FILE>` · `-m <max>` (default 1) · `--no-backup -l --dry-run`
**Use cases:** Normalize formatting without destroying readability.
**Example:** `ore collapse-blank-lines report.md -m 1`
**Can't do:** Line-based only; won't touch lines containing whitespace unless they're truly empty? (It collapses blank runs — whitespace-only lines count as blank.)

### `ore show`
**What it does:** Show content (stdin or file) in an editor (default: notepad). Strips ANSI colors.
**Syntax:** `ore show [FILE]`
**Options:** `-e <editor>` · `-p <prefix>` · `-x <ext>` · `-d` detached · `-P` print path
**Use cases:** Pipe command output into an editor; open a quick scratch view.
**Example:** `ore find "bug" src -n | ore show`
**Can't do:** Blocking by default (waits for the editor) — use `-d` to detach. Editor must exist on PATH (notepad default on Windows).

### `ore copy`
**What it does:** Copy content (stdin or file) to the system clipboard.
**Syntax:** `ore copy [FILE]` · `-t` tee (also print to stdout)
**Use cases:** Pipe a command's output into the clipboard for pasting; copy a file's content.
**Example:** `ore digest . | ore copy`
**Can't do:** One-shot — no clipboard history.

### `ore to-temp`
**What it does:** Write stdin to a temp file and print its path (chain with `ore open-file`).
**Syntax:** `ore to-temp` · `-x <ext>` · `-p <prefix>` · `-s` strip ANSI
**Use cases:** Feed generated content to an editor or an AI tool that needs a file path.
**Example:** `ore web-text https://example.com | ore to-temp -x html -P`
**Can't do:** The temp file is yours to manage — ore doesn't auto-clean it.

### `ore open-file`
**What it does:** Open a file or folder in the default OS handler or a specified editor.
**Syntax:** `ore open-file <PATH>` · `-e <editor>` · `-F` open containing folder
**Use cases:** Open a file in your editor from a pipeline; reveal a file in Explorer.
**Example:** `ore open-file src/main.rs -e code` (opens in VS Code if `code` is on PATH)
**Can't do:** Depends on the OS default handler or an editor that exists on PATH.

### `ore purge-backups`
**What it does:** Remove old backup files (`.bak…`).
**Syntax:** `ore purge-backups [PATH]`
**Options:** `--label <LABEL>` only this label · `--older-than <mins>` · `--newer-than <mins>` · `--matching <substr>` · `--dry-run` · `-y`
**Use cases:** Housekeeping after a long editing session; remove session-only backups while keeping labeled ones.
**Example:** `ore purge-backups src --older-than 1440 --dry-run`
**Can't do:** Only touches ore-style backup files — it won't touch arbitrary `*.bak` files you made by hand (safe by design).
# Editing & safety

> Every edit in this section creates a backup before writing (unless `--no-backup`), which the
> `history`/`undo` system can restore. Use `--dry-run` first when in doubt — it's cheap and shows
> exactly what would change.

### `ore backup`
**What it does:** Create a labeled backup of a file on demand.
**Syntax:** `ore backup <FILE>`
**Options:** `-l <label>` label suffix (e.g. `-l CAMFIX` → `file.ext.bakCAMFIX`) · `--list` list backups
**Use cases:** Snapshot before a manual experiment; create a named restore point the agent workflows use (`ai-fix` backs up before patching).
**Example:** `ore backup src/main.rs -l PRE-REFACTOR`
**Can't do:** Backs up the *current* content only — it's a snapshot, not version control.

### `ore restore`
**What it does:** Restore a file from its most recent (or labeled) backup.
**Syntax:** `ore restore <FILE>` · `-l <label>` pick a specific backup
**Use cases:** Undo a bad edit manually; roll back an agent's change; recover after `--no-backup` was *not* used.
**Example:** `ore restore src/main.rs -l PRE-REFACTOR`
**Can't do:** Only restores ore-made backups. `redo` will not re-apply a restored file's later edits.

### `ore patch`
**What it does:** Apply a literal find/replace to a file. **Defaults to exactly-one-match safety.**
**Syntax:** `ore patch [FILE] -f <FIND> -r <REPLACE>`
**Options:** `--patch-file <file>` (`.orepatch` format) · `--stdin` · `-a` replace all · `-n <nth>` Nth occurrence · `--first` · `--last` · `--no-backup` · `-l <label>` · `--dry-run`
**Use cases:** The safe surgical edit — fails loudly if the `find` text isn't unique (catches "the string appears twice" surprises); scripted multi-patch via `.orepatch` files.
**Example:** `ore patch src/a.ts -f "let x = 1" -r "let x = 2" --dry-run`
**Can't do:** Literal text only — for regex capture groups use `ore replace`. If `find` matches 0 or 2+ times it refuses (unless `-a`/`--first`/`--last`/`-n`).

### `ore patch-lines`
**What it does:** Replace an exact line or inclusive range with new text — no find-string needed.
**Syntax:** `ore patch-lines <FILE> <RANGE> [TEXT]` — ranges `N`, `N:M`, `N-M`. Omit or pass empty TEXT to delete the range.
**Options:** `--no-backup` · `-l <label>` · `--dry-run`
**Use cases:** Fix a known line range from a compiler error; delete a block by number — immune to "string appears twice" surprises.
**Example:** `ore patch-lines src/main.rs 83:86 "// rewritten"` … `ore patch-lines src/main.rs 40-45 ""` (delete)
**Can't do:** Line numbers only — no pattern matching (use `patch`/`patch-regex`). Empty TEXT deletes the range.

### `ore patch-insert`
**What it does:** Insert text before or after a specific line number.
**Syntax:** `ore patch-insert <FILE> <LINE> [TEXT]` — 0 = prepend to file; default inserts *after* the line
**Options:** `--before` · `--after` (default) · `--no-backup` · `-l <label>` · `--dry-run`
**Use cases:** Add an import at line 1; inject a config block after a known line; prepend a header to a file.
**Example:** `ore patch-insert src/app.ts 1 "import { z } from 'zod'"`
**Can't do:** Insert position is by line number, not pattern — use `before`/`after` for pattern-anchored inserts.

### `ore patch-preview`
**What it does:** Preview a patch as a unified diff without writing (exits 0 = find matched, 1 = not found).
**Syntax:** `ore patch-preview <FILE> -f <FIND> [-r <REPLACE>]`
**Options:** `-r` replacement (default "") · `-a` all · `-n <nth>` · `--first`/`--last` · `-C <context>` (default 3) · `--no-color` · `--literal`
**Use cases:** Check what a `patch` would do before committing to it; use the exit code as a "does this anchor exist?" gate in scripts.
**Example:** `ore patch-preview src/a.ts -f "const x = 1" -r "const x = 2"`
**Can't do:** Preview only — it never writes. Same exact-one-match default as `patch` (fails unless `-a`/`--first`/`--last`).

### `ore patch-regex`
**What it does:** Patch a file using a regular expression with capture-group replacement support.
**Syntax:** `ore patch-regex <FILE> -f <FIND> [-r <REPLACE>]`
**Options:** `-r` replacement (supports `$1`, `${name}`; default "") · `-a` all · `-n <nth>` · `--first`/`--last` · `-i` ignore case · `--preview` · `-C <context>` · `--no-backup` · `-l <label>` · `--dry-run` · `--literal`
**Use cases:** Regex swaps with capture groups that `patch` can't do; normalize patterns across a file with the backup safety net.
**Example:** `ore patch-regex src/legacy.ts -f "(\\w+)_(\\w+)" -r "$2_$1" -a`
**Can't do:** Rust regex syntax (no lookaround); exactly-one-match default — use `-a`/`--first`/`--last` for broader applies.

### `ore patch-batch`
**What it does:** Apply a `.orepatch` file with all-or-nothing (`--atomic`) and pre-flight (`--report`) support.
**Syntax:** `ore patch-batch <SOURCE>` — file path or `-` for stdin
**Options:** `--atomic` · `--report` · `--mode once|all|first|last` (default once) · `--stop-on-fail` · `--no-backup` · `-l <label>` · `--literal`
**Use cases:** Multi-file patch sets applied atomically — if any hunk fails, nothing writes; pre-flight check a patch file before running it.
**Example:** `ore patch-batch my.orepatch --report` then `ore patch-batch my.orepatch --atomic`
**Can't do:** Requires an ore-specific `.orepatch` file (see `ore patch --help` for the format); `--atomic` writes nothing unless every find matches.

### `ore replace`
**What it does:** Regex find/replace in a file, with capture-group support.
**Syntax:** `ore replace <PATTERN> <REPLACEMENT> <FILE>`
**Options:** `-F` literal · `-i` · `-w` · `-m` multiline · `-n <max>` first-N matches (0 = all) · `--no-backup -l --dry-run`
**Use cases:** Swap argument order with `$1/$2`; normalize a pattern across one file; regex normalization that `patch` can't do.
**Example:** `ore replace "(\w+)_(\w+)" "$2_$1" src/legacy.ts --dry-run`
**Can't do:** One file at a time — for project-wide use `replace-project`/`replace-ext`/`replace-dir`.

### `ore insert`
**What it does:** Insert text at a line number.
**Syntax:** `ore insert <FILE> <LINE> <TEXT>` · `--no-backup -l --dry-run`
**Use cases:** Add a missing import at line 1; inject a config line at a known position.
**Example:** `ore insert src/app.ts 1 "import { z } from 'zod'"`
**Can't do:** No "after last blank line" heuristics — you supply the exact line.

### `ore delete-lines`
**What it does:** Delete a line or range of lines.
**Syntax:** `ore delete-lines <FILE> <RANGE>` · `--no-backup -l --dry-run`
**Use cases:** Remove a dead function's line range; strip a block.
**Example:** `ore delete-lines src/main.rs 40-45`
**Can't do:** Range only — no pattern matching (use `replace-line` + regex or `before`/`after` cleverness, or `slice` for extraction).

### `ore replace-line`
**What it does:** Replace a specific line by number.
**Syntax:** `ore replace-line <FILE> <LINE> <TEXT>` · `--no-backup -l --dry-run`
**Use cases:** Fix a single known-bad line from a compiler error.
**Example:** `ore replace-line src/main.rs 12 "    let result: i32 = 0;"`
**Can't do:** One line only — for a range use `replace-range`.

### `ore replace-range`
**What it does:** Replace a range of lines with new text.
**Syntax:** `ore replace-range <FILE> <RANGE> <TEXT>` · `--no-backup -l --dry-run`
**Use cases:** Rewrite a function body (range) wholesale.
**Example:** `ore replace-range src/util.ts 30-55 "export function f() { return 42; }"`
**Can't do:** Exact lines — no "find the closing brace" logic.

### `ore before` / `ore after`
**What they do:** Insert text before / after the line matching a pattern.
**Syntax:** `ore before <FILE> <PATTERN> <TEXT>` · `ore after <FILE> <PATTERN> <TEXT>`
**Options:** `--first` only first match · `-F` literal · `-i` · `--no-backup -l --dry-run`
**Use cases:** "Insert a log line after every `return`"; "add a comment above `function main`".
**Example:** `ore after src/server.ts "app.listen" "console.log('listening');"`
**Can't do:** Inserts a *line* — not inline text on the same line.

### `ore surround`
**What it does:** Insert text before AND after a line range.
**Syntax:** `ore surround --before <BEFORE> --after <AFTER> <FILE> <RANGE>` (`\n` for multi-line)
**Use cases:** Wrap a code block in markers, a region in `#region`/`#endregion`, or a section in HTML comments.
**Example:** `ore surround -B "<!-- begin -->" -A "<!-- end -->" index.html 10-20`
**Can't do:** Both sides are required — for one-sided use `before`/`after`.

### `ore replace-project`
**What it does:** Regex replace across an entire project (all files), backing up each file.
**Syntax:** `ore replace-project <PATTERN> <REPLACEMENT> [PATH]`
**Options:** `-e <ext>` · `-x <exclude>` · `-F -i -w -m -H --no-ignore --binary` · `--dry-run` · `--no-backup`
**Use cases:** Rebrand a package name; rename an import specifier repo-wide; normalize an API call everywhere.
**Example:** `ore replace-project "from 'old-lib'" "from 'new-lib'" src -e ts,tsx --dry-run`
**Can't do:** It's a blind regex — it cannot understand code structure. Prefer `rename-symbol`/`rename-safe` for symbol renames (word-boundary, importer-aware) and always `--dry-run` first.

### `ore replace-ext`
**What it does:** Project-wide replace restricted to specific extensions.
**Syntax:** `ore replace-ext <PATTERN> <REPLACEMENT> <EXT> [PATH]`
**Options:** same family as `replace-project`, plus `-l <label>`
**Use cases:** Touch only `.md` files; update only `.ts` tests.
**Example:** `ore replace-ext "it\.skip" "it" test "spec.ts" --dry-run`
**Can't do:** One extension set per call (though comma lists work — e.g. `ts,tsx`).

### `ore replace-dir`
**What it does:** Project-wide replace restricted to one directory.
**Syntax:** `ore replace-dir <PATTERN> <REPLACEMENT> <DIR>` · `-e <ext>` · rest same family
**Use cases:** Scope a rename to `src/components` without touching `src/lib`.
**Example:** `ore replace-dir "Button" "FancyButton" src/components -e tsx --dry-run`
**Can't do:** Only that dir tree — for the whole project use `replace-project`.

### `ore patch-project`
**What it does:** Literal find/replace across a project (the project-wide cousin of `patch`).
**Syntax:** `ore patch-project --find <FIND> --replace <REPLACE> [PATH]`
**Options:** `-e -x -H --no-ignore --binary` · `-a` all (default) · `--exact-one` only files with exactly one match · `--dry-run --no-backup`
**Use cases:** Safe literal swap across files with the `--exact-one` guard for surgical migrations.
**Example:** `ore patch-project -f "v2" -r "v3" src -e ts --exact-one --dry-run`
**Can't do:** Literal only — regex users should use `replace-project`.

### `ore rename-bulk`
**What it does:** Rename files/directories by pattern.
**Syntax:** `ore rename-bulk <PATTERN> <REPLACEMENT> [PATH]`
**Options:** `-R` recursive · `-e <ext>` · `-x <exclude>` · `-H` · `--no-ignore` · `-i` · `--full-path` · `--dry-run`
**Use cases:** `kebab-case` → `snake_case` file renames; add/remove a suffix across a folder; rename test files.
**Example:** `ore rename-bulk "(.+)-test\.ts$" "$1.spec.ts" tests -R --dry-run`
**Can't do:** Renames files only — it does **not** update import references to those files (see `move-with-imports`/`rename-symbol`).

### `ore diff`
**What it does:** Diff two files, or a file against its backup.
**Syntax:** `ore diff <FILE_A> [FILE_B]` · or `ore diff <FILE> --backup` / `--label <L>`
**Options:** `-n` numbers · `-C <context>` · `-s` stats only
**Use cases:** "What did the agent change?" (`diff file --backup`); review a restore target before restoring; quick stats on a change.
**Example:** `ore diff src/main.rs --backup -s`
**Can't do:** Not a three-way merge (see `merge3`); whitespace/case-insensitive diffs need `diff-ignore`/`diff-semantic`.

### `ore encoding`
**What it does:** Inspect or convert a file's encoding.
**Syntax:** `ore encoding <FILE>` · `-t <encoding>` convert (utf-8, utf-16le, utf-16be, windows-1252…)
**Options:** `--no-backup` · `-l <label>` · `--bom` · `--strip-bom`
**Use cases:** Fix a mojibake file; convert a legacy Windows-1252 file to UTF-8; add/remove BOM.
**Example:** `ore encoding legacy.txt -t utf-8 --no-backup`
**Can't do:** Guessing is best-effort — very short or mixed files can mis-detect. It cannot *repair* corrupted byte sequences.

### `ore newlines`
**What it does:** Inspect or convert newline style.
**Syntax:** `ore newlines <FILE>` · `-t lf|crlf|cr` convert
**Use cases:** Normalize a repo to LF; fix a file that breaks on Windows tools expecting CRLF.
**Example:** `ore newlines script.sh -t lf`
**Can't do:** Whole-file only — no per-region mixing.
# Diffs & merges

### `ore diff-word`
**What it does:** Word-level (or character-level) diff between two files.
**Syntax:** `ore diff-word <FILE_A> <FILE_B>` · `-c` character-level
**Use cases:** See exactly which *words* changed in a long line (config values, prose edits, one-liners).
**Example:** `ore diff-word a.json b.json`
**Can't do:** Line-level hunks are better served by `ore diff`; no context controls.

### `ore diff-semantic`
**What it does:** Semantic diff — ignores whitespace and comments so only real code changes show.
**Syntax:** `ore diff-semantic <FILE_A> <FILE_B>` · `-v` show identical files too
**Use cases:** "Did the refactor actually change behavior?" — strips noise so a rename-with-reindent shows as clean.
**Example:** `ore diff-semantic before.ts after.ts`
**Can't do:** Comment stripping is `//`- and `#`-style only — block comments in some languages may not fully normalize.

### `ore diff-ignore`
**What it does:** Diff with configurable ignore flags (whitespace, blank lines, case, comments).
**Syntax:** `ore diff-ignore <FILE_A> <FILE_B>`
**Options:** `-w` whitespace · `-b` blank lines · `-i` case · `-c` comments · `-C <context>`
**Use cases:** CI-style "did anything actually change?" checks; reviewing generated files.
**Example:** `ore diff-ignore schema.gen.ts a.ts -w -b`
**Can't do:** Ignores combine additively; there's no "ignore only trailing whitespace" toggle (that's what `-w` covers wholly).

### `ore diff-dirs`
**What it does:** Diff two directory trees.
**Syntax:** `ore diff-dirs <DIR_A> <DIR_B>`
**Options:** `-e -x -H --no-ignore` · `-C` content-hash compare (exact) · `-v` include unchanged
**Use cases:** Compare two builds, two node_modules, a template vs its copy, or a backup tree.
**Example:** `ore diff-dirs build-old build-new -C`
**Can't do:** Size+mtime default can miss same-size same-time content changes — use `-C` for exactness (slower).

### `ore merge3`
**What it does:** Three-way merge (base + ours + theirs) with conflict markers.
**Syntax:** `ore merge3 <BASE> <OURS> <THEIRS>` (see `ore merge3 --help` for output/conflict options)
**Use cases:** Reconcile divergent edits against a shared ancestor; resolve a file when git's merge is too blunt.
**Example:** `ore merge3 base.txt ours.txt theirs.txt`
**Can't do:** No interactive conflict resolution UI — conflicts are emitted with markers for you to edit.

### `ore apply-patch`
**What it does:** Apply a `.patch`/`.diff` file (via `git apply`) with backups.
**Syntax:** `ore apply-patch <PATCH>`
**Options:** `-p <path>` apply within dir · `--no-backup` · `-l <label>` · `-R` reverse-apply · `--check` dry-run
**Use cases:** Apply a diff sent by a teammate; replay a patch from an issue; safely back out with `-R`.
**Example:** `ore apply-patch fix.diff --check`
**Can't do:** Depends on `git` being available (it shells out to `git apply`); patches that don't apply cleanly are rejected.

### `ore revert-patch`
**What it does:** Revert (reverse-apply) a `.patch`/`.diff` file.
**Syntax:** `ore revert-patch <PATCH>` · `-p <path>` · `--no-backup` · `-l <label>`
**Use cases:** Undo a previously applied patch — the explicit "un-apply".
**Example:** `ore revert-patch fix.diff`
**Can't do:** Reverses *exactly* what the patch says — if the file changed since, it may fail (regenerate the reverse patch in that case).

### `ore diff-summary`
**What it does:** English summary of what changed between two git refs.
**Syntax:** `ore diff-summary` · `-f <from>` (default `HEAD~5`) · `-t <to>` (default `HEAD`) · `-s simple|conventional`
**Use cases:** Standup notes; PR description draft; "what did last week's work touch?"
**Example:** `ore diff-summary -f HEAD~10 -t HEAD`
**Can't do:** Summarizes commit *stats and messages* — it doesn't read code semantics. Needs a git repo.
# Search

> `ore find` covers the single-pattern case. This section is the **compound search** family —
> multi-pattern, negative, multiline, fuzzy, and git-aware searches.

### `ore search-and`
**What it does:** Find files containing ALL of several patterns.
**Syntax:** `ore search-and [PATH] -p <PATTERN> -p <PATTERN> …`
**Options:** `-p` repeatable · `-F -i` · `-e -x -H --no-ignore` · `-l` files-only · `-v` which patterns matched
**Use cases:** "Files that import `zod` AND use `useState`" — narrowing to where two features intersect.
**Example:** `ore search-and src -p "zod" -p "useState" -l`
**Can't do:** All patterns must be in the *same file* (any line) — not same-line, not ordered.

### `ore search-or`
**What it does:** Find files containing ANY of several patterns.
**Syntax:** `ore search-or [PATH] -p <PATTERN> -p <PATTERN> …`
**Use cases:** "Where do we use any of the legacy APIs?" with a list of names.
**Example:** `ore search-or src -p "fooOld" -p "barOld" -p "bazOld" -l`
**Can't do:** This is effectively `find` with alternation — if you know regex, `ore find "fooOld|barOld"` is equivalent.

### `ore search-negative`
**What it does:** Find files that do NOT contain a pattern — optionally *while* requiring another.
**Syntax:** `ore search-negative <PATTERN> [PATH]`
**Options:** `-F -i -e -x -H --no-ignore` · `-r <REQUIRE>` also require this pattern · `-l` files-only
**Use cases:** "Files with `export default` but no `// @ts-nocheck`" (the `-r` combo); "test files that never mention the component name".
**Example:** `ore search-negative "@ts-nocheck" src -r "export default" -l`
**Can't do:** Negative-only search is the no-`-r` case; it scans content, not git status (see `search-changed`).

### `ore search-multiline`
**What it does:** Search for patterns spanning multiple lines.
**Syntax:** `ore search-multiline <PATTERN> [PATH]`
**Options:** `-i -e -x -H --no-ignore` · `-p` print the matched text · `--max-lines <N>` · `-l` files-only
**Use cases:** Find a missing-brace pattern, a function header + body signature, or a duplicated block.
**Example:** `ore search-multiline "if \(.*\)\s*\{" src -e ts -p`
**Can't do:** Slower than line-based search (reads whole files); very large binary-ish files are skipped.

### `ore search-fuzzy`
**What it does:** Typo-tolerant fuzzy search (filenames + content).
**Syntax:** `ore search-fuzzy <QUERY> [PATH]`
**Options:** `-d <distance>` max edit distance (default 2) · `-f` filenames only · `--min-token <n>` · `-e -x -H --no-ignore` · `-n <limit>` (default 50)
**Use cases:** You remember the name approximately ("the theming hook… `useTheme`? `ThemeProvider`?"); search filenames when you don't know exact casing.
**Example:** `ore search-fuzzy "themehook" src -f`
**Can't do:** Edit distance is per-token; longer queries mean fewer matches at the same distance. Fuzzy content search is approximate by nature — exact answers need `find`.

### `ore search-changed`
**What it does:** Search only in git-changed files (staged/unstaged/untracked filters).
**Syntax:** `ore search-changed <PATTERN>`
**Options:** `-F -i -w` · `--staged` · `--unstaged` · `--untracked` (combinable) · `-C <context>` · `-l` files-only
**Use cases:** "What in my uncommitted work mentions the new API?" — the code-review pre-flight.
**Example:** `ore search-changed "TODO" --unstaged --untracked`
**Can't do:** Needs a git repo; files that were deleted or renamed aren't searchable as content.

### `ore search-history`
**What it does:** Search across git history (pickaxe / regex).
**Syntax:** `ore search-history <QUERY>`
**Options:** `-p <path>` restrict · `-n <limit>` commits scanned (default 100) · `-d` show diff hunks · `-r` regex mode
**Use cases:** "When was this string introduced or removed?"; find the commit that deleted a feature.
**Example:** `ore search-history "deprecated_api" -d`
**Can't do:** Scans a bounded number of commits (default 100) — deep history needs `-n` raised. Requires a git repo with history.
# Git

> All git commands require being inside a git repository (a clear error is printed otherwise).
> ore shells out to your installed `git`.

### `ore git-status`
**What it does:** Working tree status.
**Syntax:** `ore git-status` · `-s` short format
**Use cases:** Quick "what's dirty" before committing; scriptable status for tooling.
**Example:** `ore git-status -s`
**Can't do:** Read-only view — staging happens via `git-stage`/`git commit`.

### `ore git-changed`
**What it does:** List changed files with powerful filters.
**Syntax:** `ore git-changed`
**Options:** `--only <sub>` · `--except <sub>` · `--starts <prefix>` · `--changed-in <dir>` · `--matching <content>` · `--staged/--unstaged/--untracked`
**Use cases:** "Which changed files mention the new API?" (`--matching`); "all changed files in src/components"; feed the list into other commands.
**Example:** `ore git-changed --staged --only ".ts"`
**Can't do:** It lists paths — it doesn't edit anything.

### `ore git-diff`
**What it does:** Show the git diff (staged/unstaged, per-file, or against a commit).
**Syntax:** `ore git-diff [FILE]`
**Options:** `-s` staged · `-c <commit>` · `--stat`
**Use cases:** Review your own work before committing; see a file's diff vs a commit.
**Example:** `ore git-diff -s --stat`
**Can't do:** Diff between two arbitrary refs is `git diff` territory (or `diff-summary` for the summary).

### `ore git-history`
**What it does:** Commit history for a file.
**Syntax:** `ore git-history <FILE>` · `-n <limit>` (default 20) · `-p` show patches
**Use cases:** "When did this file last change and why?" — the archaeology drill.
**Example:** `ore git-history src/main.rs -n 10 -p`
**Can't do:** File must exist in history — deleted files need `git log --all` style invocation.

### `ore git-blame`
**What it does:** Git blame with range support.
**Syntax:** `ore git-blame <FILE>` · `-L <range>` (`10`, `10-20`, `10:20`) · `-e` emails
**Use cases:** Who wrote line 42 — and when.
**Example:** `ore git-blame src/bug.ts -L 40-55`
**Can't do:** No ignore-rev filtering for formatting-only commits.

### `ore git-search`
**What it does:** Search git history by content or commit message.
**Syntax:** `ore git-search <QUERY>` · `--messages` · `--content` (default) · `-n <limit>`
**Use cases:** Find the commit that added a specific line; recall a commit by a message word.
**Example:** `ore git-search "rate limit" --messages`
**Can't do:** Bounded results (default 50); pickaxe-style content search needs `search-history` for more control.

### `ore git-who`
**What it does:** Contributors to a file, ranked by commits.
**Syntax:** `ore git-who <FILE>`
**Use cases:** Find the file's owner before a big refactor; credit in docs.
**Example:** `ore git-who src/router.ts`
**Can't do:** Simple counts — no per-author line/stat breakdown.

### `ore git-stage`
**What it does:** Stage files with filters.
**Syntax:** `ore git-stage` · `--all` · `--only/--except/--starts/--changed-in/--matching` · `--dry-run` · `-y`
**Use cases:** Stage only the `src` changes, or everything except `lock` files, without `git add` gymnastics.
**Example:** `ore git-stage --only "src" --dry-run`
**Can't do:** Staging only — committing is `git-commit`.

### `ore git-commit`
**What it does:** Commit files with filters (auto-stages matching, then commits).
**Syntax:** `ore git-commit -m <MESSAGE>`
**Options:** `--all` · `--only/--except/--starts/--changed-in/--matching` · `--dry-run`
**Use cases:** Commit a precisely-scoped subset of a messy tree in one step.
**Example:** `ore git-commit -m "fix: cache invalidation" --only "src/cache" --dry-run`
**Can't do:** For AI-generated messages use `git-auto-commit` or `ai-commit-message`.

### `ore git-log`
**What it does:** Git log with filters.
**Syntax:** `ore git-log`
**Options:** `-n <limit>` · `-g` graph · `--mine` · `--author <sub>` · `--grep <sub>` · `--since/--until <date>`
**Use cases:** "My commits this week"; "anything mentioning the ticket id"; a graph view of branch work.
**Example:** `ore git-log --mine --since "2 weeks ago" -n 30`
**Can't do:** One-shot listing — for change summaries between refs use `diff-summary`.

### `ore git-auto-commit`
**What it does:** Auto-generate + apply a commit message from the staged diff (heuristic, no LLM).
**Syntax:** `ore git-auto-commit`
**Options:** `-a` auto-stage all · `-p` preview only · `--conventional` · `--simple` · `-S` subject only · `-e` edit in `$EDITOR` · `-y` · `--only <sub>`
**Use cases:** Fast, LLM-free commit message generation; hook-friendly.
**Example:** `ore git-auto-commit -p`
**Can't do:** Heuristic English — for LLM-quality messages see `ai-commit-message`.

### `ore git-auto-message`
**What it does:** Generate a commit message from the diff without committing.
**Syntax:** `ore git-auto-message` · `-s` staged · `--conventional` · `--simple` · `-S` subject only
**Use cases:** Preview a message before committing; feed a message into your own commit tooling.
**Example:** `ore git-auto-message -s --conventional`
**Can't do:** Prints only — no `-e` editor flow (that's `git-auto-commit -e`).

### `ore git-suggest-commit`
**What it does:** Suggest a commit message and explain the rationale.
**Syntax:** `ore git-suggest-commit` · `-s` staged
**Use cases:** The "explain yourself" version — good for PR reviews and learning.
**Example:** `ore git-suggest-commit -s`
**Can't do:** Suggests only — never commits.

### `ore git-commit-body`
**What it does:** Compose a commit with your subject + a generated body.
**Syntax:** `ore git-commit-body <SUBJECT>` · `-u` unstaged · `-p` preview · `-y`
**Use cases:** You know the subject line; let ore write the details.
**Example:** `ore git-commit-body "feat: add retry to deploy" -p`
**Can't do:** Requires a subject argument — the subject is yours.

### `ore git-changelog`
**What it does:** Generate CHANGELOG markdown from git history.
**Syntax:** `ore git-changelog`
**Options:** `-s <since>` (`v1.0.0`, `HEAD~50`, `2 weeks ago`) · `-u <until>` · `-g` group by conventional type · `-o <file>` · `-H` hashes · `-a` authors
**Use cases:** Release notes drafts; keep a CHANGELOG without hand-writing it.
**Example:** `ore git-changelog -s v1.0.0 -g -o CHANGELOG.md`
**Can't do:** Groups by conventional-commit prefixes — un-typed commits land in an "other" bucket.

### `ore git-release-notes`
**What it does:** Generate release notes for a version.
**Syntax:** `ore git-release-notes <VERSION>` · `-p <previous>` (default: previous tag) · `-o <file>`
**Use cases:** Cut a release: notes for v2.1.0 since v2.0.0.
**Example:** `ore git-release-notes v2.1.0 -o notes.md`
**Can't do:** Requires tags for the `-p` default; no version-bumping logic.

### `ore git-undo-commit`
**What it does:** Undo the last N commits (soft/mixed/hard).
**Syntax:** `ore git-undo-commit` · `-n <count>` · `--hard` (loses changes!) · `--mixed` · `-y`
**Use cases:** Oops — committed the wrong thing. Default soft keeps changes staged.
**Example:** `ore git-undo-commit -n 2`
**Can't do:** `--hard` is destructive by design (with confirmation); this rewrites history locally — never push after a hard undo without thinking hard.

### `ore git-amend`
**What it does:** Amend the last commit.
**Syntax:** `ore git-amend` · `-m <message>` · `-n` include staged changes · `-y`
**Use cases:** Fix a typo in the last message; fold staged changes into the previous commit.
**Example:** `ore git-amend -m "feat: add retry (fix typo)" -y`
**Can't do:** Amending a pushed commit rewrites shared history — your responsibility.

### `ore git-fixup`
**What it does:** Create a fixup commit targeting a previous SHA, optionally autosquash.
**Syntax:** `ore git-fixup <TARGET>` · `-r` interactive autosquash rebase after
**Use cases:** The `git commit --fixup` workflow with a friendlier wrapper.
**Example:** `ore git-fixup abc1234 -r`
**Can't do:** The `-r` rebase is interactive (you still resolve conflicts); needs `GIT_SEQUENCE_EDITOR`/editor on PATH.

### `ore git-cleanup-branches`
**What it does:** Delete merged/orphaned local branches.
**Syntax:** `ore git-cleanup-branches`
**Options:** `-b <base>` (default main/master) · `--include-orphans` · `--force` (unmerged too) · `--dry-run` · `-y`
**Use cases:** Spring-cleaning a branch list with a preview first.
**Example:** `ore git-cleanup-branches --dry-run`
**Can't do:** Only local branches; `--force` deletes unmerged work (confirmation required) — check `--dry-run` first.

### `ore git-stash-named`
**What it does:** Named stash management (save/list/apply/pop/drop/show).
**Syntax:** `ore git-stash-named <COMMAND>` — e.g. `save <name>`, `list`, `apply <name>`, `pop <name>`, `drop <name>`, `show <name>`
**Use cases:** Context-switching with memorable stash names instead of `stash@{3}`.
**Example:** `ore git-stash-named save wip-auth-refactor` then `ore git-stash-named pop wip-auth-refactor`
**Can't do:** Wraps git stash — all of git stash's own constraints apply (untracked files need their own flag support).
# Process automation

> These commands orchestrate other commands — yours, ore's, or any executable on PATH. Command
> arguments are passed as a single quoted string and run through a shell.

### `ore run`
**What it does:** Run a command with capture/stream/silent options.
**Syntax:** `ore run <COMMAND>`
**Options:** `-s` stream live · `-q` silent · `--fail-on-error` exit non-zero on failure · `-v` timing summary · `-o <file>` stdout to file · `--err-output <file>`
**Use cases:** The disciplined wrapper — capture output, decide on failure, log stdout/stderr separately.
**Example:** `ore run "npm test" --fail-on-error -o test.log`
**Can't do:** By default it *captures* (doesn't stream) — long-running output looks stalled; add `-s`.

### `ore wait`
**What it does:** Wait for a condition (file, port, URL, command output, time).
**Syntax:** `ore wait` — pick one condition:
- `--file <FILE>` exists · `--file-missing <FILE>` · `--file-changed <FILE>` (mtime)
- `--port <PORT>` open (localhost) · `--port-closed <PORT>`
- `--time <SECS>` sleep
- `--command <CMD>` run until exit 0 · `--output-contains <CMD>` wait for its output to contain text

**Use cases:** The CI glue: wait for a server port, then run tests; wait for a file a build is producing; poll until a deploy command succeeds.
**Example:** `ore wait --port 8080` then `ore web-check http://localhost:8080`
**Can't do:** One condition per invocation (chain them with `&&`). No timeout option — it waits indefinitely, so wrap with `ore timer`/external timeout or design conditions to fail fast.

### `ore retry`
**What it does:** Retry a command until success (with backoff).
**Syntax:** `ore retry <COMMAND>`
**Options:** `-n <max>` attempts (default 5) · `-i <secs>` interval (default 1.0) · `-b <mult>` exponential backoff (default 1.0 = constant) · `-q` · `-s` stream
**Use cases:** Flaky network calls, slow-starting services, CI-style "try until it works".
**Example:** `ore retry "curl -sf http://localhost:3000/health" -n 10 -i 2 -b 2`
**Can't do:** No per-attempt jitter; success means exit 0 (a command that exits 0 but produces garbage still counts as success).

### `ore parallel`
**What it does:** Run multiple commands in parallel.
**Syntax:** `ore parallel <CMD1> <CMD2> …`
**Options:** `-l <limit>` max concurrency · `-s` stream interleaved · `-q` · `--fail-fast`
**Use cases:** Run tests on several projects at once; fetch/verify many things simultaneously with a cap.
**Example:** `ore parallel -l 4 "npm test -w a" "npm test -w b" "npm test -w c"`
**Can't do:** No per-command timeout (a hung command holds a slot); output interleaves with `-s` — use `-o`-style capture per command if you need clean logs.

### `ore sequence`
**What it does:** Run commands sequentially (stop or continue on fail).
**Syntax:** `ore sequence <CMD1> <CMD2> …`
**Options:** `-c` continue on error · `-s` stream · `-q`
**Use cases:** Pipelines with ordering guarantees and a defined failure policy.
**Example:** `ore sequence "ore verify" "ore git-auto-commit -y"`
**Can't do:** No conditional branching between steps (use `on-error`/`on-success` for that).

### `ore watch`
**What it does:** Watch a path and run a command when it changes.
**Syntax:** `ore watch <PATH> <COMMAND>`
**Options:** `-n` non-recursive · `-d <ms>` debounce (default 300) · `-e <ext>` filter · `-s` stream · `--initial` run once at start
**Use cases:** The TDD loop — re-run tests on every save; auto-rebuild on source changes.
**Example:** `ore watch src "cargo check" -e rs -s`
**Can't do:** File-watch semantics only — it won't detect content changes in files it can't read, and rapid-fire saves rely on the debounce. One path per invocation (multi-path → `watch-multi`).

### `ore watch-multi`
**What it does:** Watch multiple paths with different commands per path.
**Syntax:** `ore watch-multi -w "src=cargo check" -w "tests=npm test" …`
**Options:** `-d` debounce · `-e` ext · `-s` · `--initial` · `-n`
**Use cases:** One terminal, per-folder reactions: format on `src` changes, tests on `tests` changes.
**Example:** `ore watch-multi -w "src=ore format src" -w "tests=npm test"`
**Can't do:** Each `-w` is path=command — a change under `src` triggers only that command.

### `ore on-error` / `ore on-success`
**What they do:** Run a fallback / follow-up command based on the first command's exit code.
**Syntax:** `ore on-error --then <FALLBACK> <COMMAND>` · `ore on-success --then <FOLLOWUP> <COMMAND>`
**Use cases:** "Try the fast path; if it fails, fall back"; "deploy, and on success notify the team".
**Example:** `ore on-error --then "ore restore src/main.ts -y" "ore verify"`
**Can't do:** Binary on exit code only — no output-content conditions (that's `monitor --on-contains`).

### `ore monitor`
**What it does:** Long-running monitor of a command with alerts on change/error/text.
**Syntax:** `ore monitor <COMMAND>`
**Options:** `-i <secs>` interval (default 30) · `-n <count>` iterations (0 = forever) · `--on-change <cmd>` · `--on-error <cmd>` · `--on-contains <text>` · `--on-missing <text>` · `-v` show every poll
**Use cases:** Watch a health endpoint; alert when a log line appears; page someone when a build goes red.
**Example:** `ore monitor "curl -s localhost:3000/health" --on-error "ore notify 'server down!'" -i 10`
**Can't do:** It polls (interval-based), not event-driven — changes between polls are seen at the next tick. Default output is only on change (add `-v` to see every poll).

### `ore notify`
**What it does:** Send an OS notification.
**Syntax:** `ore notify <MESSAGE>` · `-t <title>` (default "ore") · `-e` also echo
**Use cases:** Long builds finishing; agent tasks completing; CI alerts on your desktop.
**Example:** `ore sequence "ore verify" "ore notify 'verify done'"`
**Can't do:** Desktop notifications only — no email/Slack/push (pipe to your own notifier if you need that).

### `ore schedule`
**What it does:** Windows Task Scheduler wrapper (create/list/rm/run).
**Syntax:** `ore schedule <COMMAND>` — check `ore schedule --help` for the create/list/rm/run verbs (Windows only)
**Use cases:** Schedule a nightly backup or a weekly cleanup without opening the Task Scheduler UI.
**Example:** `ore schedule create nightly-backup /tr "ore backup C:\data" /sc daily /st 02:00` (see `--help` for exact verbs/flags)
**Can't do:** **Windows-only** — it shells out to `schtasks`. On macOS/Linux it will fail; use your platform's cron/launchd.

### `ore timer`
**What it does:** Countdown timer with optional notification and follow-up command.
**Syntax:** `ore timer <DURATION>` (e.g. `25m`, `90s`)
**Options:** `-m <message>` · `-n` notify when done · `-c <command>` run when done · `-s` silent
**Use cases:** Pomodoro; "remind me to check the build in 5 minutes".
**Example:** `ore timer 5m -n -m "check the deploy"`
**Can't do:** Single-shot only — no recurring intervals (pair with `schedule` or `watch` for loops).

### `ore benchmark`
**What it does:** Benchmark a command (runs, min/mean/p50/p95/p99/max).
**Syntax:** `ore benchmark <COMMAND>`
**Options:** `-n <runs>` (default 10) · `-w <warmup>` (default 2) · `-v` per-run · `--strict` fail on any error
**Use cases:** Compare two implementations ("is the regex faster than the loop?"); track perf regressions.
**Example:** `ore benchmark "ore find 'fn main' src" -n 20 -w 3`
**Can't do:** Measures wall-clock of the whole command — JIT/caching effects are on you; `--strict` treats any non-zero run as failure.
# HTTP & networking

> All HTTP commands honor `-H <header>` (repeatable), `-t <timeout>` (seconds), and `--proxy` where
> shown. Nothing here persists cookies between calls — each request is independent (that's what
> the browser `web-*` commands are for).

### `ore fetch`
**What it does:** HTTP GET a URL (headers, output, pretty JSON).
**Syntax:** `ore fetch <URL>`
**Options:** `-H` headers · `-t` timeout (default 30) · `--no-redirect` · `--proxy` · `-o <file>` · `-i` include response headers · `-p` pretty-print JSON
**Use cases:** The curl replacement with pretty JSON built in; fetch an API endpoint and read it.
**Example:** `ore fetch https://api.github.com/repos/rust-lang/rust -p`
**Can't do:** No method/body (that's `post`); follows redirects by default — `--no-redirect` to stop.

### `ore post`
**What it does:** HTTP POST/PUT/PATCH/DELETE with body from string/file/JSON.
**Syntax:** `ore post <URL>`
**Options:** `-d <data>` raw body · `--file <file>` body from file · `-j <json>` JSON body · `-F "key=value"` form fields (repeatable) · `-X <method>` (default POST) · `-H` · `-t`
**Use cases:** Hit a webhook; create a resource; send form data.
**Example:** `ore post https://httpbin.org/post -j '{"name":"ore"}'`
**Can't do:** No automatic `Content-Type` guessing beyond what you pass via `-H`; multipart uploads need `upload`.

### `ore download`
**What it does:** Download a URL to a file.
**Syntax:** `ore download <URL>` · `-o <file>` · `--force` overwrite · `-H` · `-t` (default 300) · `--proxy` · `-y`
**Use cases:** Grab a release binary, an image set, a data dump.
**Example:** `ore download https://example.com/data.zip -o data.zip`
**Can't do:** No resume (see `resume-download`); no parallel batch (see `download-many`).

### `ore headers`
**What it does:** Show response headers only.
**Syntax:** `ore headers <URL>` · `-H` · `-t` (default 10) · `-g` GET instead of HEAD
**Use cases:** Debug caching headers; check content-type without downloading the body; verify a CDN is serving what you expect.
**Example:** `ore headers https://example.com -g`
**Can't do:** Headers only — no body. Some servers reject HEAD (`-g` helps).

### `ore status`
**What it does:** Show HTTP status code only.
**Syntax:** `ore status <URL>` · `-t` (default 10) · `-q` raw code only
**Use cases:** Scriptable health checks — "is the site up?" inside a `wait --command`.
**Example:** `ore status https://example.com -q` → `200`
**Can't do:** One URL per call — bulk checks are `check-urls`.

### `ore ping`
**What it does:** TCP ping (host:port reachability) — not ICMP.
**Syntax:** `ore ping <HOST>` · `-p <port>` (default 80) · `-n <count>` (default 4) · `-t <timeout>` · `-i <interval>`
**Use cases:** Is the port open? Latency to a service endpoint.
**Example:** `ore ping api.example.com -p 443 -n 6`
**Can't do:** **TCP only, not ICMP** — it can't detect packet loss the way `ping` does, and it doesn't resolve DNS failures in the same way (see `dns`).

### `ore dns`
**What it does:** DNS resolution.
**Syntax:** `ore dns <HOST>` · `-p <port>` (default 80)
**Use cases:** Check what a host resolves to; debug DNS split-brain.
**Example:** `ore dns example.com`
**Can't do:** No custom resolver / no `dig`-style record types (A records via system resolver only).

### `ore api-test`
**What it does:** Run API tests from a `.ore-api` spec file.
**Syntax:** `ore api-test <SPEC>` · `--fail-fast` · `-v` · `-t <timeout>`
**Use cases:** The smoke-test suite: define requests + expected status codes in a spec, run them in CI.
**Example:** `ore api-test api.ore-api`
**Can't do:** The spec format is ore's own (see the file format docs / `--help`); it validates responses, not full schema.

### `ore filesize`
**What it does:** Get remote file size (HEAD Content-Length) for one or many URLs.
**Syntax:** `ore filesize [URLS]...` · `-t` (default 10) · `-q` raw bytes one per line
**Use cases:** Size-check downloads before fetching; a scriptable "how big is this asset".
**Example:** `ore filesize https://example.com/big.zip -q`
**Can't do:** Depends on the server sending Content-Length on HEAD — chunked/compressed responses may report oddly.

### `ore upload`
**What it does:** Multipart file upload (with fields, headers).
**Syntax:** `ore upload --file "fieldname=path" <URL>`
**Options:** `-f "field=path"` repeatable · `-F "key=value"` extra fields · `-X <method>` · `-H` · `-t` (default 600) · `--proxy`
**Use cases:** Upload files to an API; send an attachment to a webhook.
**Example:** `ore upload -f "file=report.pdf" -F "title=Q3" https://uploads.example.com`
**Can't do:** No streaming progress UI — it's a one-shot multipart POST.

### `ore fetch-many`
**What it does:** Parallel HTTP GET of many URLs (rate-limit + save).
**Syntax:** `ore fetch-many [URLS]...` · `-f <file>` URL list · `-l <limit>` concurrency (default 5) · `-o <output-dir>` · `-v`
**Use cases:** Pull N API endpoints; prefetch a set of pages.
**Example:** `ore fetch-many -f urls.txt -l 8 -o out/`
**Can't do:** Saves to stdout/dir — it doesn't respect `robots.txt` or wait between domains (you set the limit).

### `ore download-many`
**What it does:** Parallel download of many URLs to a directory.
**Syntax:** `ore download-many [URLS]...` · `-f <file>` · `-o <dir>` (default `.`) · `--force` · `-l <limit>` (default 4)
**Use cases:** Grab an asset pack; mirror a list of files.
**Example:** `ore download-many -f assets.txt -o assets/ -l 6`
**Can't do:** No resume per file (see `resume-download`); filenames come from the URL unless a server sends Content-Disposition.

### `ore check-urls`
**What it does:** Bulk URL health checker (2xx/3xx/4xx/5xx).
**Syntax:** `ore check-urls [URLS]...` · `-f <file>` · `-l <limit>` (default 10) · `-t` · `--fallback-get` · `-F` failures only
**Use cases:** Link-rot audits; dead-link checks on a sitemap; verifying many routes before a release.
**Example:** `ore check-urls -f sitemap.txt -F`
**Can't do:** HEAD-based by default (some servers need `--fallback-get`); no content validation, status only.

### `ore resume-download`
**What it does:** Resumable download using HTTP Range.
**Syntax:** `ore resume-download <URL>` · `-o <file>` · `-H` · `-t` (default 600) · `--restart`
**Use cases:** Downloading big files over flaky connections — resumes from the partial file.
**Example:** `ore resume-download https://example.com/huge.iso -o huge.iso`
**Can't do:** Needs the server to support Range requests; with `--restart` the partial file is discarded.

### `ore bench-url`
**What it does:** Benchmark a URL (N reqs, concurrency, p50/p95/p99).
**Syntax:** `ore bench-url <URL>` · `-n <count>` (default 100) · `-c <concurrency>` (default 10) · `-X <method>` · `-t` · `--warmup <n>`
**Use cases:** Load-test an endpoint; compare perf before/after a deploy; spot latency tails.
**Example:** `ore bench-url https://myapi.com/ -n 500 -c 50`
**Can't do:** Single endpoint, no auth flows/session cookies — it's a raw hammer, not a load-testing suite.

### `ore ws`
**What it does:** WebSocket client (send/receive/listen).
**Syntax:** `ore ws <URL>` · `-m <message>` send one · `-n <count>` send N then close · `-r <count>` read N then exit · `--listen` read forever
**Use cases:** Smoke-test a WebSocket endpoint; send a probe message; tail a realtime feed.
**Example:** `ore ws wss://echo.example.com -m "hello" -r 1`
**Can't do:** No subprotocol negotiation flags; no reconnect logic — a dropped socket ends the session.

### `ore crawl`
**What it does:** Crawl a URL by following links (bounded depth + count).
**Syntax:** `ore crawl <URL>` · `-n <max>` pages (default 50) · `-d <depth>` (default 2) · `--same-domain` · `-o <output-dir>` save pages · `-t` · `-v`
**Use cases:** Site mapping; content inventory; feed a scraper.
**Example:** `ore crawl https://docs.example.com --same-domain -d 3 -o pages/`
**Can't do:** Respects basic link-following only — no JS rendering (use `web-*` for that), no robots.txt honoring, and it only follows `<a href>` links by default.
# Binary & hex

> Offsets/lengths accept `0x` hex and `k`/`m`/`g` suffixes. In-place byte edits create backups
> (unless `--no-backup`) and land in the history/undo safety net.

### `ore hex-view`
**What it does:** View a file as hex+ASCII (paged, with offset/length/width).
**Syntax:** `ore hex-view <FILE>` · `-o <offset>` · `-l <length>` (default 512) · `-w <width>` (default 16)
**Use cases:** Inspect a binary's header; look for structure in a data file; sanity-check an image.
**Example:** `ore hex-view logo.png -l 64`
**Can't do:** Read-only viewer — edits happen via the other `hex-*` commands. No incremental search within the view.

### `ore hex-find`
**What it does:** Find a hex pattern in a binary (with `??` wildcards).
**Syntax:** `ore hex-find <FILE> <PATTERN>` · `-C <context>` bytes (default 16) · `-n <max>` (0 = all) · `-o` offsets only
**Use cases:** Locate a magic number; find an instruction sequence; wildcard search in firmware.
**Example:** `ore hex-find game.exe "89 4? 24 ?? ?? 48" -n 10`
**Can't do:** Pattern must be hex (with `??` nibble wildcards) — no regex/text search in binaries (that's `strings` + `find`).

### `ore hex-replace`
**What it does:** Replace hex bytes (same-length in-place).
**Syntax:** `ore hex-replace <FILE> <FIND> <REPLACE>` · `-a` all · `-n <nth>` · `--no-backup -l --dry-run`
**Use cases:** Patch a version string in a binary; change a flag byte.
**Example:** `ore hex-replace app.bin "56 31" "56 32" -a --dry-run`
**Can't do:** **Same-length only** — if FIND and REPLACE differ in length it errors (that's `hex-patch` territory only for absolute offsets, or `hex-insert`/`hex-delete` for shifts).

### `ore hex-patch`
**What it does:** Write hex bytes at a specific offset (same-length or extend).
**Syntax:** `ore hex-patch <FILE> <OFFSET> <BYTES>` · `--extend` pad with zeros past EOF · `--no-backup -l --dry-run`
**Use cases:** Write at an exact offset; inject a short blob; extend a file with padding.
**Example:** `ore hex-patch data.bin 0x10 "DE AD BE EF"`
**Can't do:** Absolute-offset writes — it doesn't *find* a location (see `hex-find` + compute the offset).

### `ore hex-diff`
**What it does:** Binary diff with offset + hex dump.
**Syntax:** `ore hex-diff <FILE_A> <FILE_B>` · `-n <max>` (default 50) · `-C <context>` bytes
**Use cases:** Compare two binaries; see exactly where a corrupted file diverges.
**Example:** `ore hex-diff orig.bin corrupt.bin -C 8`
**Can't do:** Byte-level only — no semantic diff of embedded structures.

### `ore hex-extract`
**What it does:** Extract a byte range from a file.
**Syntax:** `ore hex-extract <FILE> <OFFSET> <LENGTH>` · `-o <output>` (else stdout as hex)
**Use cases:** Pull a chunk of a binary out to inspect or save; carve an embedded blob.
**Example:** `ore hex-extract game.bin 0x200 0x100 -o chunk.bin`
**Can't do:** Output defaults to hex text on stdout — pass `-o` for raw bytes.

### `ore hex-insert`
**What it does:** Insert bytes at an offset (existing bytes shift).
**Syntax:** `ore hex-insert <FILE> <OFFSET> <BYTES>` · `--no-backup -l`
**Use cases:** Add a record into a structured binary mid-file.
**Example:** `ore hex-insert data.bin 0x40 "00 01"`
**Can't do:** No validation that the insert keeps the file's internal structure valid — you know your format.

### `ore hex-delete`
**What it does:** Delete a byte range from a file (existing bytes shift).
**Syntax:** `ore hex-delete <FILE> <OFFSET> <LENGTH>` · `--no-backup -l`
**Use cases:** Strip a chunk out of a binary.
**Example:** `ore hex-delete data.bin 0x40 0x20`
**Can't do:** Same caveat as insert — structural validity is your responsibility.

### `ore strings`
**What it does:** Extract printable strings (ASCII + optional UTF-16).
**Syntax:** `ore strings <FILE>` · `-n <min>` length (default 4) · `-o` offsets · `-u` UTF-16 LE · `-m <max>` (0 = all)
**Use cases:** Peek inside a binary for readable hints; hunt for leaked paths or error messages.
**Example:** `ore strings app.exe -u -o | head -20`
**Can't do:** Only printable runs — compressed/encrypted content yields nothing (obviously).

### `ore magic`
**What it does:** Identify file type by magic bytes.
**Syntax:** `ore magic [FILES]...` · `-q` type name only
**Use cases:** "What kind of file is this mystery blob?"; verify a download is really the format it claims.
**Example:** `ore magic unknown.bin mystery.png -q`
**Can't do:** Signature-based — deliberately obfuscated files can fool it, and it doesn't validate structure (an image with a mangled body still says PNG).

### `ore bin-stats`
**What it does:** Byte frequency + entropy + histogram.
**Syntax:** `ore bin-stats <FILE>` · `-H` histogram (top 16)
**Use cases:** Detect compression/encryption (high entropy); find skew in structured files; spot padding regions.
**Example:** `ore bin-stats data.bin -H`
**Can't do:** Statistics only — no interpretation of *why* a file looks the way it does.

### `ore base64-encode`
**What it does:** Base64 encode (stdin or file).
**Syntax:** `ore base64-encode [FILE]` · `-o <output>` · `-u` URL-safe · `-w <wrap>` (0 = one line)
**Use cases:** Encode a binary for embedding; prepare data for a JSON payload.
**Example:** `ore base64-encode icon.png -u`
**Can't do:** No decode in the same command (use `base64-decode`).

### `ore base64-decode`
**What it does:** Base64 decode.
**Syntax:** `ore base64-decode [FILE]` · `-o <output>` (raw bytes) · `-u` URL-safe
**Use cases:** Decode a payload from an API response or email attachment.
**Example:** `ore base64-decode payload.txt -o payload.bin`
**Can't do:** Strict about input — malformed padding errors out rather than silently guessing.

### `ore xxd`
**What it does:** Raw xxd-style hex dump.
**Syntax:** `ore xxd <FILE>` · `-o <offset>` · `-l <length>` · `-w <width>` (default 16)
**Use cases:** Familiar xxd output in scripts; byte-exact dump for piping into other tools.
**Example:** `ore xxd header.bin -l 32`
**Can't do:** `hex-view` is the friendlier paged viewer; xxd is the raw classic.

### `ore bin-slice`
**What it does:** Extract a byte range to a new file.
**Syntax:** `ore bin-slice --output <OUTPUT> <FILE> <START> <END>` (`-o` required)
**Use cases:** Carve a region out for analysis without touching the original.
**Example:** `ore bin-slice rom.bin 0x1000 0x2000 -o part.bin`
**Can't do:** Requires an explicit output file — no stdout mode (that's `hex-extract`).

### `ore bin-cat`
**What it does:** Concatenate binary files.
**Syntax:** `ore bin-cat --output <OUTPUT> <FILES>...` (`-o` required)
**Use cases:** Join split parts; concatenate binary chunks into one file.
**Example:** `ore bin-cat part1.bin part2.bin part3.bin -o full.bin`
**Can't do:** Plain concatenation — no ordering metadata, no dedup, no verification of the result.
# Data formats

> Path syntax is the same across `*-get`/`*-set`: dot notation (`a.b.c`), brackets
> (`a[0].b`), and for JSON also full JSONPath. Edits (`*-set`, `*-fmt`) back up by default.

## JSON

### `ore json-get`
**What it does:** Get a value by dot/bracket path.
**Syntax:** `ore json-get <FILE> <PATH>` · `-p` pretty · `-j` always raw JSON
**Use cases:** Pull `package.json` fields in scripts; read a config value.
**Example:** `ore json-get package.json scripts.build`
**Can't do:** No JSONPath filtering — plain paths only (that's `json-query`).

### `ore json-set`
**What it does:** Set a value by path (creates intermediate objects).
**Syntax:** `ore json-set <FILE> <PATH> <VALUE>` · `-p` · `--no-backup -l --dry-run`
**Use cases:** Bump a version field; set a config flag in CI.
**Example:** `ore json-set package.json version 2.1.0`
**Can't do:** Values are parsed as JSON when possible (so `"true"` becomes boolean, `"42"` a number) — quote carefully if you truly need a string.

### `ore json-merge`
**What it does:** Deep-merge multiple files into a base.
**Syntax:** `ore json-merge <BASE> [OVERLAYS]...`
**Options:** `--replace-arrays` (default concatenates) · `-p` · `-o <output>` · `--no-backup -l`
**Use cases:** Layer environment-specific configs over a default; merge lock files.
**Example:** `ore json-merge default.json local.json env-ci.json -o merged.json`
**Can't do:** Objects merge deep; arrays concatenate unless `--replace-arrays` — know which you want.

### `ore json-fmt`
**What it does:** Format JSON (pretty/compact/sort-keys).
**Syntax:** `ore json-fmt <FILE>` · `-c` compact · `-s` sort keys · `-o <output>` · `--no-backup -l`
**Use cases:** Normalize a generated file; sort keys for stable diffs in CI.
**Example:** `ore json-fmt schema.json -s`
**Can't do:** It rewrites the file — comments and trailing commas in JSONC files will fail (they're not valid JSON).

### `ore json-query`
**What it does:** JSONPath query with `$.foo.bar[?(@.x>1)]` syntax.
**Syntax:** `ore json-query <FILE> <PATH>` · `-p`
**Use cases:** Filter a big API dump ("all items where price > 10").
**Example:** `ore json-query data.json "$.items[?(@.price > 10)].name"`
**Can't do:** JSONPath engine is a subset — exotic filters may not parse; verify with `json-keys` first.

### `ore json-keys`
**What it does:** List keys (flat or recursive with types).
**Syntax:** `ore json-keys <FILE> [PATH]` · `-t` types · `-r` full key tree
**Use cases:** Explore an unfamiliar API response; document a schema's shape.
**Example:** `ore json-keys data.json -r -t`
**Can't do:** Lists paths — it doesn't validate values or show them.

## YAML

### `ore yaml-get` / `ore yaml-set` / `ore yaml-fmt` / `ore yaml-to-json`
**What they do:** Get/set by path, reformat, or convert YAML to JSON.
**Syntax:** `ore yaml-get <FILE> <PATH> [-p]` · `ore yaml-set <FILE> <PATH> <VALUE> [--no-backup -l]` · `ore yaml-fmt <FILE> [-o]` · `ore yaml-to-json <FILE> [-o -c]`
**Use cases:** Edit a GitHub Actions workflow in CI; convert a k8s manifest to JSON for tooling; normalize a hand-edited YAML file.
**Example:** `ore yaml-set docker-compose.yml services.web.image "nginx:1.27"`
**Can't do:** YAML anchors/aliases are preserved on parse in most tools but `yaml-to-json` flattens them; comments are lost when rewriting via `yaml-set`/`yaml-fmt`.

## TOML

### `ore toml-get` / `ore toml-set` / `ore toml-fmt` / `ore toml-to-json`
**What they do:** Get/set by path, reformat, or convert TOML to JSON.
**Syntax:** `ore toml-get <FILE> <PATH> [-p]` · `ore toml-set <FILE> <PATH> <VALUE>` · `ore toml-fmt <FILE>` · `ore toml-to-json <FILE>`
**Use cases:** Edit `Cargo.toml`/`pyproject.toml` fields; bump a version in CI.
**Example:** `ore toml-set Cargo.toml package.version 0.3.0`
**Can't do:** TOML must be valid (tables, not inline-only quirks aside — the parser is strict); comments are lost on rewrite.

## CSV

### `ore csv-query`
**What it does:** Query a column with optional `--where` filters.
**Syntax:** `ore csv-query <FILE> <COLUMN>` · `-w "col=value"` repeatable · `--no-header` · `-d <delim>`
**Use cases:** "All rows where status=active — show me the email column."
**Example:** `ore csv-query users.csv email -w "plan=pro"`
**Can't do:** Exact-match filters only — no numeric comparisons or `LIKE` (that's `json-query`-style filtering, which CSV lacks).

### `ore csv-filter`
**What it does:** Filter rows by column=value.
**Syntax:** `ore csv-filter <FILE>` · `-w "col=value"` (all must match) · `--no-header -d` · `-o`
**Use cases:** Extract a subset of a CSV into its own file.
**Example:** `ore csv-filter sales.csv -w "region=EU" -o eu.csv`
**Can't do:** Exact matches, AND-combined only — no OR, no ranges.

### `ore csv-select`
**What it does:** Select a subset of columns.
**Syntax:** `ore csv-select <FILE> <COLUMNS>` · `--no-header -d -o`
**Use cases:** Strip a wide CSV down to the columns you need.
**Example:** `ore csv-select report.csv "id,name,total"`
**Can't do:** Column order follows your list (reordering allowed); duplicate names in headers are ambiguous.

### `ore csv-to-json`
**What it does:** Convert CSV to JSON (array of objects).
**Syntax:** `ore csv-to-json <FILE>` · `--no-header -d -o -c`
**Use cases:** Feed CSV data into JSON tooling.
**Example:** `ore csv-to-json users.csv -o users.json`
**Can't do:** Values stay strings (no type inference) — numbers won't become JSON numbers.

### `ore csv-stats`
**What it does:** Per-column stats (unique count, empties, numeric?).
**Syntax:** `ore csv-stats <FILE>` · `--no-header -d`
**Use cases:** Data quality check before processing a dump.
**Example:** `ore csv-stats dataset.csv`
**Can't do:** Summary only — no correlations, no histograms.

## .env

### `ore env-get`
**What it does:** Get a value by key (or list all).
**Syntax:** `ore env-get <FILE> [KEY]`
**Use cases:** Pull a key from `.env` in scripts.
**Example:** `ore env-get .env DATABASE_URL`
**Can't do:** No expansion of `${VAR}` references — you get the literal value.

### `ore env-set`
**What it does:** Set or delete a key.
**Syntax:** `ore env-set <FILE> <KEY> <VALUE>` · `--delete` · `--no-backup -l`
**Use cases:** Rotate a key in CI without manual edits.
**Example:** `ore env-set .env API_KEY newvalue123`
**Can't do:** Appends if missing, updates in place if present — comments near the key are preserved only for existing keys.

### `ore env-diff`
**What it does:** Diff two `.env` files.
**Syntax:** `ore env-diff <FILE_A> <FILE_B>` · `-D` only differing keys
**Use cases:** Compare local vs production env; spot a missing key before deploy.
**Example:** `ore env-diff .env .env.example -D`
**Can't do:** Key/value text comparison — it doesn't know which values are secrets.

## XML

### `ore xml-get`
**What it does:** Get element text or attribute value.
**Syntax:** `ore xml-get <FILE> <ELEMENT>` · `-a <attr>` print attribute value instead of text
**Use cases:** Pull a version from a pom.xml; read an SVG metadata attribute.
**Example:** `ore xml-get pom.xml project.version`
**Can't do:** First matching element only — no XPath expressions.

### `ore xml-fmt`
**What it does:** Reformat XML with indentation.
**Syntax:** `ore xml-fmt <FILE>` · `-w <width>` (default 2) · `-o` · `--no-backup -l`
**Use cases:** Normalize generated XML.
**Example:** `ore xml-fmt feed.xml -w 4`
**Can't do:** Must be well-formed XML — HTML-ish or malformed files fail.

### `ore xml-to-json`
**What it does:** Convert XML to JSON.
**Syntax:** `ore xml-to-json <FILE>` · `-o -c`
**Use cases:** Pipe XML APIs into JSON tooling.
**Example:** `ore xml-to-json data.xml -o data.json`
**Can't do:** Attributes/text/children mapping follows a fixed convention (see output) — complex schemas with mixed content can flatten oddly.
# Code analysis

> The analysis commands are regex/import-graph based, not full semantic analysis — they're fast
> and practical, but they can be fooled by exotic syntax. For TS/JS/Rust/Python they handle the
> common cases well.

## Symbol discovery

### `ore symbols`
**What it does:** List every named/exported symbol across a path (functions, classes, hooks, etc.).
**Syntax:** `ore symbols [PATH]`
**Options:** `-e -x` · `-k <kind>` (fn, class, hook, comp, const, type, enum, interface, struct, trait, mod) · `-E` exported only · `-n <name>` substring
**Use cases:** "What does this repo export?"; inventory before a refactor; find all React components.
**Example:** `ore symbols src -k comp -E`
**Can't do:** Regex-based — inline/obfuscated definitions can be missed; no type information.

### `ore outline`
**What it does:** Outline one file's structure with line numbers.
**Syntax:** `ore outline <FILE>` · `-E` exported only · `-j` JSON
**Use cases:** Get the shape of a file at a glance; plan where to add a symbol.
**Example:** `ore outline src/lib.ts -E`
**Can't do:** One file at a time (whole-tree is `symbols`).

### `ore snippet`
**What it does:** Extract a function/class/type by name from a file.
**Syntax:** `ore snippet <FILE> <SYMBOL>` · `-N` numbers · `-L` label header · `-o <output>`
**Use cases:** Grab one function's body for review or an AI prompt.
**Example:** `ore snippet src/util.ts parseDate -L`
**Can't do:** Assumes balanced braces — unusual syntax (template literals with braces) can mis-slice.

### `ore pluck`
**What it does:** Pluck exports/imports/types/interfaces/signatures from a file.
**Syntax:** `ore pluck <FILE>` — `--exports` · `--imports` · `--types` · `--interfaces` · `--signatures`
**Use cases:** "Show me only this file's imports"; extract type declarations for a quick audit.
**Example:** `ore pluck src/api.ts --exports`
**Can't do:** One category at a time (combine flags as needed); structural, not semantic.

## Reference hunting

### `ore refs`
**What it does:** Find every reference to a symbol across a path.
**Syntax:** `ore refs <SYMBOL> [PATH]`
**Options:** `-e -x` · `-C <context>` · `-l` files-only · `-D` include definition lines
**Use cases:** "Where is `parseDate` used?"; pre-rename impact survey.
**Example:** `ore refs parseDate src -l`
**Can't do:** Name-based matching — a local variable shadowing the same name shows up as a false positive; no type-aware resolution.

### `ore used-by`
**What it does:** List files that import from a given file.
**Syntax:** `ore used-by <FILE> [PATH]` · `-e -x` · `-n` show which named imports
**Use cases:** "If I delete this file, who breaks?"; find consumers of a module.
**Example:** `ore used-by src/db.ts -n`
**Can't do:** Static import analysis only — dynamic `require()`/`import()` with computed paths are missed.

### `ore imports-of`
**What it does:** Show what a file imports (with optional resolution).
**Syntax:** `ore imports-of <FILE>` · `-r` resolve relative imports to real files · `-j` JSON
**Use cases:** Understand a file's dependencies; verify a refactor didn't leave broken imports.
**Example:** `ore imports-of src/main.ts -r`
**Can't do:** Same static-import caveat — dynamic imports are best-effort.

### `ore neighbors`
**What it does:** Recursive dependency neighborhood around a file.
**Syntax:** `ore neighbors <FILE> [PATH]` · `-d <depth>` (default 2) · `-u` upstream · `-D` downstream (default true) · `-e -x`
**Use cases:** "What's the blast radius of this module?"; understand a file's context before editing.
**Example:** `ore neighbors src/store.ts -d 3`
**Can't do:** Follows static imports only; depth is bounded — very deep graphs are truncated at `-d`.

## Structural edits

### `ore add-import`
**What it does:** Add a named/default import to a file (merges with existing).
**Syntax:** `ore add-import --from <MODULE> <FILE>` · `-n <name>` named · `-D <default>` default · `--no-backup -l`
**Use cases:** Auto-add `useState` to a component; merge with an existing import line from the same module instead of duplicating.
**Example:** `ore add-import -n useState -s react src/App.tsx`
**Can't do:** Simple text insertion — import ordering/style follows the file's existing conventions only loosely.

### `ore remove-import`
**What it does:** Remove a named import or an entire import line.
**Syntax:** `ore remove-import <FILE>` · `-n <name>` · `-s <from>` remove whole line
**Use cases:** Clean up an unused import; strip a module import.
**Example:** `ore remove-import -n lodash src/util.ts`
**Can't do:** Removing a named import that's still used is on you — no usage checking.

### `ore split-file`
**What it does:** Split a multi-symbol file into per-symbol files (with optional barrel hub).
**Syntax:** `ore split-file <FILE>`
**Options:** `-o <output-dir>` · `-k` keep original as re-exporting hub · `-b fn|class|export|all` · `-e <ext>` · `-n kebab|exact`
**Use cases:** Break up a 2000-line `utils.ts` into per-symbol modules without breaking imports (`-k`).
**Example:** `ore split-file src/utils.ts -b export -k`
**Can't do:** Preserves exports, but intra-file references between the split symbols may need follow-up fixes — the hub keeps imports working, not the internals.

### `ore merge-files`
**What it does:** Merge multiple files into one (dedup imports, headers per file).
**Syntax:** `ore merge-files --output <OUTPUT> <FILES>...` · `-H` headers · `-d` dedup imports · `-s` skip empty · `--force`
**Use cases:** Consolidate a folder of tiny modules into one file.
**Example:** `ore merge-files -o src/merged.ts src/a.ts src/b.ts -d -H`
**Can't do:** Text concatenation with import dedup — symbol name collisions between files are NOT resolved.

### `ore extract-fn`
**What it does:** Extract a named function/class into a new file, optionally re-exporting from the source.
**Syntax:** `ore extract-fn --output <OUTPUT> <FILE> <SYMBOL>` · `-r` re-export · `-i` carry imports · `--no-backup -l`
**Use cases:** Pull one function into its own module and keep the old file working via re-export.
**Example:** `ore extract-fn src/util.ts parseDate -o src/date.ts -r -i`
**Can't do:** The function's own helpers/dependencies stay behind — you may need to carry more than the function body.

### `ore move-with-imports`
**What it does:** Move a file and update every importer's path.
**Syntax:** `ore move-with-imports <SRC> <DST>` · `-r <root>` scan root (default `.`) · `-e -x` · `--no-backup -l`
**Use cases:** The safe move: relocate a module and fix all `import './old'` references.
**Example:** `ore move-with-imports src/lib/old.ts src/lib/features/new.ts`
**Can't do:** Updates relative import specifiers — absolute/alias imports (e.g. `@/lib/old`) may need manual or `rename-symbol` treatment.

### `ore hub`
**What it does:** Create a barrel `index.ts` (or `mod.rs`/`__init__.py`) from a folder.
**Syntax:** `ore hub <DIR>` · `-o <output>` · `-E` exported only · `-s` star exports · `--force` · `--dry-run`
**Use cases:** Build a public API surface for a folder; make imports tidy.
**Example:** `ore hub src/features -E --dry-run`
**Can't do:** Re-exports what it finds statically — files with unusual export styles may be missed.

### `ore flatten-hub`
**What it does:** Inline all re-exports of a hub into a single file.
**Syntax:** `ore flatten-hub <HUB>` · `-i` carry imports · `--no-backup -l --dry-run`
**Use cases:** Collapse a barrel back into one file (the inverse of `split-file -k`).
**Example:** `ore flatten-hub src/index.ts --dry-run`
**Can't do:** Conflicts between re-exported names across the sources are not auto-resolved.

### `ore rename-symbol`
**What it does:** Rename a symbol across the codebase (word-boundary regex, all files).
**Syntax:** `ore rename-symbol <OLD> <NEW> [PATH]` · `-e -x` · `--no-backup -l --dry-run`
**Use cases:** Repo-wide rename of a function/component/variable.
**Example:** `ore rename-symbol parseDate toIsoDate src --dry-run`
**Can't do:** Word-boundary text rename — comments and string literals get renamed too; type-aware renaming doesn't exist (see `rename-safe` for verify+rollback).

### `ore organize`
**What it does:** Analyze and optionally reorganize top-level files into folders.
**Syntax:** `ore organize [PATH]` · `-b type|feature` · `-e -x` · `--apply` (default plan-only) · `--no-backup`
**Use cases:** Turn a flat `src/` with 80 files into a typed folder layout — plan first, then `--apply`.
**Example:** `ore organize src -b type --dry-run`
**Can't do:** Heuristic grouping — review the plan before `--apply`; imports inside moved files are updated, but exotic path references may break.

## Analysis

### `ore analyze-imports`
**What it does:** Import graph: fanout/fanin per file.
**Syntax:** `ore analyze-imports [PATH]` · `-s fanout|fanin|name` · `-n <top>` (default 20) · `-j`
**Use cases:** Find the most-imported (fanin) and most-importing (fanout) files — architecture red flags.
**Example:** `ore analyze-imports src -s fanin -n 15`
**Can't do:** Static analysis — circular/conditional imports appear only if statically resolvable.

### `ore analyze-exports`
**What it does:** Export counts per file.
**Syntax:** `ore analyze-exports [PATH]` · `-n <top>` (default 30) · `-j`
**Use cases:** Find "god files" exporting 50 symbols — refactor candidates.
**Example:** `ore analyze-exports src -n 10`
**Can't do:** Counts only — no usage analysis (that's `analyze-dead-exports`).

### `ore analyze-coupling`
**What it does:** Coupling score (fanout + fanin — most entangled files).
**Syntax:** `ore analyze-coupling [PATH]` · `-n <top>` (default 20)
**Use cases:** Spot the files that everything touches (and that touch everything) — refactor targets.
**Example:** `ore analyze-coupling src`
**Can't do:** Simple additive score — "entanglement" quality is a heuristic.

### `ore analyze-churn`
**What it does:** Files with the highest git churn.
**Syntax:** `ore analyze-churn` · `-p <path>` · `-s <since>` · `-n <top>` (default 20) · `-j`
**Use cases:** Which files change most often — candidates for refactoring or test coverage.
**Example:** `ore analyze-churn -s "6 months ago" -n 15`
**Can't do:** Git-based — needs a repo with history; rename-heavy repos may undercount.

### `ore analyze-hotspot`
**What it does:** Hotspot analysis (churn × complexity).
**Syntax:** `ore analyze-hotspot [PATH]` · `-e -x` · `-s <since>` · `-n <top>` (default 20)
**Use cases:** The files that are both complex AND frequently changed — where bugs live.
**Example:** `ore analyze-hotspot src -n 10`
**Can't do:** Complexity is a heuristic (nesting/branch counting) — cross-check with `analyze-complexity`.

### `ore analyze-complexity`
**What it does:** Cyclomatic complexity per function (above threshold).
**Syntax:** `ore analyze-complexity [PATH]` · `-t <threshold>` (default 10) · `-n <top>` (default 20) · `-j`
**Use cases:** Find functions that need decomposing.
**Example:** `ore analyze-complexity src -t 8`
**Can't do:** Heuristic cyclomatic counting — matches common control flow, exotic patterns may skew.

### `ore analyze-dead-exports`
**What it does:** Exported symbols never imported anywhere.
**Syntax:** `ore analyze-dead-exports [PATH]` · `-e -x` · `-k <keep>` entry-point patterns · `-n <top>` (default 50)
**Use cases:** Find dead code before a cleanup; pair with `trim-dead` to remove it.
**Example:** `ore analyze-dead-exports src -k index -k main`
**Can't do:** "Never imported" means *statically* — dynamic/string imports of an export will false-positive as dead.

### `ore analyze-circular`
**What it does:** Circular import detection.
**Syntax:** `ore analyze-circular [PATH]` · `-e -x` · `-n <top>` (default 20)
**Use cases:** Find import cycles that cause init-order bugs.
**Example:** `ore analyze-circular src`
**Can't do:** Static cycle detection — circularity through dynamic requires is missed.

### `ore analyze-type-coverage`
**What it does:** TS type coverage (any-density).
**Syntax:** `ore analyze-type-coverage [PATH]` · `-x` · `-n <top>` (default 20)
**Use cases:** Find the files with the most `any` — migrate-away targets.
**Example:** `ore analyze-type-coverage src`
**Can't do:** TypeScript only; counts `any`/`@ts-ignore`-ish patterns heuristically.

### `ore analyze-duplication`
**What it does:** Duplicated code blocks across files.
**Syntax:** `ore analyze-duplication [PATH]` · `-m <min-lines>` (default 6) · `-n <top>` (default 20)
**Use cases:** Copy-paste debt detection before a DRY pass.
**Example:** `ore analyze-duplication src -m 10`
**Can't do:** Exact/near-exact block matching — semantic duplicates with different names aren't found (see `consolidate`).

## Impact & dependency flow

### `ore impact`
**What it does:** Transitive impact if a file changes (upstream propagation).
**Syntax:** `ore impact <FILE> [ROOT]` · `-e -x` · `-d <depth>` (default 5)
**Use cases:** "If I touch `db.ts`, what tests will I likely need to update?"
**Example:** `ore impact src/db.ts`
**Can't do:** Direction: impact is upstream (who imports this, transitively) — downstream is `neighbors`/`route`.

### `ore trace`
**What it does:** Every call site of a function/method with context.
**Syntax:** `ore trace <NAME> [PATH]` · `-e -x` · `-C <context>` (default 1) · `-D` include defs
**Use cases:** Method-level usage map: where is `save()` called and how.
**Example:** `ore trace save src -C 2`
**Can't do:** Name-based — overloads and same-named methods in different classes all match.

### `ore blast-radius`
**What it does:** Transitive impact of changing a symbol (depth-based).
**Syntax:** `ore blast-radius <SYMBOL> [ROOT]` · `-e -x` · `-d <depth>` (default 3)
**Use cases:** Symbol-level version of `impact`: change `API_KEY`, who's affected at depth 2?
**Example:** `ore blast-radius API_KEY src -d 2`
**Can't do:** File-level granularity per hop — symbol-level precision is `trace`/`refs`.

### `ore related`
**What it does:** Files that "go together" with a given file (siblings + imports + git co-change).
**Syntax:** `ore related <FILE> [ROOT]` · `-e` · `-n <top>` (default 15)
**Use cases:** "What else should I look at while I'm in this file?" — the PR-buddy finder.
**Example:** `ore related src/Button.tsx`
**Can't do:** Heuristic combination — git co-change needs history; brand-new files rely on imports/siblings.

### `ore route`
**What it does:** Caller/callee tree for a file (upstream + downstream).
**Syntax:** `ore route <FILE> [ROOT]` · `-e` · `-d <depth>` (default 2)
**Use cases:** Visualize a module's position in the call graph.
**Example:** `ore route src/middleware/auth.ts -d 3`
**Can't do:** Static trees — runtime dispatch (interfaces, polymorphism) is approximated by name matching.

## Cleanup & safety

### `ore trim-dead`
**What it does:** Strip `export` keyword from unused exports (with backup + dry-run).
**Syntax:** `ore trim-dead [PATH]` · `-e -x` · `-k <keep>` preserve patterns · `--dry-run`
**Use cases:** The cleanup step after `analyze-dead-exports` — un-export unused symbols without deleting them.
**Example:** `ore trim-dead src --dry-run`
**Can't do:** Only removes the `export` keyword (the symbol stays, now module-private) — genuinely dead files need `rm`. String/dynamic import false positives apply as with `analyze-dead-exports`.

### `ore consolidate`
**What it does:** Find near-duplicate function bodies across the codebase.
**Syntax:** `ore consolidate [PATH]` · `-m <min-len>` chars (default 80) · `-s <similarity>` (default 0.85) · `-n <top>` (default 20)
**Use cases:** Find the two `formatDate` variants that should be one.
**Example:** `ore consolidate src -s 0.8`
**Can't do:** Reports only — deduplicating is a manual (or `ai-refactor`) job; similarity is text-based.

### `ore rename-safe`
**What it does:** Rename a symbol, run verify, auto-rollback on failure.
**Syntax:** `ore rename-safe <OLD> <NEW> [PATH]` · `-e -x` · `-v <verify-command>` (auto-detect tsc/cargo) · `-y` · `--dry-run`
**Use cases:** The paranoid rename: it renames, compiles/typechecks, and restores from backup if verification fails.
**Example:** `ore rename-safe parseDate toIsoDate src -y`
**Can't do:** Verification depends on a working `tsc`/`cargo` (or your `-v` command); if the project was already broken, rollback may trigger even for a good rename.

## Change & time analysis

### `ore since`
**What it does:** Everything that changed since a date or ref.
**Syntax:** `ore since <WHEN>` (e.g. `"2 weeks ago"`, `v1.0.0`) · `-s` diff stats · `-p <path>`
**Use cases:** "What changed since last Friday?" — standup prep.
**Example:** `ore since "3 days ago" -s`
**Can't do:** Git-based; `WHEN` parsing is natural-language-ish — stick to git's accepted date formats.

### `ore hot-files`
**What it does:** Files with the highest git churn (hotspots for refactoring).
**Syntax:** `ore hot-files` · `-s <since>` (default "90 days ago") · `-p` · `-n <top>` (default 20)
**Use cases:** Where the team spends its time — coverage/refactor priorities.
**Example:** `ore hot-files -s "6 months ago"`
**Can't do:** Same git-history caveats as `analyze-churn` (they're siblings).

### `ore stale-files`
**What it does:** Files nobody has touched in a long time.
**Syntax:** `ore stale-files` · `-o <older-than>` (default "180 days ago") · `-p` · `-n <top>` (default 50)
**Use cases:** Deletion/archival candidates; "does anyone still use this?"
**Example:** `ore stale-files -o "1 year ago"`
**Can't do:** Git-mtime based — files with no history at all may not appear as expected.

## AI context builders

### `ore explain`
**What it does:** Heuristic English explanation of what a file does (no LLM).
**Syntax:** `ore explain <FILE>`
**Use cases:** Instant, free "what is this file" — when you don't want to spend tokens on `ai-explain`.
**Example:** `ore explain src/middleware.ts`
**Can't do:** Heuristic, not semantic — quality is far below `ai-explain` for nuanced code.

### `ore digest`
**What it does:** Codebase digest for AI (structural summary, per-file exports/imports).
**Syntax:** `ore digest [PATH]` · `-e -x` · `-o <output>` · `--with-imports` · `--with-tree` · `--with-stats`
**Use cases:** The context block for an LLM: "here's the shape of the repo, answer my question" (`ai-explain "question"` does this automatically).
**Example:** `ore digest src --with-imports -o digest.md`
**Can't do:** Structural summary — no file contents (that's `pack`).

### `ore condense`
**What it does:** Condense a file (strip comments/blanks/whitespace to save tokens).
**Syntax:** `ore condense <FILE>` · `-l light|medium|aggressive` · `-o <output>`
**Use cases:** Shrink a file before sending it to an LLM; `ai-explain`/`ai-review` auto-condense large files this way.
**Example:** `ore condense src/main.rs -l medium`
**Can't do:** Loses comments — don't use the output as the canonical file; it's a token-saving view.

### `ore chunk`
**What it does:** Split a file into per-function/class/section chunks with a manifest.
**Syntax:** `ore chunk <FILE>` · `-b function|class|export|section` · `-o <dir>` · `--manifest` (chunks.json) · `--dry-run`
**Use cases:** Build RAG-style chunks of a codebase for embedding/indexing.
**Example:** `ore chunk src/lib.ts -b function --manifest`
**Can't do:** Text segmentation — chunks are standalone views; cross-chunk references aren't resolved.

### `ore ai-prompt`
**What it does:** Build a task-focused AI prompt (finds relevant files, packs them).
**Syntax:** `ore ai-prompt <TASK> [PATH]` · `-e -x` · `-n <max-files>` (default 12) · `-o <output>` · `--copy` · `--with-digest`
**Use cases:** Assemble "here's my task + the relevant files" in one paste-able block.
**Example:** `ore ai-prompt "fix the auth bug" src -n 8 --copy`
**Can't do:** Relevance = filename/content heuristics — the best files may not be picked; edit the result before sending.
# Reports & compile/verify

> All `report-*` commands emit **Markdown** (to stdout or `-o <file>`) — they're for humans and
> for pasting into issues/PRs/docs. `workspace-report` combines several of them into one file.

## Reports

### `ore report-health`
**What it does:** Report: codebase health as markdown.
**Syntax:** `ore report-health [PATH]` · `-o <file>` · `-e <ext>`
**Use cases:** A README-able health snapshot; the doc for "how healthy is this repo".
**Example:** `ore report-health src -o health.md`
**Can't do:** Health score is heuristic (todos, smells, meta files) — not a static-analysis guarantee.

### `ore report-todos`
**What it does:** Report: all TODO/FIXME/HACK comments as markdown.
**Syntax:** `ore report-todos [PATH]` · `-e -x -o`
**Use cases:** Technical-debt board; pre-sprint review of loose ends.
**Example:** `ore report-todos src -o todos.md`
**Can't do:** Comment-style detection covers common forms (`//`, `#`, `<!-- -->`) — exotic comment syntax may be missed.

### `ore report-imports`
**What it does:** Report: import graph as markdown.
**Syntax:** `ore report-imports [PATH]` · `-e -x -o` · `-n <top>` (default 30)
**Use cases:** Architecture documentation; the "who imports what" appendix.
**Example:** `ore report-imports src -o imports.md`
**Can't do:** Static import graph — dynamic imports are best-effort.

### `ore report-api`
**What it does:** Report: public API surface as markdown.
**Syntax:** `ore report-api [PATH]` · `-e -x -o`
**Use cases:** Generated API docs for a library; a public-surface review.
**Example:** `ore report-api src -o api.md`
**Can't do:** "Public" = exported symbols heuristically — JSDoc/README text isn't included.

### `ore report-contributors`
**What it does:** Report: git contributors as markdown.
**Syntax:** `ore report-contributors` · `-s <since>` · `-o`
**Use cases:** Release-notes attribution; team metrics.
**Example:** `ore report-contributors -s "1 year ago" -o contributors.md`
**Can't do:** Git-author based — name/email variations can split one person.

### `ore report-coverage`
**What it does:** Report: structural test coverage as markdown.
**Syntax:** `ore report-coverage [PATH]` · `-e -x -o`
**Use cases:** "Which files have no test sibling?" — the coverage *gaps* view.
**Example:** `ore report-coverage src -o coverage.md`
**Can't do:** Structural only (file↔test-file pairing) — it is **not** line/branch coverage from a coverage tool.

### `ore report-changes`
**What it does:** Report: recent git changes as markdown.
**Syntax:** `ore report-changes` · `-s <since>` · `-u <until>` · `-o`
**Use cases:** The "what happened this sprint" summary for a team doc.
**Example:** `ore report-changes -s "2 weeks ago" -o changes.md`
**Can't do:** Git-based; uncommitted work isn't included.

### `ore report-errors`
**What it does:** Report: last cached compile errors as markdown.
**Syntax:** `ore report-errors` · `-o`
**Use cases:** Paste a formatted error digest into an issue.
**Example:** `ore report-errors -o errors.md`
**Can't do:** Shows the last cached errors from `compile-*` — run a compile first; it doesn't compile for you.

### `ore workspace-report`
**What it does:** Full workspace snapshot (health + structure + git + analysis) as markdown.
**Syntax:** `ore workspace-report [PATH]` · `-o <file>` (default `workspace-report.md`) · `-e`
**Use cases:** The one-file onboarding doc for a repo; a complete snapshot before a big refactor.
**Example:** `ore workspace-report -o report.md`
**Can't do:** It's a snapshot of heuristics — combine with `ai-explain`/`ai-review` for interpretation.

## Compile

### `ore compile-ts`
**What it does:** Run `tsc --noEmit`, parse errors, cache them.
**Syntax:** `ore compile-ts [PATH]` · `-a <args>` · `-s` stream · `-j` JSON parsed errors
**Use cases:** Typecheck without emitting; get clean structured errors; feed `errors-last`/`report-errors`.
**Example:** `ore compile-ts src -j`
**Can't do:** Needs `tsc` on PATH (or the project's local TypeScript); `--noEmit` only — no build output.

### `ore compile-rust`
**What it does:** Run `cargo check`/`build`, parse errors, cache them.
**Syntax:** `ore compile-rust [PATH]` · `-c` check (fast) · `-a <args>` · `-s` · `-j`
**Use cases:** The fast Rust check loop; structured error output.
**Example:** `ore compile-rust -c -j`
**Can't do:** Requires cargo + the project to be a Cargo workspace; first build is slow (caching is cargo's job).

### `ore compile-node`
**What it does:** Run an npm/yarn/pnpm script and cache output.
**Syntax:** `ore compile-node [PATH]` · `-r <script>` (default `build`) · `--pm npm|yarn|pnpm` · `-s`
**Use cases:** Run the project's build script with error caching.
**Example:** `ore compile-node --pm pnpm -r typecheck`
**Can't do:** Runs one script — parallel scripts are your orchestrator's job.

### `ore errors-last`
**What it does:** Replay the last cached compile errors (grouped/filtered/JSON).
**Syntax:** `ore errors-last` · `-w` warnings too · `-g` group by file · `-r` raw · `-j` JSON · `-f <file>` filter
**Use cases:** Re-show yesterday's errors after a recompile; the "what was broken" query.
**Example:** `ore errors-last -g -f src/main`
**Can't do:** Cache is per-workspace and overwritten by the next `compile-*` run.

## Verify

### `ore verify`
**What it does:** Run typecheck + lint + tests in sequence.
**Syntax:** `ore verify [PATH]` · `-t auto|ts|rust|node` · `--no-test` · `--no-lint`
**Use cases:** The pre-commit/pre-PR gate — one command for the whole check suite.
**Example:** `ore verify . --no-lint`
**Can't do:** Auto-detects the project type; a project with no recognized toolchain reports "nothing to verify" rather than failing.

### `ore verify-anchor`
**What it does:** Check that an exact text anchor exists in a file before patching (exits 0 = found, 1 = not found).
**Syntax:** `ore verify-anchor <FILE> -f <FIND>`
**Options:** `-q` quiet (exit code only) · `-c` print match count · `-n` print first match line number · `-i` ignore case · `-x` regex
**Use cases:** The pre-patch guard in scripts and agent workflows — confirm an anchor exists (and where) before a `patch`/`patch-lines` that depends on it.
**Example:** `ore verify-anchor src/main.rs -f "fn run" -n`
**Can't do:** Verifies presence only — it doesn't verify uniqueness (use `patch --dry-run` for that).

### `ore health`
**What it does:** Codebase health report (score, todos, code smells, meta files).
**Syntax:** `ore health [PATH]` · `-e -x` · `-j` JSON
**Use cases:** Quick numeric health score in CI or before a refactor.
**Example:** `ore health src -j`
**Can't do:** Same heuristic caveat as `report-health` (they're siblings in different output formats).

### `ore verify-json`
**What it does:** Validate one or more JSON files.
**Syntax:** `ore verify-json [FILES]...` · `-f` format info · `-L` lenient (JSON5 comments/trailing commas — tsconfig-friendly)
**Use cases:** CI gate on config files; check a generated JSON before committing.
**Example:** `ore verify-json package.json tsconfig.json -L`
**Can't do:** Lenient mode accepts JSON5-ish input — it doesn't *convert* it to strict JSON.

### `ore verify-syntax`
**What it does:** Basic syntax check (JSON, TOML, brace-balance for code).
**Syntax:** `ore verify-syntax [FILES]...`
**Use cases:** Quick sanity check on config + code files before a commit.
**Example:** `ore verify-syntax Cargo.toml main.rs`
**Can't do:** Brace-balance is shallow — it won't catch semantic errors, only gross imbalance.

### `ore verify-encoding`
**What it does:** Validate UTF-8 encoding of one or more files.
**Syntax:** `ore verify-encoding [FILES]...` · `-b` flag BOM as a warning
**Use cases:** Catch mojibake before it hits production.
**Example:** `ore verify-encoding src/**/*.ts`
**Can't do:** UTF-8 validity only — valid-but-wrong (e.g. Latin-1 interpreted as UTF-8) passes silently.

### `ore verify-imports`
**What it does:** Verify relative imports resolve (JS/TS).
**Syntax:** `ore verify-imports [FILES]...` · `-r` resolve with common extensions (`/index.ts`, `.tsx`, …)
**Use cases:** Post-refactor check — "did every import survive the move?"
**Example:** `ore verify-imports src -r`
**Can't do:** Static resolution — extensionless/alias imports need `-r` or manual config; absolute aliases aren't resolved.
# Scaffolding, tooling & workspace

## Scaffolding

### `ore scaffold`
**What it does:** Scaffold a new project from a template.
**Syntax:** `ore scaffold <TEMPLATE> <NAME>` · `-p <parent>` · `--pm npm|yarn|pnpm` · `--no-install` · `--no-git` · `--dry-run`
**Use cases:** Spin up a project skeleton (see `ore scaffold --help` for available templates) without waiting for interactive CLIs.
**Example:** `ore scaffold react-app my-app --no-install`
**Can't do:** Templates are a fixed set — custom templates need the `template` command; installs run package managers which need network.

### `ore scaffold-add`
**What it does:** Add a feature (tailwind, zustand, prettier, eslint, etc.) to a project.
**Syntax:** `ore scaffold-add <FEATURE> [DIR]` · `--pm` · `--no-install`
**Use cases:** Add a tool without the interactive `npx` prompt dance.
**Example:** `ore scaffold-add tailwind --no-install`
**Can't do:** Supports the feature list baked into the command — check `--help` for the exact set.

### `ore scaffold-component` / `ore scaffold-hook` / `ore scaffold-store` / `ore scaffold-context` / `ore scaffold-api`
**What they do:** Scaffold a React component / hook / Zustand store / context+provider+hook / REST API client module.
**Syntax:** `ore scaffold-component <NAME> [-o src/components --with-css --with-test]` · `scaffold-hook <NAME> [-o src/hooks]` · `scaffold-store <NAME> [-o src/store]` · `scaffold-context <NAME> [-o src/contexts]` · `scaffold-api <NAME> [-u /api -o src/lib/api]`
**Use cases:** Consistent, fast React scaffolding with your conventions baked in.
**Example:** `ore scaffold-component Button --with-css --with-test`
**Can't do:** Templates are conventions, not generators tied to your framework version — generated code may need tweaks for React 18/19 specifics.

### `ore scaffold-test`
**What it does:** Scaffold a test file for an existing source file.
**Syntax:** `ore scaffold-test <FILE>` · `-f <framework>` (default vitest)
**Use cases:** Create the matching `.test.ts` skeleton for a module.
**Example:** `ore scaffold-test src/util.ts -f jest`
**Can't do:** Generates an empty-ish skeleton — it doesn't know your test cases.

## Tooling

### `ore setup`
**What it does:** Verify a toolchain is installed (rust/node/git/python/env).
**Syntax:** `ore setup <TOOL>`
**Use cases:** Onboarding check — "is this machine ready for the repo?"
**Example:** `ore setup node`
**Can't do:** Checks PATH/versions only — it won't install anything (that's `install-if-missing`).

### `ore check-deps`
**What it does:** Check that a set of tools are available on PATH.
**Syntax:** `ore check-deps` · `-t <tools>` comma list (default common ones)
**Use cases:** Pre-flight in CI scripts.
**Example:** `ore check-deps -t node,git,rustc`
**Can't do:** PATH presence only — no version minimums.

### `ore install-if-missing`
**What it does:** Install missing tools via winget/choco/npm/cargo/scoop.
**Syntax:** `ore install-if-missing <TOOLS>` · `-s winget|choco|npm|cargo|scoop` (default winget) · `-y`
**Use cases:** One-command dev environment bootstrap on Windows.
**Example:** `ore install-if-missing node git -y`
**Can't do:** **Windows-centric** (default winget) — other platforms need the right `-s`; installs are best-effort and depend on the package source being available. Any install changes your system — review before running.

### `ore snip`
**What it does:** Snippet manager (save/load/list/copy/find/export/import).
**Syntax:** `ore snip <COMMAND>` — save/load/list/copy/find/export/import subcommands
**Use cases:** Your personal snippet library, searchable and copyable.
**Example:** `ore snip save sql-upsert "INSERT ... ON CONFLICT ..."` then `ore snip copy sql-upsert`
**Can't do:** Local file-backed storage — no sync/cloud.

### `ore template`
**What it does:** Template manager with variable interpolation (`{{var}}`).
**Syntax:** `ore template <COMMAND>` — create/render/list/rm subcommands
**Use cases:** Reusable file templates (configs, boilerplate) with placeholders.
**Example:** `ore template render my-template -v name=app`
**Can't do:** Simple `{{var}}` interpolation — no loops/conditionals in templates.

### `ore macro`
**What it does:** Macro manager (save/run/list — sequence of commands).
**Syntax:** `ore macro <COMMAND>` — save/run/list subcommands
**Use cases:** "release" = verify → test → git-auto-commit → tag; run it with one word.
**Example:** `ore macro save deploy "ore verify && ore notify 'done'"`
**Can't do:** Runs command strings through a shell — macro safety is on you.

## Session & focus

### `ore session`
**What it does:** Session tracking (start/end/log/notes).
**Syntax:** `ore session <COMMAND>` — start/end/log/notes subcommands
**Use cases:** Time-boxed work sessions with a log of what you did.
**Example:** `ore session start "fix auth bug"` … `ore session end`
**Can't do:** Local tracking — no timesheet export or syncing.

### `ore focus`
**What it does:** Set/show/clear a focus path for the workspace.
**Syntax:** `ore focus <COMMAND>` — set/show/clear
**Use cases:** Scope searches/analyses to a subdirectory without typing it everywhere (commands that respect focus).
**Example:** `ore focus set src/components`
**Can't do:** Not every command reads the focus path — check each command's behavior; it's a convenience, not a sandbox.

## Project memory

### `ore notes`
**What it does:** Persistent key-value notes for the project, stored across sessions.
**Syntax:** `ore notes <COMMAND>` — set/get/rm/list/clear/search
**Options:** `--dir <DIR>` (default: current dir)
**Use cases:** Project memory that survives restarts — architecture decisions, gotchas, API notes; `session-export` can fold them into a handoff.
**Example:** `ore notes set "build" "cargo build --release then copy ore.exe"` … `ore notes search "build"`
**Can't do:** Per-workspace storage (keyed to the working directory), not global; plain key-value — no nesting or rich formatting.

### `ore bookmark`
**What it does:** Named file:line references for quick navigation.
**Syntax:** `ore bookmark <COMMAND>` — set/get/rm/list/jump/clear
**Options:** `set <name> <file:line> [-m "<memo>"]` · `--dir <DIR>`
**Use cases:** Keep a named index of the important spots in a codebase — `jump` prints the content around the bookmarked line.
**Example:** `ore bookmark set patch-engine "src/engine/patch.rs:10" -m "main entry"` … `ore bookmark jump patch-engine`
**Can't do:** Stores the *path:line* — if the file moves, the bookmark still points at the same line number, not the content.

### `ore tag`
**What it does:** Tag files with labels (read, patched, reviewed…) for session tracking.
**Syntax:** `ore tag <COMMAND>` — add/rm/get/files/list/clear-file/clear-all/summary
**Options:** `--dir <DIR>`
**Use cases:** Track which files you've reviewed or patched in a session; find every file carrying a tag.
**Example:** `ore tag add src/main.rs read patched` … `ore tag files patched`
**Can't do:** Tags live in the workspace store — a local tracking aid, not git metadata (tags don't travel with commits).

### `ore session-export`
**What it does:** Export a session handoff document — git status, notes, history, modified files.
**Syntax:** `ore session-export` · `-o <FILE>` (default stdout)
**Options:** `--git` include git status · `--notes` include notes · `--limit <N>` history entries (default 50) · `--dir <DIR>`
**Use cases:** End-of-day handoff; hand a teammate (or an LLM) everything they need to pick up your work.
**Example:** `ore session-export -o handoff.md --git --notes`
**Can't do:** Summarizes recorded state (history/notes/git) — it can't capture unrecorded work in progress.

## Locks

### `ore lock` / `ore unlock` / `ore locks`
**What they do:** Mark file(s) as locked / unlock / list all locked files.
**Syntax:** `ore lock [FILES]...` · `ore unlock [FILES]... [-a all]` · `ore locks`
**Use cases:** Guard rails for shared files: a config that shouldn't be edited casually (registry-only today — `rm`/`mv` guards are future work).
**Example:** `ore lock .env` then `ore locks`
**Can't do:** **Registry-only in v1** — locking records the file but doesn't yet *block* `rm`/`mv`/edits. It's an inventory, not enforcement, until those guards ship.

## Config & aliases

### `ore config`
**What it does:** Global config (get/set/list persistent settings).
**Syntax:** `ore config <COMMAND>` — get/set/list
**Use cases:** Persist preferences (defaults, formatting, keys) across invocations.
**Example:** `ore config set editor code`
**Can't do:** The config surface is the fixed key set — unknown keys are rejected.

### `ore alias`
**What it does:** User-defined command aliases.
**Syntax:** `ore alias <COMMAND>` — add/rm/list
**Use cases:** `ore alias add q "ore find"` then `ore q pattern` — your own shorthand.
**Example:** `ore alias add gs "ore git-status -s"`
**Can't do:** Simple text expansion (no argument templating beyond appending).
# Index, history, undo & redo

## The index

> The index is a SQLite database at `<workspace-root>/.ore-index/index.db` (find it with
> `index-locate`). It stores files, symbols, and imports so symbol lookups are fast across
> commands. It's **opt-in** — `--from-index` defaults are off — commands work without it.

### `ore index-build`
**What it does:** Build a SQLite index of files, symbols, and imports (fast reuse across commands).
**Syntax:** `ore index-build [ROOT]` · `-e -x` · `-f` force full rebuild · `--gitignore` auto-append `.ore-index/`
**Use cases:** The one-time setup that makes `index-search` and index-backed commands fast.
**Example:** `ore index-build --gitignore`
**Can't do:** Building is a scan — it doesn't *use* the index (that's `index-search` and commands with `--from-index`). The DB lives at the nearest workspace root, not necessarily your cwd.

### `ore index-update`
**What it does:** Incremental refresh: reindex only changed/new files.
**Syntax:** `ore index-update [ROOT]`
**Use cases:** The daily-driver maintenance — cheap after edits.
**Example:** `ore index-update`
**Can't do:** Needs an existing index (build first); deleted files are removed on update, but `index-gc` is the vacuum step.

### `ore index-status`
**What it does:** Show index size, file/symbol/import counts, staleness.
**Syntax:** `ore index-status [ROOT]`
**Use cases:** "Is the index current or stale?" before trusting index-backed results.
**Example:** `ore index-status`
**Can't do:** Read-only summary — no repair (that's `index-update`/`index-gc`).

### `ore index-search`
**What it does:** Fast symbol search via the index.
**Syntax:** `ore index-search <PATTERN> [ROOT]` · `-k <kind>` · `-E` exported only · `-n <top>` (default 50) · `-j`
**Use cases:** Instant symbol lookup across a big repo without a full scan.
**Example:** `ore index-search useAuth -k hook -E`
**Can't do:** Only as fresh as the last build/update — stale index = stale answers. Regex support is pattern-based like `symbols`.

### `ore index-gc`
**What it does:** Remove orphaned entries + vacuum.
**Syntax:** `ore index-gc [ROOT]`
**Use cases:** After a big deletion/rename, shrink and clean the DB.
**Example:** `ore index-gc`
**Can't do:** It removes entries pointing at missing files — it can't fix content that was never indexed.

### `ore index-clear`
**What it does:** Delete the index database.
**Syntax:** `ore index-clear [ROOT]` · `-y`
**Use cases:** Start fresh (rebuild with `index-build`); reclaim space; reset a corrupt DB.
**Example:** `ore index-clear -y`
**Can't do:** **This deletes the DB** — the AI tables (`ai_usage`, `ai_sessions`, `ai_models`) also live in `.ore-index/index.db`, so `ai-history`/`ai-session`/`ai-recall` data goes with it. Confirm before `-y`.

### `ore index-locate`
**What it does:** Print the index database path.
**Syntax:** `ore index-locate [ROOT]`
**Use cases:** Find where the DB actually is (workspace-root resolution can surprise you); point tooling at it.
**Example:** `ore index-locate`
**Can't do:** Prints the path only.

## History, undo & redo

> Every backup-making operation (patches, replaces, deletes, `mv`/`cp`/`rm` overwrites) records
> an entry in the operation log inside the index DB. `undo` restores from those backups.

### `ore history`
**What it does:** Show operation history (backups, patches, deletes) — auto-recorded.
**Syntax:** `ore history [ROOT]` · `-f <file>` filter · `-a` include undone · `-n <top>` (default 30) · `-j`
**Use cases:** "What did I (or the agent) touch today?" — the audit trail.
**Example:** `ore history -f src/main.ts`
**Can't do:** Only records operations ore performed (backup-eligible ones) — plain text edits with a different editor aren't in the log.

### `ore undo`
**What it does:** Undo the last N recorded operations (restores from backup).
**Syntax:** `ore undo [ROOT]` · `-n <count>` (default 1) · `-f <file>` · `--dry-run` preview · `-y`
**Use cases:** "Undo that patch I just made"; roll back an agent's edit.
**Example:** `ore undo --dry-run` then `ore undo -y`
**Can't do:** Restores the file content from the backup — it does not re-run inverse operations. Operations marked undone won't be re-undone.

### `ore redo`
**What it does:** Mark undone operations as redone (**does not replay changes**).
**Syntax:** `ore redo [ROOT]` · `-n <count>` (default 1) · `-y`
**Use cases:** Tell the log "I'm keeping the current state — don't treat it as undone anymore."
**Example:** `ore redo -y`
**Can't do:** **It does not replay** — there is no re-apply of the undone change. If you need the change back, re-apply it with the original command (or `restore`).

### `ore tui`
**What it does:** Launch the interactive TUI (file tree, preview, search, command palette, git panel).
**Syntax:** `ore tui` · `-p <path>` root (default: focus setting or cwd)
**Use cases:** Browse and work interactively; discover commands via the palette.
**Example:** `ore tui`
**Can't do:** A TUI, not a full IDE — editing happens via ore commands; it needs a real terminal (not a dumb pipe).
# Browser automation (web-*)

> These commands drive a real Chromium browser (via `headless_chrome`). They render pages, so
> they see what a browser sees — but **every command launches a fresh, stateless browser
> profile**. Nothing persists between invocations: cookies, localStorage, typed values, and
> logged-in sessions are gone on the next call. Design around that.

### `ore web-open`
**What it does:** Open a URL in a headless browser (or `--visible`).
**Syntax:** `ore web-open <URL>` · `-V` visible · `-w <selector>` wait · `-t` timeout (default 30) · `-k <secs>` keep open
**Use cases:** Warm a page; verify it renders; drive the browser for `-k`/`-V` demos.
**Example:** `ore web-open https://example.com -w "h1"`
**Can't do:** No persistence — a fresh profile each time; no interactive session across commands.

### `ore web-screenshot`
**What it does:** Screenshot a page (viewport / full-page / per-selector, device presets).
**Syntax:** `ore web-screenshot <URL>`
**Options:** `-o <file>` (default `screenshot.png`) · `-f` full page · `-s <selector>` element only · `--viewport WxH` · `-d <device>` (iphone-14, ipad, desktop, fhd, 4k…) · `-w <selector>` wait · `-F png|jpeg` · `-q <quality>`
**Use cases:** Visual regression captures; mobile-viewport checks; element-level screenshots for docs.
**Example:** `ore web-screenshot https://example.com -d iphone-14 -o mobile.png`
**Can't do:** No lazy-scroll stitching for infinitely-loading pages — full-page mode can miss content below the fold on virtualized sites.

### `ore web-pdf`
**What it does:** Render a page to PDF.
**Syntax:** `ore web-pdf <URL>` · `-o <file>` (default `page.pdf`) · `-L` landscape · `-b` backgrounds · `-m <inches>` margin (default 0.4) · `-w <selector>` wait · `-t` (default 60)
**Use cases:** Save an article/invoice/dashboard as a PDF.
**Example:** `ore web-pdf https://example.com/report -L -b`
**Can't do:** Print-CSS quirks apply — complex layouts may paginate oddly; no page-size/range controls.

### `ore web-text`
**What it does:** Extract visible text (optionally from a selector).
**Syntax:** `ore web-text <URL>` · `-s <selector>` (default `body`) · `-w` wait · `-t` · `-o <file>`
**Use cases:** Read an article without the chrome; feed text to an LLM or a summarizer.
**Example:** `ore web-text https://example.com -s article`
**Can't do:** Visible text only — hidden/`display:none` content is excluded by the browser, and JS-heavy content needs a `-w` wait.

### `ore web-html`
**What it does:** Extract rendered HTML (optionally per-selector).
**Syntax:** `ore web-html <URL>` · `-s <selector>` · `-w` · `-t` · `-o <file>`
**Use cases:** Get the *rendered* DOM (post-JS) for analysis or scraping.
**Example:** `ore web-html https://example.com -s "#app" -o app.html`
**Can't do:** It's the live DOM — scripts re-run on capture; output isn't normalized/cleaned (that's `web-fetch-clean`).

### `ore web-title`
**What it does:** Print the page title.
**Syntax:** `ore web-title <URL>` · `-t`
**Use cases:** Quick "what page is this" checks in scripts.
**Example:** `ore web-title https://example.com`
**Can't do:** Title only — no meta extraction.

### `ore web-links`
**What it does:** Extract all links from a page (with filters + same-domain).
**Syntax:** `ore web-links <URL>` · `-f <substring>` filter · `-s` same-domain only · `-o` · `-j`
**Use cases:** Sitemap discovery; link audits; finding a specific navigation item.
**Example:** `ore web-links https://example.com -s -j`
**Can't do:** Anchors only, deduped by URL — no link text ranking; relative links are resolved against the page.

### `ore web-click`
**What it does:** Click an element and inspect the resulting state.
**Syntax:** `ore web-click <URL> <SELECTOR>` · `-d <ms>` delay after click (default 500) · `--screenshot <path>` · `-V` · `-t`
**Use cases:** Verify a button navigates; trigger a JS action and screenshot the result.
**Example:** `ore web-click https://example.com "a" --screenshot after.png`
**Can't do:** One click per invocation (state doesn't persist); no drag/keyboard-combo actions.

### `ore web-type`
**What it does:** Type into an input (with optional `--submit` and `--clear`).
**Syntax:** `ore web-type <URL> <SELECTOR> <TEXT>` · `--submit` press Enter · `-c` clear first · `-V` · `-t`
**Use cases:** Fill a search box and submit; form automation where the URL reflects the submission.
**Example:** `ore web-type https://example.com "input[name=q]" "hello" --submit`
**Can't do:** Fresh browser per call — the typed value dies with the session (a submit that navigates is observable via the resulting URL, but you can't continue the session).

### `ore web-eval`
**What it does:** Evaluate JavaScript on a page and print the return value.
**Syntax:** `ore web-eval <URL> <EXPRESSION>` · `-j` JSON
**Use cases:** Probe live state (`document.title`, `window.__DATA__`), run one-off browser-side logic.
**Example:** `ore web-eval https://example.com "document.querySelectorAll('a').length"`
**Can't do:** The expression runs in the page context and its return value must be JSON-serializable to be useful; no multi-statement debugging session.

### `ore web-wait`
**What it does:** Wait for a selector / text / URL substring.
**Syntax:** `ore web-wait <URL>` · `--selector <sel>` · `--text <txt>` · `--url-contains <sub>` · `-t` timeout (default 30) · `-i` poll interval (default 500)
**Use cases:** The condition-gate: "wait until the login redirect lands" before scraping.
**Example:** `ore web-wait https://example.com/login --url-contains "dashboard" -t 60`
**Can't do:** One condition per call; exits with an error on timeout (use it with `&&` chains).

### `ore web-scrape`
**What it does:** Structured scrape: `field=selector` pairs, optional repeating container.
**Syntax:** `ore web-scrape <URL> -f "title=h1" -f "price=.price" …`
**Options:** `-r <selector>` repeating container (fields relative to each match) · `-a <attr>` attribute instead of text · `-F json|csv` · `-w` wait · `-o` output
**Use cases:** Product listings, search results, tables — the structured extraction workhorse.
**Example:** `ore web-scrape https://example.com/items -r ".item" -f "name=h2" -f "price=.price" -F csv`
**Can't do:** CSS-selector scraping — it won't reverse-engineer data from JSON blobs (use `web-eval` for that); complex pagination is your loop to write.

### `ore web-screenshot-many`
**What it does:** Screenshot many URLs to a directory.
**Syntax:** `ore web-screenshot-many [URLS]...` · `-f <file>` URL list (# comments allowed) · `-o <dir>` (default `./screenshots`) · `-F` full page · `-t`
**Use cases:** Bulk captures for audits; visual inventory of many routes.
**Example:** `ore web-screenshot-many -f urls.txt -o shots/`
**Can't do:** One fresh browser per URL — slow for large lists; no per-URL device/selector settings.

### `ore web-screenshot-set`
**What it does:** Screenshot one URL at multiple viewport widths (responsive audit).
**Syntax:** `ore web-screenshot-set <URL>` · `-s <widths>` (default `375,768,1024,1440,1920`) · `-a <aspect>` (default 1.7777) · `-o <dir>` · `-F` full page
**Use cases:** Responsive design review in one command.
**Example:** `ore web-screenshot-set https://example.com -s 375,768,1440`
**Can't do:** Heights are derived from a single aspect ratio — not per-preset device metrics.

### `ore web-cookies`
**What it does:** Dump all cookies for a URL.
**Syntax:** `ore web-cookies <URL>` · `-j`
**Use cases:** Inspect what a site sets (for a fresh profile — remember, no persistence).
**Example:** `ore web-cookies https://example.com -j`
**Can't do:** The cookies belong to a throwaway profile — you can't inject them back into a real session.

### `ore web-ws-status`
**What it does:** Quick ready-state check for a URL (**exits 1 if not ready**).
**Syntax:** `ore web-ws-status <URL>` · `-t` · `-q`
**Use cases:** The scriptable "is the page actually rendered?" gate in CI/deploy checks.
**Example:** `ore web-ws-status https://myapp.com && echo "ready"`
**Can't do:** Ready-state ≈ page finished loading — it doesn't assert specific content (see `web-wait --text`).

### `ore web-check`
**What it does:** Bulk headless render check across many URLs.
**Syntax:** `ore web-check [URLS]...` · `-f <file>` · `-F` failures only · `-t` (default 15)
**Use cases:** Post-deploy smoke sweep; verify N routes render.
**Example:** `ore web-check -f routes.txt -F`
**Can't do:** Render/status check only — no content assertions; one browser per URL keeps it simple but slow.
# Web search & AI

## Web search

### `ore web-search`
**What it does:** Search the web via SearXNG with DuckDuckGo fallback.
**Syntax:** `ore web-search <QUERY>` · `-j` JSON · `-o <file>` · `-q` result-only
**Use cases:** Real web search from the terminal; the retrieval step the AI agents use.
**Example:** `ore web-search "rust async runtime comparison" -j`
**Can't do:** Results are text snippets — no page rendering (use `web-fetch-clean`/`web-open` for content); public SearXNG instances are rate-limited, so searches fall through to DuckDuckGo when instances are busy.

### `ore web-search-config`
**What it does:** Configure search endpoint, fallbacks, limits.
**Syntax:** `ore web-search-config <COMMAND>` — list/set/get
**Use cases:** Point at your own SearXNG instance; tune `search_max_results`, timeouts, fallback lists.
**Example:** `ore web-search-config set search_searxng_url https://searx.example.com`
**Can't do:** Config lives in the same `ai.toml` as AI settings — see `ai-config` for the full key list.

### `ore web-search-instances`
**What it does:** List/test SearXNG instances (primary + fallbacks) with latency.
**Syntax:** `ore web-search-instances` · `-t` probe each with a 3s latency check
**Use cases:** Pick a fast primary for `web-search-config`; monitor which public instances work today.
**Example:** `ore web-search-instances -t`
**Can't do:** Latency is a point-in-time probe — public instances throttle erratically; a slow probe today ≠ dead instance.

### `ore web-fetch-clean`
**What it does:** Fetch a URL and strip to article text (removes nav/scripts/styles).
**Syntax:** `ore web-fetch-clean <URL>` · `-m <max-chars>` (default from config) · `-o <file>`
**Use cases:** The clean article extractor for LLM context — no script noise.
**Example:** `ore web-fetch-clean https://en.wikipedia.org/wiki/CLI -m 800`
**Can't do:** Text extraction is heuristics — interactive apps/SPAs with no article-like content extract poorly; paywalls and JS-only rendering defeat it (those need `web-text` on a rendered page).

### `ore ai-search-test`
**What it does:** Dry-run: show exactly what an agent would retrieve for a query.
**Syntax:** `ore ai-search-test <QUERY>`
**Use cases:** Debug retrieval — see the instance chain, failures, and truncated results before an agent burns tokens.
**Example:** `ore ai-search-test "what is oregrep"`
**Can't do:** It's the retrieval preview only — no LLM call, no follow-up fetches.

## AI setup

### `ore ai-keys`
**What it does:** Manage AI provider API keys (register/unregister/list/test/rotate).
**Syntax:** `ore ai-keys <COMMAND>` — register/unregister/list/test/rotate
**Use cases:** Register a Groq/OpenAI/Anthropic key; verify a key with a live test; rotate a leaked one.
**Example:** `ore ai-keys register groq gsk-…` then `ore ai-keys test groq`
**Can't do:** Keys are stored locally in the state dir — protection is best-effort file permissions; env vars (`GROQ_API_KEY` etc.) override stored keys.

### `ore ai-config`
**What it does:** AI configuration (default provider/model/budget/temperature/etc.).
**Syntax:** `ore ai-config <COMMAND>` — list/set/get/path
**Use cases:** Set `default_provider`; raise `max_output_tokens`; set `session_budget_usd`; tune `cost_mode` (cheap/balanced/quality); set `router_provider`.
**Example:** `ore ai-config set default_provider anthropic`
**Can't do:** Unknown keys are rejected; budgets only *estimate* cost for uncosted models (see `ai-budget`).

### `ore ai-models`
**What it does:** List models available from a provider (with pricing + context window).
**Syntax:** `ore ai-models <PROVIDER>` · `-r` force refresh (ignore cache) · `-j`
**Use cases:** See what a provider offers before `-m provider:model`; check pricing/context.
**Example:** `ore ai-models groq -j`
**Can't do:** Needs a valid key (or a running local server for ollama/lmstudio); pricing is augmented from a curated table where providers omit it.

### `ore ai-providers`
**What it does:** Show all configured/available providers.
**Syntax:** `ore ai-providers`
**Use cases:** "What can I use right now?" — which keys are stored, which are local.
**Example:** `ore ai-providers`
**Can't do:** Shows configuration state — local backends show available even if no server is running (they fail on first real call).

## Usage, budgets, memory

### `ore ai-usage`
**What it does:** Cumulative token + cost usage across all AI calls.
**Syntax:** `ore ai-usage` · `-d <days>` · `-j`
**Use cases:** Cost tracking; "how many tokens did this week cost me?"
**Example:** `ore ai-usage -d 7`
**Can't do:** Per-workspace (the DB is in `.ore-index`) — totals are scoped to where you run it.

### `ore ai-history`
**What it does:** Full history of every AI call (timestamp, task, model, cost, tokens).
**Syntax:** `ore ai-history` · `-n <limit>` (default 50) · `-t <task>` filter · `--today` · `-j`
**Use cases:** Audit what the agents did; reconcile invoices; find a past generation.
**Example:** `ore ai-history -t fix --today`
**Can't do:** Same per-workspace scope; only records successful (recorded) generations.

### `ore ai-budget`
**What it does:** Show current process AI spend vs configured caps.
**Syntax:** `ore ai-budget` · `-j`
**Use cases:** See what this shell session has spent; check remaining before a big agent run.
**Example:** `ore ai-budget`
**Can't do:** Process spend resets when the process exits; estimates for models without pricing entries are $0 (the call-budget cap can't block those); enforcement happens *before* the HTTP call, so a mid-call overrun still completes.

### `ore ai-prompts`
**What it does:** Edit/list/reset the AI system prompts.
**Syntax:** `ore ai-prompts <COMMAND>` — list/show/edit/reset/diff
**Use cases:** Tune the agent's personality and instructions; diff your customizations against the bundled defaults.
**Example:** `ore ai-prompts edit ask`
**Can't do:** Prompt names are the fixed bundled set (ask, explain, review, fix, refactor, commit-message, chat-system, router) — new custom prompt *types* aren't added via this command.

### `ore ai-session`
**What it does:** Manage AI chat sessions (list/show/rm).
**Syntax:** `ore ai-session <COMMAND>` — list/show/rm
**Use cases:** See all conversations; inspect one; delete a stale session.
**Example:** `ore ai-session list` · `ore ai-session show work -n 20`
**Can't do:** Per-workspace storage; `rm` needs confirmation (`-y` to skip).

### `ore ai-recall`
**What it does:** Full-text search across all past AI session messages.
**Syntax:** `ore ai-recall <QUERY>` · `-n <limit>` (default 20) · `-s <session>` · `-j`
**Use cases:** "What did I ask about the budget last week?" — memory across sessions.
**Example:** `ore ai-recall "budget" -s work`
**Can't do:** Substring (LIKE) search — no semantic search, no fuzzy ranking; scoped to the workspace's sessions.

## AI generation

### `ore ai-ask`
**What it does:** One-shot AI question with streaming + auto model selection — and **agentic read-only tool access by default**.
**Syntax:** `ore ai-ask [QUESTION]` (reads stdin if omitted)
**Options:** `-m provider:model` · `--no-stream` · `--events-json` · `-q` · `-W` why (router decision + timing) · `--no-tools` (old stateless behavior) · `--auto` (also allow destructive tools) · `-s <session>` · `--continue`
**Use cases:** Ask a question with the codebase as context — the model can run `ore-find`/`ore-cat`-style read-only tools to answer; script it with `-q`.
**Example:** `ore ai-ask "how many files import from src/db.ts?" -W`
**Can't do:** Without `--auto`, tools are **read-only** — it can't edit files. Session history only loads with `-s`/`--continue`.

### `ore ai-chat`
**What it does:** Persistent multi-turn chat (with session storage).
**Syntax:** `ore ai-chat` · `-s <session>` (default `default`) · `-m` · `--no-stream` · `-q` · `-p <prompt>` one-shot (skip REPL)
**Use cases:** The REPL: `/exit`, `/reset`, `/history`; persistent conversations across runs.
**Example:** `ore ai-chat -s work` then `ore ai-chat -p "summarize our plan" -s work`
**Can't do:** No tool use in chat (that's `ai-ask`/`ai-agent`); history is capped at the last 40 messages in context.

### `ore ai-agent`
**What it does:** Autonomous agent loop with tool access (research, exploration, edits).
**Syntax:** `ore ai-agent <TASK>` · `-m` · `--auto` approve destructive tools · `-i <iters>` (default 10) · `--events-json` · `-q` · `-s <session>` · `--continue`
**Use cases:** "Find the three largest Rust files and summarize them"; open-ended research with web-search + ore tools.
**Example:** `ore ai-agent "What are the top coupling hotspots? Use the analyze tools."`
**Can't do:** Without `--auto`, destructive tools (patch/replace/backup/restore/verify) are blocked. Iterations are capped (`-i`) — a loop can hit the cap without finishing. Every call costs tokens/money and hits provider rate limits.

### `ore ai-explain`
**What it does:** LLM-quality explanation of a file — or a repo-wide question.
**Syntax:** `ore ai-explain <FILE_OR_QUESTION>` · `-m` · `--no-stream` · `-q`
**Use cases:** A file: "explain what this does." A non-file argument: runs `ore digest .` and answers a repo-level question with structural context.
**Example:** `ore ai-explain src/main.rs` · `ore ai-explain "how does the undo system work?"`
**Can't do:** Large files are auto-condensed (comments stripped) — fine for understanding, lossy for exact detail; repo questions use digest structure, not full source.

### `ore ai-review`
**What it does:** AI code review with severity + line refs.
**Syntax:** `ore ai-review <FILE>` · `-m` · `--no-stream` · `-q`
**Use cases:** A second pair of eyes before a PR; pre-commit review.
**Example:** `ore ai-review src/db.ts`
**Can't do:** Reviews one file at a time (a full-PR review isn't this command); large files auto-condense.

### `ore ai-fix`
**What it does:** Agent that analyzes + patches + verifies + rolls back on failure.
**Syntax:** `ore ai-fix <FILE>` · `-i <issue>` · `-m` · `--auto` · `--max-iters <n>` (default 6) · `-q` · `-s <session>` · `--continue`
**Use cases:** "Fix the type mismatch in this file" — the agent backs up, patches, verifies, restores on failure.
**Example:** `ore ai-fix src/main.ts -i "unused variable warnings" --auto`
**Can't do:** Without `--auto`, the agent can't patch (destructive tools blocked). Verification depends on a working `tsc`/`cargo`/`verify` in the project. It's an AI — review its changes before committing.

### `ore ai-refactor`
**What it does:** Multi-step agent that plans → executes → verifies a refactor intent.
**Syntax:** `ore ai-refactor <FILE> <INTENT>` · `-m` · `--auto` · `--max-iters <n>` (default 10) · `-q` · `-s <session>` · `--continue`
**Use cases:** "Extract the validation into a helper" — with plan-first instructions and verify-after-each-step.
**Example:** `ore ai-refactor src/util.ts "extract date parsing into its own module" --auto`
**Can't do:** Same destructive-tool and verification caveats as `ai-fix`; ambitious intents may exceed `--max-iters`.

### `ore ai-commit-message`
**What it does:** Generate (and optionally apply) a git commit message from the diff.
**Syntax:** `ore ai-commit-message` · `--unstaged` (default staged) · `-m <model>` · `-c` actually commit · `--no-stream` · `-q`
**Use cases:** LLM-quality commit messages from the staged diff; print-only mode for review.
**Example:** `ore git-stage --all` then `ore ai-commit-message -c`
**Can't do:** Needs a git repo with a non-empty diff; `-c` runs the commit — review the message first (print-only is the default).

---

## AI: how the pieces fit (quick map)

- **Router** — every AI command picks provider+model via `route()`: an LLM router call (cheapest configured model) if `router_provider` is set and keyed, otherwise the heuristic (`cost_mode` + task class + registered providers). Failures fall back silently.
- **Safety** — `ai-fix`/`ai-refactor` operate through the backup/undo/verify loop: read → backup → patch → verify → restore on failure. `undo` can revert any of their edits.
- **Costs** — every call records usage (`ai-usage`/`ai-history`), accrues process spend (`ai-budget`), and is pre-checked against `session_budget_usd`/`call_budget_usd` before the HTTP request.
- **Rate limits** — 429/5xx retries with exponential backoff (up to 4 attempts); 413 yields an actionable error (condense the input or switch models).
- **Backends** — registered keys for groq/openai/anthropic/google/deepseek/mistral/openrouter; local ollama/lmstudio need a running server. Free tiers are rate-limited — agent loops burn through them fast; budget accordingly.
