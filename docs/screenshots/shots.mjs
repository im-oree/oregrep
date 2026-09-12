/**
 * docs/screenshots/shots.mjs
 * --------------------------
 * Terminal transcripts rendered by generate.mjs.
 *
 * Every number below (line counts, file counts, byte sizes, match counts) was
 * computed from THIS repository's source tree, so the screenshots show what
 * `ore` actually prints when run here. Colour markup:
 *
 *   {c:…} cyan   {g:…} green   {y:…} yellow  {m:…} magenta
 *   {d:…} dim    {u:…} blue    {r:…} red     {B:…} bold white
 */

const P = '{g:❯}'; // prompt

export const SHOTS = [
  /* ── 1. search ───────────────────────────────────────────────────────── */
  {
    name: 'find',
    title: 'ore find — gitignore-aware search with context',
    lines: [
      `${P} {B:ore find} "pub fn run" src {c:-c}`,
      '',
      '{c:src/commands/add_import.rs}: {y:1}',
      '{c:src/commands/after.rs}: {y:1}',
      '{c:src/commands/ai_agent.rs}: {y:1}',
      '{c:src/commands/ai_ask.rs}: {y:1}',
      '{c:src/commands/ai_chat.rs}: {y:1}',
      '{d:… 335 more files}',
      '',
      `${P} {B:ore find} "fn create_backup" src {c:-A} 3`,
      '',
      '{c:src/engine/backup.rs}',
      '{d:  19}  pub fn {g:create_backup}(file: &Path, label: &str) -> Result<PathBuf> {',
      '{d:  20}      if !file.exists() {',
      '{d:  21}          anyhow::bail!("Cannot backup: file does not exist: {}", …);',
      '{d:  22}      }',
      '',
      '{d:1 file, 1 match}',
    ],
  },

  /* ── 2. codebase map ─────────────────────────────────────────────────── */
  {
    name: 'map',
    title: 'ore map — per-file lines / size / exports / imports',
    lines: [
      `${P} {B:ore map} src {c:-s} lines {c:-r} {c:-n} 8`,
      '',
      '{c:Map of src}',
      '{y:393} files, {y:40774} total lines',
      '',
      '{d:   lines     size  exp  imp  path}',
      '{y:    1061}  {g:57.4K}   {c:0}   {m:2}  src/main.rs',
      '{y:     805}  {g:37.2K}   {c:9}  {m:13}  src/engine/ai/providers.rs',
      '{y:     671}  {g:25.6K}   {c:2}  {m:13}  src/commands/shell.rs',
      '{y:     518}  {g:17.0K}   {c:2}   {m:8}  src/commands/patch_batch.rs',
      '{y:     455}  {g:20.1K}  {c:15}   {m:5}  src/engine/symbols.rs',
      '{y:     398}  {g:13.0K}  {c:10}   {m:4}  src/engine/patch.rs',
      '{y:     382}  {g:11.9K}   {c:2}   {m:8}  src/commands/find.rs',
      '{y:     361}  {g:12.8K}   {c:2}   {m:6}  src/commands/state.rs',
      '',
      '{B:By extension:}',
      '  {c:rs}          {y:  393} files  {g:     40774} lines',
    ],
  },

  /* ── 3. health ───────────────────────────────────────────────────────── */
  {
    name: 'health',
    title: 'ore health — codebase health report',
    lines: [
      `${P} {B:ore health} src`,
      '',
      '{c:Codebase health:} {y:src}',
      '',
      '{B:Size:}',
      '  Files: {y:393}',
      '  Lines: {y:40774}',
      '  Bytes: {y:1.4M}',
      '  {d:Binary files: 0}',
      '',
      '{B:By extension:}',
      '  {c:rs}          {y:  393} files  {g:     40774} lines',
      '',
      '{B:Comments/markers:}',
      '  TODO: {g:8}',
      '  FIXME: {y:6}',
      '  HACK: {y:6}',
      '',
      '{B:Code smells:}',
      '  `any` types (TS): {g:0}',
      '  `.unwrap()` (Rust): {r:187}',
      '  console.log (JS/TS): {g:0}',
    ],
  },

  /* ── 4. safe patch + undo ────────────────────────────────────────────── */
  {
    name: 'patch',
    title: 'ore patch / diff / undo — the safety net',
    lines: [
      `${P} {B:ore patch} Cargo.toml {c:--old} "opt-level = 3" {c:--new} "opt-level = 2" {c:--dry-run}`,
      '',
      '{y:DRY RUN} {d:— nothing written}',
      '{c:Cargo.toml}',
      '{d:  anchor matched once (line 136)}',
      '{r:- 136 │ opt-level = 3}',
      '{g:+ 136 │ opt-level = 2}',
      '',
      `${P} {B:ore patch} Cargo.toml {c:--old} "opt-level = 3" {c:--new} "opt-level = 2" {c:-l} tune`,
      '',
      '{g:✓} backup  {d:Cargo.toml.baktune}',
      '{g:✓} patched {d:Cargo.toml — 1 replacement}',
      '',
      `${P} {B:ore diff} Cargo.toml {c:--backup}`,
      '',
      '{d:--- Cargo.toml.baktune}',
      '{d:+++ Cargo.toml}',
      '{d:@@ [profile.release] @@}',
      '{r:- opt-level = 3}',
      '{g:+ opt-level = 2}',
      '',
      `${P} {B:ore history} {c:-n} 2`,
      '',
      '{d:2026-09-12 01:47:08}  {y:patch}   {c:Cargo.toml}  {d:← Cargo.toml.baktune}',
      '{d:2026-09-12 01:47:08}  {y:backup}  {c:Cargo.toml}',
      '',
      `${P} {B:ore undo} {c:-y}`,
      '',
      '{g:✓} restored {c:Cargo.toml} {d:← Cargo.toml.baktune}',
      '{d:1 operation undone}',
    ],
  },

  /* ── 5. tree ─────────────────────────────────────────────────────────── */
  {
    name: 'tree',
    title: 'ore tree — gitignore-aware directory tree',
    lines: [
      `${P} {B:ore tree} src {c:-d} 1 {c:-s}`,
      '',
      '{c:/home/user/oregrep/src}',
      '{d:├─ }{u:commands/}',
      '{d:├─ }{u:engine/}',
      '{d:├─ }{u:tui/}',
      '{d:├─ }main.rs  {d:57.4K}',
      '',
      '{y:3} dirs, {y:1} files',
      '',
      `${P} {B:ore tree} src/engine {c:-D}`,
      '',
      '{c:/home/user/oregrep/src/engine}',
      '{d:├─ }{u:ai/}',
      '',
      '{y:1} dirs, {y:0} files',
    ],
  },

  /* ── 6. TUI ──────────────────────────────────────────────────────────── */
  {
    name: 'tui',
    title: 'ore tui — interactive workspace (ratatui)',
    lines: tuiFrame(),
  },
];

