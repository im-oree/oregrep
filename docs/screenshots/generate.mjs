#!/usr/bin/env node
/**
 * docs/screenshots/generate.mjs
 * -----------------------------
 * Renders the terminal screenshots used in README.md.
 *
 * Each "shot" is an ANSI-styled transcript of a real `ore` command. The
 * transcripts in `shots.mjs` are produced from this repository's own source
 * tree (line counts, file names and match counts are computed, not invented),
 * then painted into an HTML terminal and captured with Playwright/Chromium.
 *
 * Usage:
 *   npm install playwright
 *   node docs/screenshots/generate.mjs
 *
 * Env:
 *   CHROME_PATH   path to a Chromium/Chrome binary (optional; Playwright's
 *                 bundled browser is used when unset)
 */
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { SHOTS } from './shots.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const OUT = HERE;
mkdirSync(OUT, { recursive: true });

/* ── palette: mirrors src/tui/theme.rs + the `colored` crate defaults ─────── */
const C = {
  bg: '#12131a', fg: '#d7dae0', dim: '#6f7580',
  cyan: '#56d4dd', green: '#6be398', yellow: '#e5c07b',
  magenta: '#c678dd', red: '#e06c75', blue: '#61afef', white: '#ffffff',
};

const esc = (s) => String(s)
  .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/** Tiny markup: {c:text} {g:text} {y:text} {m:text} {d:text} {b:text} {B:text} {r:text} */
function paint(line) {
  const map = { c: C.cyan, g: C.green, y: C.yellow, m: C.magenta,
                d: C.dim, r: C.red, u: C.blue, w: C.white };
  let out = '', i = 0;
  while (i < line.length) {
    const m = /\{([cgymdruwB]):/.exec(line.slice(i));
    if (!m || m.index !== 0) { out += esc(line[i]); i += 1; continue; }
    let depth = 1, j = i + m[0].length, body = '';
    while (j < line.length && depth > 0) {
      if (line[j] === '{') depth++;
      if (line[j] === '}') { depth--; if (!depth) break; }
      body += line[j]; j++;
    }
    const bold = m[1] === 'B';
    const color = bold ? C.white : map[m[1]];
    out += `<span style="color:${color}${bold ? ';font-weight:700' : ''}">${paint(body)}</span>`;
    i = j + 1;
  }
  return out;
}

const page_html = (shot) => `<!doctype html><html><head><meta charset="utf-8">
<style>
  @import url('');
  * { box-sizing: border-box; }
  body { margin:0; background:#0a0b0f; padding:26px;
         font-family: "DejaVu Sans Mono", "Liberation Mono", Menlo, Consolas, monospace; }
  .term { background:${C.bg}; border-radius:10px; overflow:hidden;
          box-shadow:0 18px 50px rgba(0,0,0,.55); border:1px solid #23252e; }
  .bar { height:34px; background:#1b1d25; display:flex; align-items:center;
         padding:0 13px; gap:8px; border-bottom:1px solid #23252e; }
  .dot { width:11px; height:11px; border-radius:50%; }
  .title { color:${C.dim}; font-size:12px; margin-left:10px; letter-spacing:.4px; }
  pre { margin:0; padding:16px 18px; color:${C.fg}; font-size:13.5px;
        line-height:1.52; white-space:pre; }
</style></head><body>
<div class="term">
  <div class="bar">
    <div class="dot" style="background:#ff5f57"></div>
    <div class="dot" style="background:#febc2e"></div>
    <div class="dot" style="background:#28c840"></div>
    <div class="title">${esc(shot.title)}</div>
  </div>
  <pre>${shot.lines.map(paint).join('\n')}</pre>
</div></body></html>`;

const launchOpts = { args: ['--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu',
                            '--disable-software-rasterizer', '--in-process-gpu', '--no-zygote'] };
if (process.env.CHROME_PATH) launchOpts.executablePath = process.env.CHROME_PATH;

const browser = await chromium.launch(launchOpts);
const page = await browser.newPage({ viewport: { width: 1180, height: 800 },
                                     deviceScaleFactor: 2 });

for (const shot of SHOTS) {
  const html = page_html(shot);
  writeFileSync(join(OUT, `.${shot.name}.html`), html);
  await page.setContent(html, { waitUntil: 'load' });
  const el = await page.$('.term');
  await el.screenshot({ path: join(OUT, `${shot.name}.png`) });
  console.log(`  ✓ ${shot.name}.png`);
}

await browser.close();
console.log(`\n${SHOTS.length} screenshots written to docs/screenshots/`);