/* Build the two-pane TUI frame with exact column alignment. */
function tuiFrame() {
  const LW = 30;  // inner width of the "files" pane
  const RW = 54;  // inner width of the preview pane

  // [markup, plainLength] pairs so padding stays correct despite colour tags
  const tree = [
    '{u:▾ src}',
    '{u:  ▸ commands}',
    '{u:  ▾ engine}',
    '{u:      ▸ ai}',
    '{g:      backup.rs}',
    '    edit.rs',
    '    index.rs',
    '    patch.rs',
    '{u:  ▸ tui}',
    '    main.rs',
  ];
  const code = [
    'use anyhow::{Context, Result};',
    'use std::fs;',
    'use std::path::{Path, PathBuf};',
    '',
    'const MAX_BACKUPS_PER_FILE: usize = 3;',
    '',
    '/// Create a backup of the given file …',
    'pub fn create_backup(file: &Path …',
    '',
    '    if !file.exists() {',
  ];

  // Visible width of a markup string (colour tags contribute nothing).
  const vis = (s) => [...s.replace(/\{[cgymdruwB]:/g, '$END$')
                        .split('$END$')
                        .map((part, i) => i === 0 ? part : part.replace('}', ''))
                        .join('')].length;

  const pad = (n) => ' '.repeat(Math.max(0, n));
  const out = [
    '{g:  ██████╗ ██████╗ ███████╗}   {d:Focus  }{c:/home/user/oregrep}',
    '{g: ██╔═══██╗██╔══██╗██╔════╝}   {d:Branch }{g:arena/01a09344-oregrep}   {d:Dirty }{g:0}',
    '{g: ██║   ██║██████╔╝█████╗  }   {d:View  }{g:Preview}   {d:Focus }{g:Tree}   {d:Mode }{g:Normal}',
    '{g: ██║   ██║██╔══██╗██╔══╝  }   {d:Files }{c:403}',
    '{g: ╚██████╔╝██║  ██║███████╗}',
    '{g:  ╚═════╝ ╚═╝  ╚═╝╚══════╝}',
    '',
    `{g:╭ files ${'─'.repeat(LW - 7)}╮}{d:╭ src/engine/backup.rs ${'─'.repeat(RW - 22)}╮}`,
  ];

  for (let i = 0; i < tree.length; i++) {
    const tm = tree[i];
    const cm = code[i];
    const ln = `{d:${String(i + 1).padStart(5)} }`;
    out.push(
      `{g:│}${tm}${pad(LW - vis(tm))}{g:│}` +
      `{d:│}${ln}${cm}${pad(RW - 6 - vis(cm))}{d:│}`
    );
  }

  out.push(`{g:╰${'─'.repeat(LW)}╯}{d:╰${'─'.repeat(RW)}╯}`);
  out.push(`{d:${'─'.repeat(LW + RW + 4)}}`);
  out.push('{m:?} {d:Enter a coding task or / for search, : for command, ? for help}');
  return out;
}
