# ORE  Complete Command Reference

**Generated:** 2026-08-15 15:10:52
**Total commands:** 306

Use \`ore <command> --help\` for the same info at any time.

---

## Table of Contents
- [\$cmd\](#add-import)
- [\$cmd\](#after)
- [\$cmd\](#ai-agent)
- [\$cmd\](#ai-ask)
- [\$cmd\](#ai-budget)
- [\$cmd\](#ai-chat)
- [\$cmd\](#ai-commit-message)
- [\$cmd\](#ai-config)
- [\$cmd\](#ai-explain)
- [\$cmd\](#ai-fix)
- [\$cmd\](#ai-history)
- [\$cmd\](#ai-keys)
- [\$cmd\](#ai-models)
- [\$cmd\](#ai-prompt)
- [\$cmd\](#ai-prompts)
- [\$cmd\](#ai-providers)
- [\$cmd\](#ai-recall)
- [\$cmd\](#ai-refactor)
- [\$cmd\](#ai-review)
- [\$cmd\](#ai-search-test)
- [\$cmd\](#ai-session)
- [\$cmd\](#ai-usage)
- [\$cmd\](#alias)
- [\$cmd\](#analyze-churn)
- [\$cmd\](#analyze-circular)
- [\$cmd\](#analyze-complexity)
- [\$cmd\](#analyze-coupling)
- [\$cmd\](#analyze-dead-exports)
- [\$cmd\](#analyze-duplication)
- [\$cmd\](#analyze-exports)
- [\$cmd\](#analyze-hotspot)
- [\$cmd\](#analyze-imports)
- [\$cmd\](#analyze-type-coverage)
- [\$cmd\](#api-test)
- [\$cmd\](#apply-patch)
- [\$cmd\](#backup)
- [\$cmd\](#before)
- [\$cmd\](#benchmark)
- [\$cmd\](#bench-url)
- [\$cmd\](#bin-cat)
- [\$cmd\](#bin-slice)
- [\$cmd\](#bin-stats)
- [\$cmd\](#blast-radius)
- [\$cmd\](#bookmark)
- [\$cmd\](#cat)
- [\$cmd\](#cat-around)
- [\$cmd\](#check-deps)
- [\$cmd\](#checksum)
- [\$cmd\](#check-urls)
- [\$cmd\](#chunk)
- [\$cmd\](#collapse-blank-lines)
- [\$cmd\](#compile-node)
- [\$cmd\](#compile-rust)
- [\$cmd\](#compile-ts)
- [\$cmd\](#condense)
- [\$cmd\](#config)
- [\$cmd\](#consolidate)
- [\$cmd\](#copy)
- [\$cmd\](#count)
- [\$cmd\](#cp)
- [\$cmd\](#crawl)
- [\$cmd\](#csv-filter)
- [\$cmd\](#csv-query)
- [\$cmd\](#csv-select)
- [\$cmd\](#csv-stats)
- [\$cmd\](#csv-to-json)
- [\$cmd\](#dedup-lines)
- [\$cmd\](#delete-lines)
- [\$cmd\](#diff)
- [\$cmd\](#diff-dirs)
- [\$cmd\](#diff-ignore)
- [\$cmd\](#diff-semantic)
- [\$cmd\](#diff-summary)
- [\$cmd\](#diff-word)
- [\$cmd\](#digest)
- [\$cmd\](#dns)
- [\$cmd\](#download)
- [\$cmd\](#download-many)
- [\$cmd\](#encoding)
- [\$cmd\](#env-diff)
- [\$cmd\](#env-get)
- [\$cmd\](#env-set)
- [\$cmd\](#errors-last)
- [\$cmd\](#explain)
- [\$cmd\](#extract)
- [\$cmd\](#extract-fn)
- [\$cmd\](#fetch)
- [\$cmd\](#fetch-many)
- [\$cmd\](#filesize)
- [\$cmd\](#find)
- [\$cmd\](#find-dupes)
- [\$cmd\](#flatten-hub)
- [\$cmd\](#focus)
- [\$cmd\](#git-amend)
- [\$cmd\](#git-auto-commit)
- [\$cmd\](#git-auto-message)
- [\$cmd\](#git-blame)
- [\$cmd\](#git-changed)
- [\$cmd\](#git-changelog)
- [\$cmd\](#git-cleanup-branches)
- [\$cmd\](#git-commit)
- [\$cmd\](#git-commit-body)
- [\$cmd\](#git-diff)
- [\$cmd\](#git-fixup)
- [\$cmd\](#git-history)
- [\$cmd\](#git-log)
- [\$cmd\](#git-release-notes)
- [\$cmd\](#git-search)
- [\$cmd\](#git-stage)
- [\$cmd\](#git-stash-named)
- [\$cmd\](#git-status)
- [\$cmd\](#git-suggest-commit)
- [\$cmd\](#git-undo-commit)
- [\$cmd\](#git-who)
- [\$cmd\](#head)
- [\$cmd\](#headers)
- [\$cmd\](#health)
- [\$cmd\](#hex-delete)
- [\$cmd\](#hex-diff)
- [\$cmd\](#hex-extract)
- [\$cmd\](#hex-find)
- [\$cmd\](#hex-insert)
- [\$cmd\](#hex-patch)
- [\$cmd\](#hex-replace)
- [\$cmd\](#hex-view)
- [\$cmd\](#history)
- [\$cmd\](#hot-files)
- [\$cmd\](#hub)
- [\$cmd\](#impact)
- [\$cmd\](#imports-of)
- [\$cmd\](#index-build)
- [\$cmd\](#index-clear)
- [\$cmd\](#index-gc)
- [\$cmd\](#index-locate)
- [\$cmd\](#index-search)
- [\$cmd\](#index-status)
- [\$cmd\](#index-update)
- [\$cmd\](#insert)
- [\$cmd\](#install-if-missing)
- [\$cmd\](#json-fmt)
- [\$cmd\](#json-get)
- [\$cmd\](#json-keys)
- [\$cmd\](#json-merge)
- [\$cmd\](#json-query)
- [\$cmd\](#json-set)
- [\$cmd\](#line)
- [\$cmd\](#lock)
- [\$cmd\](#locks)
- [\$cmd\](#macro)
- [\$cmd\](#magic)
- [\$cmd\](#map)
- [\$cmd\](#merge-files)
- [\$cmd\](#mkdir)
- [\$cmd\](#mkfile)
- [\$cmd\](#mkfile-from)
- [\$cmd\](#monitor)
- [\$cmd\](#move-with-imports)
- [\$cmd\](#mv)
- [\$cmd\](#neighbors)
- [\$cmd\](#newlines)
- [\$cmd\](#notes)
- [\$cmd\](#notify)
- [\$cmd\](#on-error)
- [\$cmd\](#on-success)
- [\$cmd\](#open-file)
- [\$cmd\](#organize)
- [\$cmd\](#outline)
- [\$cmd\](#pack)
- [\$cmd\](#pack-changed)
- [\$cmd\](#pack-lines)
- [\$cmd\](#parallel)
- [\$cmd\](#patch)
- [\$cmd\](#patch-batch)
- [\$cmd\](#patch-insert)
- [\$cmd\](#patch-lines)
- [\$cmd\](#patch-preview)
- [\$cmd\](#patch-project)
- [\$cmd\](#patch-regex)
- [\$cmd\](#ping)
- [\$cmd\](#pluck)
- [\$cmd\](#post)
- [\$cmd\](#purge-backups)
- [\$cmd\](#redo)
- [\$cmd\](#refs)
- [\$cmd\](#related)
- [\$cmd\](#remove-import)
- [\$cmd\](#rename-bulk)
- [\$cmd\](#rename-safe)
- [\$cmd\](#rename-symbol)
- [\$cmd\](#replace)
- [\$cmd\](#replace-dir)
- [\$cmd\](#replace-ext)
- [\$cmd\](#replace-line)
- [\$cmd\](#replace-project)
- [\$cmd\](#replace-range)
- [\$cmd\](#report-api)
- [\$cmd\](#report-changes)
- [\$cmd\](#report-contributors)
- [\$cmd\](#report-coverage)
- [\$cmd\](#report-errors)
- [\$cmd\](#report-health)
- [\$cmd\](#report-imports)
- [\$cmd\](#report-todos)
- [\$cmd\](#restore)
- [\$cmd\](#resume-download)
- [\$cmd\](#retry)
- [\$cmd\](#revert-patch)
- [\$cmd\](#rm)
- [\$cmd\](#route)
- [\$cmd\](#run)
- [\$cmd\](#scaffold)
- [\$cmd\](#scaffold-add)
- [\$cmd\](#scaffold-api)
- [\$cmd\](#scaffold-component)
- [\$cmd\](#scaffold-context)
- [\$cmd\](#scaffold-hook)
- [\$cmd\](#scaffold-store)
- [\$cmd\](#scaffold-test)
- [\$cmd\](#schedule)
- [\$cmd\](#search-and)
- [\$cmd\](#search-changed)
- [\$cmd\](#search-fuzzy)
- [\$cmd\](#search-history)
- [\$cmd\](#search-multiline)
- [\$cmd\](#search-negative)
- [\$cmd\](#search-or)
- [\$cmd\](#sequence)
- [\$cmd\](#session)
- [\$cmd\](#session-export)
- [\$cmd\](#setup)
- [\$cmd\](#shell)
- [\$cmd\](#show)
- [\$cmd\](#since)
- [\$cmd\](#slice)
- [\$cmd\](#snip)
- [\$cmd\](#snippet)
- [\$cmd\](#sort-lines)
- [\$cmd\](#split-file)
- [\$cmd\](#stale-files)
- [\$cmd\](#stats)
- [\$cmd\](#status)
- [\$cmd\](#strings)
- [\$cmd\](#strip-blank-lines)
- [\$cmd\](#surround)
- [\$cmd\](#symbols)
- [\$cmd\](#tag)
- [\$cmd\](#tail)
- [\$cmd\](#template)
- [\$cmd\](#timer)
- [\$cmd\](#toml-fmt)
- [\$cmd\](#toml-get)
- [\$cmd\](#toml-set)
- [\$cmd\](#toml-to-json)
- [\$cmd\](#to-temp)
- [\$cmd\](#touch)
- [\$cmd\](#trace)
- [\$cmd\](#tree)
- [\$cmd\](#trim)
- [\$cmd\](#trim-dead)
- [\$cmd\](#tui)
- [\$cmd\](#undo)
- [\$cmd\](#unlock)
- [\$cmd\](#upload)
- [\$cmd\](#used-by)
- [\$cmd\](#verify)
- [\$cmd\](#verify-anchor)
- [\$cmd\](#verify-checksum)
- [\$cmd\](#verify-encoding)
- [\$cmd\](#verify-imports)
- [\$cmd\](#verify-json)
- [\$cmd\](#verify-syntax)
- [\$cmd\](#wait)
- [\$cmd\](#watch)
- [\$cmd\](#watch-multi)
- [\$cmd\](#wc)
- [\$cmd\](#web-check)
- [\$cmd\](#web-click)
- [\$cmd\](#web-cookies)
- [\$cmd\](#web-eval)
- [\$cmd\](#web-fetch-clean)
- [\$cmd\](#web-html)
- [\$cmd\](#web-links)
- [\$cmd\](#web-open)
- [\$cmd\](#web-pdf)
- [\$cmd\](#web-scrape)
- [\$cmd\](#web-screenshot)
- [\$cmd\](#web-screenshot-many)
- [\$cmd\](#web-screenshot-set)
- [\$cmd\](#web-search)
- [\$cmd\](#web-search-config)
- [\$cmd\](#web-search-instances)
- [\$cmd\](#web-text)
- [\$cmd\](#web-title)
- [\$cmd\](#web-type)
- [\$cmd\](#web-wait)
- [\$cmd\](#web-ws-status)
- [\$cmd\](#workspace-report)
- [\$cmd\](#ws)
- [\$cmd\](#xml-fmt)
- [\$cmd\](#xml-get)
- [\$cmd\](#xml-to-json)
- [\$cmd\](#xxd)
- [\$cmd\](#yaml-fmt)
- [\$cmd\](#yaml-get)
- [\$cmd\](#yaml-set)
- [\$cmd\](#yaml-to-json)

---

## \$cmd\`n
\\\	ext
Add a named/default import to a file (merges with existing)

Usage: ore.exe add-import [OPTIONS] --from <FROM> <FILE>

Arguments:
  <FILE>  

Options:
  -n, --name <NAME>        Named import to add, e.g. "useState"
  -D, --default <DEFAULT>  Default import, e.g. "React"
  -s, --from <FROM>        Source module, e.g. "react"
      --no-backup          
  -l, --label <LABEL>      
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe after [OPTIONS] <FILE> <PATTERN> <TEXT>

Arguments:
  <FILE>     File to modify
  <PATTERN>  Pattern to match (regex)
  <TEXT>     Text to insert after matching line(s). Use \n for multi-line

Options:
      --first          
  -F, --literal        
  -i, --ignore-case    
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Autonomous agent loop with tool access (research, exploration, edits)

Usage: ore.exe ai-agent [OPTIONS] <TASK>

Arguments:
  <TASK>  

Options:
  -m, --model <MODEL>                    Force a specific model as "provider:model"
      --auto                             Auto-approve destructive tool calls
  -i, --max-iterations <MAX_ITERATIONS>  Max iterations before giving up [default: 10]
      --events-json                      JSON events on stderr
  -q, --quiet                            
  -s, --session <SESSION>                Session name for persistent memory
      --continue                         Continue the "default" session
      --vision <VISION>                  Path to an image file (attaches vision context to the task)
  -h, --help                             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
One-shot AI question with streaming + auto model selection

Usage: ore.exe ai-ask [OPTIONS] [QUESTION]

Arguments:
  [QUESTION]  The question. If omitted, reads from stdin

Options:
  -m, --model <MODEL>      Force a specific model as "provider:model" (bypasses router)
      --no-stream          Disable streaming
      --events-json        Emit events as JSON on stderr (for GUI / tooling)
  -q, --quiet              Silent ΓÇö result only, no event chatter
  -W, --why                Show router decision + timing at end
      --no-tools           Disable agentic tool use (old stateless behavior)
      --auto               Also allow destructive tools (rare for ai-ask)
  -s, --session <SESSION>  Session name for persistent memory
      --continue           Continue the "default" session (shorthand for --session default)
      --vision <VISION>    Path to an image file (enables vision mode)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show current process AI spend vs configured caps

Usage: ore.exe ai-budget [OPTIONS]

Options:
  -j, --json  
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Persistent multi-turn chat (with session storage)

Usage: ore.exe ai-chat [OPTIONS]

Options:
  -s, --session <SESSION>  Session name (persists conversation). Default: 'default' [default: default]
  -m, --model <MODEL>      Force a specific model as "provider:model"
      --no-stream          Disable streaming
  -q, --quiet              Quiet event stream
  -p, --prompt <PROMPT>    One-shot: send this message and exit (skip interactive REPL)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Generate (and optionally apply) a git commit message from the diff

Usage: ore.exe ai-commit-message [OPTIONS]

Options:
      --unstaged       Analyze staged (default) vs all working tree changes
  -m, --model <MODEL>  
  -c, --commit         Actually commit (default: print only)
      --no-stream      
  -q, --quiet          
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
AI configuration (default provider/model/budget/temperature/etc.)

Usage: ore.exe ai-config <COMMAND>

Commands:
  list   
  get    
  set    
  path   
  reset  
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
LLM-quality explanation of what a file does

Usage: ore.exe ai-explain [OPTIONS] <FILE_OR_QUESTION>

Arguments:
  <FILE_OR_QUESTION>  File path to explain, OR a natural-language question about the repo

Options:
  -m, --model <MODEL>    
      --no-stream        
  -q, --quiet            
      --vision <VISION>  Path to an image file (attach visual context alongside the file/question)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Agent that analyzes + patches + verifies + rolls back on failure

Usage: ore.exe ai-fix [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -i, --issue <ISSUE>               What to fix (defaults to "any obvious issues you find")
  -m, --model <MODEL>               
      --auto                        Auto-approve patches
      --max-iters <MAX_ITERATIONS>  [default: 6]
  -q, --quiet                       
  -s, --session <SESSION>           Session name for persistent memory
      --continue                    Continue the "default" session
  -h, --help                        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Full history of every AI call (timestamp, task, model, cost, tokens)

Usage: ore.exe ai-history [OPTIONS]

Options:
  -n, --limit <LIMIT>  Max entries to show (default 50) [default: 50]
  -t, --task <TASK>    Filter by task label (ask, explain, review, fix, refactor, agent, chat, commit-message)
      --today          Only show entries from today
  -j, --json           JSON output
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Manage AI provider API keys (register/unregister/list/test/rotate)

Usage: ore.exe ai-keys <COMMAND>

Commands:
  register    Register (or overwrite) an API key for a provider
  unregister  Remove a stored key
  list        Show which providers have keys registered (env or stored)
  test        Quick liveness test: fetch model list from the provider
  rotate      Replace an existing key
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List models available from a provider (with pricing + context window)

Usage: ore.exe ai-models [OPTIONS] <PROVIDER>

Arguments:
  <PROVIDER>  

Options:
  -r, --refresh  Force refresh from provider (ignore cache)
  -j, --json     JSON output
  -h, --help     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Build a task-focused AI prompt (finds relevant files, packs them)

Usage: ore.exe ai-prompt [OPTIONS] <TASK> [PATH]

Arguments:
  <TASK>  The task you're describing to the AI (used to select relevant files)
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>              
  -x, --exclude <EXCLUDE>      
  -n, --max-files <MAX_FILES>  Max files to include [default: 12]
  -o, --output <OUTPUT>        Output file (default: stdout)
      --copy                   Copy to clipboard
      --with-digest            Include structural digest at the top
      --condense               Compress included file content (medium condense)
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Edit / list / reset the AI system prompts

Usage: ore.exe ai-prompts <COMMAND>

Commands:
  list   
  show   
  path   
  edit   
  reset  
  diff   
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show all configured/available providers

Usage: ore.exe ai-providers

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Full-text search across all past AI session messages

Usage: ore.exe ai-recall [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Search term (substring match across all session messages)

Options:
  -n, --limit <LIMIT>      Max results [default: 20]
  -s, --session <SESSION>  Filter to a specific session
  -j, --json               JSON output
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Multi-step agent that plans ΓåÆ executes ΓåÆ verifies a refactor intent

Usage: ore.exe ai-refactor [OPTIONS] <FILE> <INTENT>

Arguments:
  <FILE>    
  <INTENT>  What you want changed (natural language)

Options:
  -m, --model <MODEL>               
      --auto                        
      --max-iters <MAX_ITERATIONS>  [default: 10]
  -q, --quiet                       
  -s, --session <SESSION>           Session name for persistent memory
      --continue                    Continue the "default" session
  -h, --help                        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
AI code review with severity + line refs

Usage: ore.exe ai-review [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -m, --model <MODEL>  
      --no-stream      
  -q, --quiet          
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Dry-run: show exactly what an agent would retrieve for a query

Usage: ore.exe ai-search-test <QUERY>

Arguments:
  <QUERY>  

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Manage AI chat sessions (list/show/rm)

Usage: ore.exe ai-session <COMMAND>

Commands:
  list  List saved sessions with message counts
  show  Show a session's messages
  rm    Delete a session
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Cumulative token + cost usage across all AI calls

Usage: ore.exe ai-usage [OPTIONS]

Options:
  -d, --days <DAYS>  Filter to last N days (0 = all-time) [default: 0]
  -j, --json         
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
User-defined command aliases

Usage: ore.exe alias <COMMAND>

Commands:
  list  List all aliases
  add   Add an alias: ore alias add <name> "<commands...>"
  rm    Remove an alias
  path  Show the aliases file path
  run   Run an alias by name (mainly for scripting; usually invoked as `ore <name>`)
  show  Show what an alias would expand to
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Files with the highest git churn

Usage: ore.exe analyze-churn [OPTIONS]

Options:
  -p, --path <PATH>    Restrict to a subdirectory
  -s, --since <SINCE>  
  -n, --top <TOP>      [default: 20]
  -j, --json           
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Circular import detection

Usage: ore.exe analyze-circular [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -n, --top <TOP>          [default: 20]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Cyclomatic complexity per function (above threshold)

Usage: ore.exe analyze-complexity [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>              
  -x, --exclude <EXCLUDE>      
  -n, --top <TOP>              [default: 20]
  -t, --threshold <THRESHOLD>  Complexity threshold (default 10) [default: 10]
  -j, --json                   
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Coupling score (fanout+fanin ΓÇö most entangled files)

Usage: ore.exe analyze-coupling [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -n, --top <TOP>          [default: 20]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Exported symbols never imported anywhere

Usage: ore.exe analyze-dead-exports [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -k, --keep <KEEP>        Additional entry-point patterns (never treated as dead). Repeat
  -n, --top <TOP>          [default: 50]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Duplicated code blocks across files

Usage: ore.exe analyze-duplication [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>              
  -x, --exclude <EXCLUDE>      
  -m, --min-lines <MIN_LINES>  Minimum lines in a duplicated block [default: 6]
  -n, --top <TOP>              [default: 20]
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Export counts per file

Usage: ore.exe analyze-exports [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -n, --top <TOP>          [default: 30]
  -j, --json               
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Hotspot analysis (churn ├ù complexity)

Usage: ore.exe analyze-hotspot [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -s, --since <SINCE>      
  -n, --top <TOP>          [default: 20]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Import graph: fanout / fanin per file

Usage: ore.exe analyze-imports [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -s, --sort <SORT>        Sort by: fanout (default), fanin, name [default: fanout]
  -n, --top <TOP>          [default: 20]
  -j, --json               
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TS type coverage (any-density)

Usage: ore.exe analyze-type-coverage [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -x, --exclude <EXCLUDE>  
  -n, --top <TOP>          [default: 20]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run API tests from a .ore-api spec file

Usage: ore.exe api-test [OPTIONS] <SPEC>

Arguments:
  <SPEC>  

Options:
      --fail-fast          
  -v, --verbose            
  -t, --timeout <TIMEOUT>  [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Apply a .patch/.diff file (via git apply, with backups)

Usage: ore.exe apply-patch [OPTIONS] <PATCH>

Arguments:
  <PATCH>  .patch or .diff file

Options:
  -p, --path <PATH>    Path to apply within (default: current dir)
      --no-backup      Skip backups
  -l, --label <LABEL>  Backup label
  -R, --reverse        Reverse-apply (undo the patch)
      --check          Dry-run: just check if it would apply cleanly
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe backup [OPTIONS] <FILE>

Arguments:
  <FILE>  File to back up

Options:
  -l, --label <LABEL>  Label suffix (e.g. "CAMFIX" -> file.ext.bakCAMFIX). Defaults to timestamp
      --list           Just list existing backups, don't create new
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe before [OPTIONS] <FILE> <PATTERN> <TEXT>

Arguments:
  <FILE>     File to modify
  <PATTERN>  Pattern to match (regex)
  <TEXT>     Text to insert before matching line(s). Use \n for multi-line

Options:
      --first          Match only the first occurrence
  -F, --literal        Treat pattern as literal
  -i, --ignore-case    Case-insensitive
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Benchmark a command (runs, min/mean/p50/p95/p99/max)

Usage: ore.exe benchmark [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to benchmark

Options:
  -n, --runs <RUNS>      Number of runs [default: 10]
  -w, --warmup <WARMUP>  Warmup runs (not counted) [default: 2]
  -v, --verbose          Show per-run timings
      --strict           Fail if any run errored
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Benchmark a URL (N reqs, concurrency, p50/p95/p99)

Usage: ore.exe bench-url [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -n, --count <COUNT>              Total requests to send [default: 100]
  -c, --concurrency <CONCURRENCY>  Concurrency [default: 10]
  -X, --method <METHOD>            Method [default: GET]
  -t, --timeout <TIMEOUT>          [default: 30]
      --warmup <WARMUP>            Warmup requests (not counted) [default: 5]
  -h, --help                       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Concatenate binary files

Usage: ore.exe bin-cat --output <OUTPUT> <FILES>...

Arguments:
  <FILES>...  Files to concatenate (in order)

Options:
  -o, --output <OUTPUT>  Output file (required ΓÇö raw bytes)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract byte range to a new file

Usage: ore.exe bin-slice --output <OUTPUT> <FILE> <START> <END>

Arguments:
  <FILE>   
  <START>  Start offset (inclusive)
  <END>    End offset (exclusive)

Options:
  -o, --output <OUTPUT>  Output file (required ΓÇö raw bytes)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Byte frequency + entropy + histogram

Usage: ore.exe bin-stats [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -H, --histogram  Show byte-frequency histogram (top 16)
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Transitive impact of changing a symbol (depth-based)

Usage: ore.exe blast-radius [OPTIONS] <SYMBOL> [ROOT]

Arguments:
  <SYMBOL>  Symbol name (function/const/class/type)
  [ROOT]    [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -d, --depth <DEPTH>      Max transitive depth [default: 3]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Bookmarks: named file:line references for quick navigation

Usage: ore.exe bookmark [OPTIONS] <COMMAND>

Commands:
  set    Set a bookmark: ore bookmark set <name> <file:line> [-m "description"]
  get    Get a bookmark by name (prints file:line)
  rm     Remove a bookmark
  list   List all bookmarks
  jump   Jump: print file content around the bookmarked line
  clear  Clear all bookmarks
  help   Print this message or the help of the given subcommand(s)

Options:
      --dir <DIR>  Working directory (default: current dir)
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe cat [OPTIONS] <FILE>

Arguments:
  <FILE>  File to print

Options:
  -n, --number       Show line numbers
      --binary       Force print even if binary
  -g, --grep <GREP>  Show only lines matching pattern
      --raw          Print raw bytes without decoding (for binary inspection)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Print lines around every match of a pattern (context viewer)

Usage: ore.exe cat-around [OPTIONS] <FILE> <PATTERN>

Arguments:
  <FILE>     File to search
  <PATTERN>  Pattern to search for (substring or regex with --regex)

Options:
  -C, --context <CONTEXT>  Lines of context before and after each match (default: 5) [default: 5]
  -n, --line-numbers       Show line numbers
  -i, --ignore-case        Case-insensitive matching
  -x, --regex              Treat pattern as a regular expression
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Check that a set of tools are available on PATH

Usage: ore.exe check-deps [OPTIONS]

Options:
  -t, --tools <TOOLS>  Comma-separated list of tools to check (default: common ones)
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Compute file checksum (sha256/md5/crc32/all)

Usage: ore.exe checksum [OPTIONS] [FILES]...

Arguments:
  [FILES]...  File(s) to checksum

Options:
  -a, --algo <ALGO>  Algorithm [default: sha256] [possible values: sha256, md5, crc32, all]
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Bulk URL health checker (2xx/3xx/4xx/5xx)

Usage: ore.exe check-urls [OPTIONS] [URLS]...

Arguments:
  [URLS]...  

Options:
  -f, --file <FILE>        
  -l, --limit <LIMIT>      [default: 10]
  -t, --timeout <TIMEOUT>  [default: 10]
      --fallback-get       Also try GET on failures (some servers block HEAD)
  -F, --failures-only      Only show non-OK results
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Split a file into per-function/class/section chunks with a manifest

Usage: ore.exe chunk [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -b, --by <BY>                  Chunk boundary strategy [default: function] [possible values: function, class, export, section]
  -o, --output-dir <OUTPUT_DIR>  Output directory (default: "<stem>-chunks/")
      --manifest                 Also write a manifest (chunks.json) listing all chunks with metadata
      --dry-run                  
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe collapse-blank-lines [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -m, --max <MAX>      Max consecutive blank lines to keep (default 1) [default: 1]
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run npm/yarn/pnpm script, cache output

Usage: ore.exe compile-node [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -r, --script <SCRIPT>  npm script to run (default: "build") [default: build]
      --pm <PM>          Use yarn / pnpm instead of npm [default: npm]
  -s, --stream           
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run cargo check/build, parse errors, cache them

Usage: ore.exe compile-rust [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -c, --check        Use `cargo check` (fast) instead of `cargo build`
  -a, --args <ARGS>  Extra cargo args
  -s, --stream       Stream live output
  -j, --json         JSON parsed output
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run tsc --noEmit, parse errors, cache them

Usage: ore.exe compile-ts [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to project (with tsconfig.json) [default: .]

Options:
  -a, --args <ARGS>  Extra tsc args
  -s, --stream       Stream output live
  -j, --json         JSON output (parsed errors)
  -f, --file <FILE>  Only show errors from this file (substring match)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Condense a file (strip comments/blanks/whitespace to save tokens)

Usage: ore.exe condense [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -l, --level <LEVEL>    How aggressive to condense [default: medium] [possible values: light, medium, aggressive]
  -o, --output <OUTPUT>  Write to file
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Global config (get/set/list persistent settings)

Usage: ore.exe config <COMMAND>

Commands:
  list   List all config values
  get    Get a specific value
  set    Set a value
  rm     Remove a value
  path   Show the path to the config file
  reset  Reset config (delete file)
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find near-duplicate function bodies across the codebase

Usage: ore.exe consolidate [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>                
  -x, --exclude <EXCLUDE>        
  -m, --min-len <MIN_LEN>        Minimum body length in chars to consider [default: 80]
  -s, --similarity <SIMILARITY>  Similarity threshold (0.0-1.0) [default: 0.85]
  -n, --top <TOP>                [default: 20]
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Copy content (stdin or file) to system clipboard

Usage: ore.exe copy [OPTIONS] [FILE]

Arguments:
  [FILE]  File to copy (omit for stdin)

Options:
  -t, --tee   Also print to stdout (tee behavior)
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe count [OPTIONS] <PATTERN> [PATH]

Arguments:
  <PATTERN>  Pattern (regex by default)
  [PATH]     Path (file or directory) [default: .]

Options:
  -F, --literal            
  -i, --ignore-case        
  -w, --word               
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -v, --verbose            Show per-file counts (default: only totals)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Copy a file or directory (auto-backup on overwrite)

Usage: ore.exe cp [OPTIONS] <SRC> <DST>

Arguments:
  <SRC>  Source file or directory
  <DST>  Destination path

Options:
  -r, --recursive      Recursive (for directories)
  -y, --yes            Bypass confirmation
      --force          Force overwrite
      --no-backup      Skip backup on overwrite
  -l, --label <LABEL>  Backup label
      --dry-run        Dry run
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Crawl a URL by following links (bounded depth + count)

Usage: ore.exe crawl [OPTIONS] <URL>

Arguments:
  <URL>  Starting URL

Options:
  -n, --max <MAX>                Max pages to fetch [default: 50]
  -d, --depth <DEPTH>            Max link-follow depth [default: 2]
      --same-domain              Only follow links on the SAME domain as start URL
  -o, --output-dir <OUTPUT_DIR>  Save fetched pages here (one file per URL)
  -t, --timeout <TIMEOUT>        [default: 20]
  -v, --verbose                  Verbose (show link extraction per page)
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
CSV: filter rows by column=value

Usage: ore.exe csv-filter [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -w, --where <FILTERS>  column=value (exact). Repeatable, all must match
      --no-header        
  -d, --delim <DELIM>    [default: ,]
  -o, --output <OUTPUT>  Output file (default: stdout)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
CSV: query a column with optional --where filters

Usage: ore.exe csv-query [OPTIONS] <FILE> <COLUMN>

Arguments:
  <FILE>    
  <COLUMN>  Column name to show

Options:
  -w, --where <FILTERS>  Filter: column=value (exact match). Repeatable
      --no-header        No header row (columns are 0,1,2,...)
  -d, --delim <DELIM>    Delimiter (default comma) [default: ,]
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
CSV: select subset of columns

Usage: ore.exe csv-select [OPTIONS] <FILE> <COLUMNS>

Arguments:
  <FILE>     
  <COLUMNS>  Comma-separated columns (names or indexes)

Options:
      --no-header        
  -d, --delim <DELIM>    [default: ,]
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
CSV: per-column stats (unique count, empties, numeric?)

Usage: ore.exe csv-stats [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --no-header      
  -d, --delim <DELIM>  [default: ,]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
CSV: convert to JSON (array of objects)

Usage: ore.exe csv-to-json [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --no-header        
  -d, --delim <DELIM>    [default: ,]
  -o, --output <OUTPUT>  
  -c, --compact          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe dedup-lines [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -a, --adjacent       Only remove ADJACENT duplicates (like uniq)
  -i, --ignore-case    Ignore case when comparing
  -t, --trim           Ignore leading/trailing whitespace when comparing
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe delete-lines [OPTIONS] <FILE> <RANGE>

Arguments:
  <FILE>   File to modify
  <RANGE>  Line or range: "42", "10:20", "10-20"

Options:
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe diff [OPTIONS] <FILE_A> [FILE_B]

Arguments:
  <FILE_A>  First file (or "backup" to diff current file against latest backup)
  [FILE_B]  Second file (omit if using --backup)

Options:
      --backup             Diff current file against its latest backup
      --label <LABEL>      Specific backup label to compare against
  -n, --number             Show line numbers
  -C, --context <CONTEXT>  Number of context lines (default 3) [default: 3]
  -s, --stats              Stats only (additions, deletions counts)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Diff two directory trees

Usage: ore.exe diff-dirs [OPTIONS] <DIR_A> <DIR_B>

Arguments:
  <DIR_A>  
  <DIR_B>  

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -H, --hidden             
      --no-ignore          
  -C, --content            Compare by content hash (default is size + mtime; slower but exact)
  -v, --verbose            Verbose (show unchanged too)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Diff with configurable ignore flags (whitespace, blank, case, comments)

Usage: ore.exe diff-ignore [OPTIONS] <FILE_A> <FILE_B>

Arguments:
  <FILE_A>  
  <FILE_B>  

Options:
  -w, --whitespace         Ignore whitespace
  -b, --blank-lines        Ignore blank lines
  -i, --case               Ignore case
  -c, --comments           Ignore comments (// and #)
  -C, --context <CONTEXT>  Context lines [default: 3]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Semantic diff (ignores whitespace + comments)

Usage: ore.exe diff-semantic [OPTIONS] <FILE_A> <FILE_B>

Arguments:
  <FILE_A>  
  <FILE_B>  

Options:
  -v, --verbose  Show identical files output too
  -h, --help     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
English summary of what changed between two refs

Usage: ore.exe diff-summary [OPTIONS]

Options:
  -f, --from <FROM>    First ref (default: HEAD~5) [default: HEAD~5]
  -t, --to <TO>        Second ref (default: HEAD) [default: HEAD]
  -s, --style <STYLE>  English summary style: simple | conventional (default simple) [default: simple]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Word/character-level diff

Usage: ore.exe diff-word [OPTIONS] <FILE_A> <FILE_B>

Arguments:
  <FILE_A>  
  <FILE_B>  

Options:
  -c, --chars  Character-level instead of word
  -h, --help   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Codebase digest for AI (structural summary, per-file exports/imports)

Usage: ore.exe digest [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>                  
  -x, --exclude <EXCLUDE>          
  -o, --output <OUTPUT>            Output file (default: stdout)
      --with-imports               Include imports section per file
      --with-tree                  Include tree overview at the top
      --with-stats                 Include per-file size/lines
      --max-exports <MAX_EXPORTS>  Cap: skip files with more than N exports (usually barrel files) [default: 0]
      --only <ONLY>                Only include files matching this substring
  -h, --help                       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
DNS resolution

Usage: ore.exe dns [OPTIONS] <HOST>

Arguments:
  <HOST>  

Options:
  -p, --port <PORT>  [default: 80]
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Download a URL to a file

Usage: ore.exe download [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -o, --output <OUTPUT>    
      --force              
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 300]
      --proxy <PROXY>      
  -y, --yes                
      --no-progress        Disable progress bar (useful for scripts)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Parallel download of many URLs

Usage: ore.exe download-many [OPTIONS] [URLS]...

Arguments:
  [URLS]...  

Options:
  -f, --file <FILE>              
  -o, --output-dir <OUTPUT_DIR>  [default: .]
      --force                    
  -l, --limit <LIMIT>            [default: 4]
  -H, --header <HEADERS>         
  -t, --timeout <TIMEOUT>        [default: 300]
  -r, --rate <RATE>              [default: 0]
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe encoding [OPTIONS] <FILE>

Arguments:
  <FILE>  File to inspect or convert

Options:
  -t, --to <TO>        Convert to this encoding (utf-8, utf-16le, utf-16be, windows-1252, etc.)
      --no-backup      Skip backup on conversion
  -l, --label <LABEL>  Backup label
      --bom            Add BOM after conversion (only meaningful for utf-8, utf-16)
      --strip-bom      Strip BOM after conversion
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
.env: diff two files

Usage: ore.exe env-diff [OPTIONS] <FILE_A> <FILE_B>

Arguments:
  <FILE_A>  
  <FILE_B>  

Options:
  -D, --only-diff  Only show differing keys (skip identical)
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
.env: get value by key (or list all)

Usage: ore.exe env-get <FILE> [KEY]

Arguments:
  <FILE>  
  [KEY]   Key (omit to list all)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
.env: set / delete key

Usage: ore.exe env-set [OPTIONS] <FILE> <KEY> <VALUE>

Arguments:
  <FILE>   
  <KEY>    
  <VALUE>  

Options:
      --no-backup      
  -l, --label <LABEL>  
      --delete         Delete the key instead of setting it
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Replay the last cached compile errors (grouped/filtered/JSON)

Usage: ore.exe errors-last [OPTIONS]

Options:
  -w, --warnings     Show warnings too
  -g, --group        Group by file
  -r, --raw          Show raw output
  -j, --json         JSON
  -f, --file <FILE>  Only errors in this file (substring)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Heuristic English explanation of what a file does

Usage: ore.exe explain <FILE>

Arguments:
  <FILE>  

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract line ranges from one or more files (multi-file, multi-range, labels)

Usage: ore.exe extract [OPTIONS] [FILE] [RANGES]

Arguments:
  [FILE]    File to extract from (omit if using --spec or --spec-file)
  [RANGES]  Line ranges. Comma-separated. Formats: "10", "10-30", "10:30", "10-30,50-70,100"

Options:
      --spec <SPEC>            Multi-file spec: "file1:10-30,file2:5-15,file3:100-200"
      --spec-file <SPEC_FILE>  Load specs from a file (one spec per line, format: "path:range1,range2")
  -L, --label                  Prepend a === file:range === label before each chunk
  -C, --context <CONTEXT>      Include N lines of context before/after each range [default: 0]
  -N, --number                 Show line numbers
  -m, --merge                  Merge overlapping/adjacent ranges within same file
  -o, --output <OUTPUT>        Write output to file instead of stdout
      --plain                  Suppress colors (for pipe/redirect)
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract a named function/class into a new file, re-export from source

Usage: ore.exe extract-fn [OPTIONS] --output <OUTPUT> <FILE> <SYMBOL>

Arguments:
  <FILE>    Source file
  <SYMBOL>  Symbol name to extract

Options:
  -o, --output <OUTPUT>  Output file (new file where the symbol will live)
  -r, --reexport         Add re-export from source to output ("export { foo } from './output'")
  -i, --carry-imports    Include imports from source file in the new output
      --no-backup        
  -l, --label <LABEL>    
      --dry-run          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
HTTP GET a URL (with headers, output, pretty JSON)

Usage: ore.exe fetch [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 30]
      --no-redirect        
      --proxy <PROXY>      
  -o, --output <OUTPUT>    
  -i, --include-headers    
  -q, --no-body            Skip body output (status + headers only)
  -j, --pretty             
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Parallel HTTP GET of many URLs (rate-limit + save)

Usage: ore.exe fetch-many [OPTIONS] [URLS]...

Arguments:
  [URLS]...  

Options:
  -f, --file <FILE>              
  -l, --limit <LIMIT>            [default: 5]
  -H, --header <HEADERS>         
  -t, --timeout <TIMEOUT>        [default: 30]
  -o, --output-dir <OUTPUT_DIR>  
  -v, --verbose                  
  -r, --rate <RATE>              Rate limit (requests per second, 0 = no limit) [default: 0]
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Get remote file size (one or many URLs) without downloading

Usage: ore.exe filesize [OPTIONS] [URLS]...

Arguments:
  [URLS]...  One or more URLs

Options:
  -t, --timeout <TIMEOUT>  [default: 10]
  -q, --quiet              Raw bytes only, one per line (for scripts)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe find [OPTIONS] <PATTERN> [PATH]

Arguments:
  <PATTERN>  Pattern to search for (regex by default)
  [PATH]     Path to search (file or directory) [default: .]

Options:
  -F, --literal          Treat pattern as literal string, not regex
  -i, --ignore-case      Case-insensitive search
  -w, --word             Whole word match only
  -H, --hidden           Include hidden files
      --no-ignore        Don't respect .gitignore
      --binary           Search binary files too
  -l, --files-only       Show only file names with matches
  -c, --count-only       Show only match count per file
  -B, --before <BEFORE>  Show N lines before match [default: 0]
  -A, --after <AFTER>    Show N lines after match [default: 0]
  -e, --ext <EXT>        File extension filter (e.g. "ts,tsx,rs")
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find duplicate files by content hash

Usage: ore.exe find-dupes [OPTIONS] <PATHS>...

Arguments:
  <PATHS>...  Root paths (one or more)

Options:
  -e, --ext <EXT>            
  -x, --exclude <EXCLUDE>    
  -H, --hidden               
      --no-ignore            
  -s, --min-size <MIN_SIZE>  [default: 1]
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Inline all re-exports of a hub into a single file

Usage: ore.exe flatten-hub [OPTIONS] <HUB>

Arguments:
  <HUB>  Hub barrel file

Options:
  -i, --carry-imports  Include imports from each source into flattened file
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Set/show/clear a focus path for the workspace

Usage: ore.exe focus <COMMAND>

Commands:
  set    Set the focus path (subsequent commands can default to this)
  show   Show current focus
  clear  Clear focus
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Amend the last commit

Usage: ore.exe git-amend [OPTIONS]

Options:
  -m, --message <MESSAGE>  New commit message (leaves existing if omitted)
  -n, --no-edit            Also add all staged changes (like --amend without --no-edit)
  -y, --yes                
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Auto-generate + apply commit message from staged diff

Usage: ore.exe git-auto-commit [OPTIONS]

Options:
  -a, --all              Auto-stage all modified tracked files first (like git commit -a)
  -p, --preview          Only preview the message; don't commit
      --conventional     Force conventional-commits style
      --simple           Force simple English
  -S, --subject-only     Subject line only, no body
  -e, --edit             Open message in $EDITOR before committing
  -y, --yes              Bypass confirmation
      --only <ONLY>      Only include files matching this substring (filter which are staged & committed)
      --except <EXCEPT>  Exclude files matching
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Generate a commit message from diff (don't commit)

Usage: ore.exe git-auto-message [OPTIONS]

Options:
  -s, --staged        Analyze staged changes (default: unstaged/working tree vs HEAD)
      --conventional  Force conventional-commits style
      --simple        Force simple English style
  -S, --subject-only  Skip body, subject line only
  -h, --help          Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Git blame with range support

Usage: ore.exe git-blame [OPTIONS] <FILE>

Arguments:
  <FILE>  File to blame

Options:
  -L, --range <RANGE>  Only a line range: "10", "10-20", "10:20"
  -e, --email          Show email instead of name
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List changed files (with filters)

Usage: ore.exe git-changed [OPTIONS]

Options:
      --only <ONLY>              Only files matching (substring)
      --except <EXCEPT>          Exclude files matching
      --starts <STARTS>          Only files whose basename starts with
      --changed-in <CHANGED_IN>  Only files in this subdirectory
      --matching <MATCHING>      Only files whose content contains this substring
      --staged                   Include only staged
      --unstaged                 Include only unstaged
      --untracked                Include only untracked
  -p, --paths-only               Print paths only, no color/decoration (for piping)
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Generate CHANGELOG markdown from git history

Usage: ore.exe git-changelog [OPTIONS]

Options:
  -s, --since <SINCE>    Since tag/commit/date (e.g. "v1.0.0", "HEAD~50", "2 weeks ago")
  -u, --until <UNTIL>    Until tag/commit/date
  -g, --group            Group by conventional-commit type (feat, fix, chore, etc.)
  -o, --output <OUTPUT>  Write to file
  -H, --hash             Include commit hash
  -a, --author           Include author name
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Delete merged/orphaned local branches

Usage: ore.exe git-cleanup-branches [OPTIONS]

Options:
  -b, --base <BASE>      Branch to consider "the trunk" (default: main, then master)
      --include-orphans  Also delete branches with no upstream
      --force            Force-delete unmerged branches too
      --dry-run          
  -y, --yes              
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Commit files with filters (--only/--except/--starts/--changed-in)

Usage: ore.exe git-commit [OPTIONS] --message <MESSAGE>

Options:
  -m, --message <MESSAGE>        Commit message
      --all                      Commit all currently modified tracked files (git commit -am)
      --only <ONLY>              Only files matching (substring). Auto-stages then commits
      --except <EXCEPT>          Exclude files matching
      --starts <STARTS>          Only files whose basename starts with
      --changed-in <CHANGED_IN>  Only files in this subdirectory
      --matching <MATCHING>      Only files whose content contains this substring
      --dry-run                  Preview which files would be committed
  -y, --yes                      Bypass confirmation
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Compose commit with your subject + generated body

Usage: ore.exe git-commit-body [OPTIONS] <SUBJECT>

Arguments:
  <SUBJECT>  Subject line you want

Options:
  -u, --unstaged  Analyze staged (default: staged)
  -p, --preview   Just preview, don't commit
  -y, --yes       
  -h, --help      Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show git diff (staged/unstaged, per-file, or commit)

Usage: ore.exe git-diff [OPTIONS] [FILE]

Arguments:
  [FILE]  Specific file (default: entire repo)

Options:
  -s, --staged           Show staged diff instead of unstaged
  -c, --commit <COMMIT>  Diff against a specific commit
      --stat             Stats only
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create a fixup commit targeting a previous SHA (with optional autosquash)

Usage: ore.exe git-fixup [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Target commit SHA (or ref like HEAD~3)

Options:
  -r, --rebase  Also start an interactive autosquash rebase after creating the fixup commit
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Commit history for a file

Usage: ore.exe git-history [OPTIONS] <FILE>

Arguments:
  <FILE>  File whose history to show

Options:
  -n, --limit <LIMIT>  Max commits (default 20) [default: 20]
  -p, --patch          Show patches (full diff per commit)
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Git log with filters (--mine, --author, --grep, --since, --until)

Usage: ore.exe git-log [OPTIONS]

Options:
  -n, --limit <LIMIT>    Max commits [default: 20]
  -g, --graph            Graph view
      --mine             Only my commits (matches git config user.name)
      --author <AUTHOR>  Filter by author substring
      --grep <GREP>      Filter by commit message substring
      --since <SINCE>    Since date (e.g. "2 weeks ago", "2025-01-01")
      --until <UNTIL>    Until date
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Generate release notes for a version

Usage: ore.exe git-release-notes [OPTIONS] <VERSION>

Arguments:
  <VERSION>  Version/tag being released

Options:
  -p, --previous <PREVIOUS>  Previous tag/ref to compare from (default: previous tag)
  -o, --output <OUTPUT>      
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Search git history by content or commit message

Usage: ore.exe git-search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Text to search in git history

Options:
      --messages       Search commit messages
      --content        Search introduced/removed content (default)
  -n, --limit <LIMIT>  Limit results [default: 50]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Stage files with filters (--only/--except/--starts/--changed-in)

Usage: ore.exe git-stage [OPTIONS]

Options:
      --all                      Stage all changed files
      --only <ONLY>              Only files matching (substring)
      --except <EXCEPT>          Exclude files matching
      --starts <STARTS>          Only files whose basename starts with
      --changed-in <CHANGED_IN>  Only files in this subdirectory
      --matching <MATCHING>      Only files whose content contains this substring
      --dry-run                  Preview which files would be staged
  -y, --yes                      Bypass confirmation
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Named stash save/list/apply/pop/drop/show

Usage: ore.exe git-stash-named <COMMAND>

Commands:
  save   Save current changes with a named label
  list   List named stashes
  apply  Apply a named stash (keeps it in stash list)
  pop    Pop a named stash (removes it)
  drop   Drop a named stash without applying
  show   Show contents of a named stash
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Git working tree status

Usage: ore.exe git-status [OPTIONS]

Options:
  -s, --short  Short format (single-char per file)
  -h, --help   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Suggest a commit message and explain the rationale

Usage: ore.exe git-suggest-commit [OPTIONS]

Options:
  -s, --staged  
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Undo last N commits (soft/mixed/hard)

Usage: ore.exe git-undo-commit [OPTIONS]

Options:
  -n, --count <COUNT>  How many commits to undo (default 1) [default: 1]
      --hard           Hard reset (loses changes) ΓÇö default is soft (keeps changes staged)
      --mixed          Mixed reset (keeps changes unstaged)
  -y, --yes            
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Contributors to a file (ranked by commits)

Usage: ore.exe git-who <FILE>

Arguments:
  <FILE>  File to analyze

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe head [OPTIONS] <FILE>

Arguments:
  <FILE>  File to read

Options:
  -n, --lines <LINES>  Number of lines (default 10) [default: 10]
  -N, --number         Show line numbers
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show response headers only

Usage: ore.exe headers [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 10]
  -g, --get                Use GET instead of HEAD (some servers don't support HEAD)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Codebase health report (score, todos, code smells, meta files)

Usage: ore.exe health [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          Extensions to include (comma-separated)
  -x, --exclude <EXCLUDE>  Excludes
  -j, --json               JSON output
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Delete a byte range from a file

Usage: ore.exe hex-delete [OPTIONS] <FILE> <OFFSET> <LENGTH>

Arguments:
  <FILE>    
  <OFFSET>  Offset to delete FROM
  <LENGTH>  Number of bytes to delete

Options:
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Binary diff with offset + hex dump

Usage: ore.exe hex-diff [OPTIONS] <FILE_A> <FILE_B>

Arguments:
  <FILE_A>  
  <FILE_B>  

Options:
  -n, --max <MAX>          Max differences to show (0 = all) [default: 50]
  -C, --context <CONTEXT>  Context bytes around each diff [default: 0]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract a byte range from a file

Usage: ore.exe hex-extract [OPTIONS] <FILE> <OFFSET> <LENGTH>

Arguments:
  <FILE>    
  <OFFSET>  Start offset
  <LENGTH>  Length in bytes

Options:
  -o, --output <OUTPUT>  Output file (omit for stdout as hex)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find hex pattern in binary (with ?? wildcards)

Usage: ore.exe hex-find [OPTIONS] <FILE> <PATTERN>

Arguments:
  <FILE>     
  <PATTERN>  Hex pattern (supports ?? wildcards, spaces optional). Examples: "deadbeef", "de ad ?? ef"

Options:
  -C, --context <CONTEXT>  Show N bytes of context around each match [default: 16]
  -n, --max <MAX>          Max matches to show (0 = unlimited) [default: 0]
  -o, --offsets-only       Only show offsets, no hex dump
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Insert bytes at offset (existing bytes shift)

Usage: ore.exe hex-insert [OPTIONS] <FILE> <OFFSET> <BYTES>

Arguments:
  <FILE>    
  <OFFSET>  Offset to insert AT (existing bytes shift forward)
  <BYTES>   Hex bytes to insert

Options:
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Write hex bytes at a specific offset (same-length or extend)

Usage: ore.exe hex-patch [OPTIONS] <FILE> <OFFSET> <BYTES>

Arguments:
  <FILE>    
  <OFFSET>  Offset (supports 0x, k, m, g)
  <BYTES>   Hex bytes to write at that offset

Options:
      --extend         Extend file if offset is past EOF (pad with zeros)
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Replace hex bytes (same-length in-place)

Usage: ore.exe hex-replace [OPTIONS] <FILE> <FIND> <REPLACE>

Arguments:
  <FILE>     
  <FIND>     Hex pattern to find (wildcards ?? allowed)
  <REPLACE>  Hex bytes to replace with (must NOT contain wildcards, MUST be same length)

Options:
  -a, --all            Replace all occurrences (default: fail if not exactly 1)
  -n, --nth <NTH>      Replace only Nth (1-indexed)
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
View file as hex+ASCII (paged, with offset/length/width)

Usage: ore.exe hex-view [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --offset <OFFSET>  Start offset (supports 0x, k, m, g suffixes)
  -l, --length <LENGTH>  Byte count to show (0 = to end) [default: 512]
  -w, --width <WIDTH>    Bytes per line [default: 16]
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show operation history (backups, patches, deletes) ΓÇö auto-recorded

Usage: ore.exe history [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -f, --file <FILE>  Filter to entries for a specific file
  -a, --all          Include undone entries
  -n, --top <TOP>    Max entries [default: 30]
  -j, --json         JSON output
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Files with highest git churn (hotspots for refactoring)

Usage: ore.exe hot-files [OPTIONS]

Options:
  -s, --since <SINCE>  [default: "90 days ago"]
  -p, --path <PATH>    
  -n, --top <TOP>      [default: 20]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create a barrel index.ts (or mod.rs / __init__.py) from a folder

Usage: ore.exe hub [OPTIONS] <DIR>

Arguments:
  <DIR>  Directory of files to build a barrel index for

Options:
  -o, --output <OUTPUT>  Output file (default: <dir>/index.ts or <dir>/mod.rs based on contents)
  -E, --exported-only    Only include exported symbols
  -s, --star             Star-export (export * from ...) instead of named re-exports
      --force            Overwrite existing hub file
      --dry-run          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Transitive impact if a file changes (upstream propagation)

Usage: ore.exe impact [OPTIONS] <FILE> [ROOT]

Arguments:
  <FILE>  Target file
  [ROOT]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -d, --depth <DEPTH>      Max depth of upstream traversal (default 5) [default: 5]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show what a file imports (with optional resolution)

Usage: ore.exe imports-of [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -r, --resolve  Resolve relative imports to real files
  -j, --json     JSON output
  -h, --help     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Build a SQLite index of files, symbols, and imports (fast reuse across commands)

Usage: ore.exe index-build [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -f, --force              Force full rebuild even if index exists
      --gitignore          Add .ore-index/ to .gitignore automatically
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Delete the index database

Usage: ore.exe index-clear [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -y, --yes   
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Remove orphaned entries + vacuum

Usage: ore.exe index-gc [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Print the index database path

Usage: ore.exe index-locate [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Fast symbol search via the index

Usage: ore.exe index-search [OPTIONS] <PATTERN> [ROOT]

Arguments:
  <PATTERN>  Symbol name substring
  [ROOT]     [default: .]

Options:
  -k, --kind <KIND>  Filter by kind (fn, class, hook, comp, const, type, iface, enum, struct, trait, mod)
  -E, --exported     Only exported
  -n, --top <TOP>    Max results [default: 50]
  -j, --json         JSON output
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show index size, file/symbol/import counts, staleness

Usage: ore.exe index-status [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Incremental refresh: reindex only changed/new files

Usage: ore.exe index-update [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe insert [OPTIONS] <FILE> <LINE> <TEXT>

Arguments:
  <FILE>  File to modify
  <LINE>  Line number to insert AT (1-indexed). Existing line N shifts down. Use 0 to insert at start, or a number greater than line count to append
  <TEXT>  Text to insert. Use \n for multiple lines

Options:
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Install missing tools via winget/choco/npm/cargo/scoop

Usage: ore.exe install-if-missing [OPTIONS] <TOOLS>

Arguments:
  <TOOLS>  Tool name (or comma-separated list)

Options:
  -s, --via <VIA>  Install source: winget | choco | npm | cargo | scoop [default: winget]
  -y, --yes        
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: format (pretty/compact/sort-keys)

Usage: ore.exe json-fmt [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -c, --compact          Compact (single line). Default: pretty
  -s, --sort-keys        Sort keys alphabetically
      --no-backup        
  -l, --label <LABEL>    
  -o, --output <OUTPUT>  Write to a different file
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: get value by dot/bracket path

Usage: ore.exe json-get [OPTIONS] <FILE> <PATH>

Arguments:
  <FILE>  
  <PATH>  Path like "foo.bar[0].baz" or "foo/bar/0/baz"

Options:
  -p, --pretty  Pretty print objects/arrays
  -j, --json    Print as raw JSON always (even for scalars)
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: list keys (flat or recursive with types)

Usage: ore.exe json-keys [OPTIONS] <FILE> [PATH]

Arguments:
  <FILE>  
  [PATH]  Path to the object whose keys to list (default: root) [default: ""]

Options:
  -t, --types      Include type of each value
  -r, --recursive  Recursive: dump full key tree with dot-notation
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: deep-merge multiple files into base

Usage: ore.exe json-merge [OPTIONS] <BASE> [OVERLAYS]...

Arguments:
  <BASE>         Base file (also the output unless -o)
  [OVERLAYS]...  One or more files to merge INTO base (later wins for scalars)

Options:
      --replace-arrays   Replace arrays instead of concatenating
  -p, --pretty           
  -o, --output <OUTPUT>  Write output to this file instead of overwriting base
      --no-backup        
  -l, --label <LABEL>    
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: JSONPath query (with $.foo.bar[?(@.x>1)] syntax)

Usage: ore.exe json-query [OPTIONS] <FILE> <PATH>

Arguments:
  <FILE>  
  <PATH>  JSONPath expression, e.g. "$.foo.bar[?(@.age > 30)].name"

Options:
  -p, --pretty  
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
JSON: set value by path (creates intermediate objects)

Usage: ore.exe json-set [OPTIONS] <FILE> <PATH> <VALUE>

Arguments:
  <FILE>   
  <PATH>   
  <VALUE>  Value (JSON literal like 42 / true / "str" / [1,2], or bare string)

Options:
  -p, --pretty         
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe line [OPTIONS] <FILE> <RANGE>

Arguments:
  <FILE>   File to read
  <RANGE>  Line number or range (e.g. "42" or "10:20" or "10-20")

Options:
  -N, --no-number          Suppress line numbers
  -C, --context <CONTEXT>  Include N lines of context before/after [default: 0]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Mark file(s) as locked (registry-only, use with rm/mv guards later)

Usage: ore.exe lock [FILES]...

Arguments:
  [FILES]...  

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List all locked files

Usage: ore.exe locks

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Macro manager (save/run/list ΓÇö sequence of commands)

Usage: ore.exe macro <COMMAND>

Commands:
  save    Save a macro (sequence of commands, one per line) from stdin or --file
  run     Run a macro (executes each command sequentially)
  list    List saved macros
  show    Show a macro's content
  rm      Delete a macro
  path    Show macro file path
  export  Export a macro to a file
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Identify file type by magic bytes

Usage: ore.exe magic [OPTIONS] [FILES]...

Arguments:
  [FILES]...  File(s) to identify

Options:
  -q, --quiet  Quiet (just type name)
  -h, --help   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Codebase map: per-file lines/size/exports/imports overview

Usage: ore.exe map [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to map [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -H, --hidden             
      --no-ignore          
  -s, --sort <SORT>        Sort by: name, lines, size, exports, imports [default: name]
  -r, --reverse            Reverse sort
  -n, --top <TOP>          Top N files only [default: 0]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Merge multiple files into one (dedup imports, headers per file)

Usage: ore.exe merge-files [OPTIONS] --output <OUTPUT> <FILES>...

Arguments:
  <FILES>...  Files to merge (in order)

Options:
  -o, --output <OUTPUT>  Output file
  -H, --headers          Include a header comment before each file's content
  -d, --dedup-imports    Deduplicate identical import lines at the top of each file
  -s, --skip-empty       Skip empty files
      --force            
      --dry-run          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create directory (recursive)

Usage: ore.exe mkdir [PATHS]...

Arguments:
  [PATHS]...  Directory paths to create

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create a file with optional initial content

Usage: ore.exe mkfile [OPTIONS] <FILE>

Arguments:
  <FILE>  File to create

Options:
  -c, --content <CONTENT>  Initial content (use \n for newlines). If omitted, file is empty
  -p, --parents            Create parent dirs
      --force              Overwrite if exists
  -y, --yes                Bypass confirmation
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create a file from clipboard, stdin, or another file

Usage: ore.exe mkfile-from [OPTIONS] <FILE>

Arguments:
  <FILE>  Destination file to create or overwrite

Options:
      --clipboard      Use clipboard content as the source
      --stdin          Read content from stdin
      --file <SOURCE>  Copy content from an existing file
  -f, --force          Overwrite without prompting if file already exists
      --no-backup      Skip creating a backup when overwriting
  -l, --label <LABEL>  Backup label when overwriting (default: timestamp)
      --strip-bom      Strip UTF-8 BOM from source content
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Long-running monitor of a command with alerts on change/error/text

Usage: ore.exe monitor [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command whose output/exit is monitored

Options:
  -i, --interval <INTERVAL>        Interval in seconds [default: 30]
  -n, --count <COUNT>              Max iterations (0 = forever) [default: 0]
      --on-change <ON_CHANGE>      Alert command to run when the state changes
      --on-error <ON_ERROR>        Alert command to run when the command exits non-zero
      --on-contains <ON_CONTAINS>  Alert when output contains this text
      --on-missing <ON_MISSING>    Alert when output STOPS containing this text
  -v, --verbose                    Show every poll (default: only on change or alert)
  -h, --help                       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Move a file and update every importer's path

Usage: ore.exe move-with-imports [OPTIONS] <SRC> <DST>

Arguments:
  <SRC>  Source file
  <DST>  Destination path (file or dir)

Options:
  -r, --root <ROOT>        Root path to scan for importers [default: .]
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
      --no-backup          
  -l, --label <LABEL>      
      --dry-run            
  -y, --yes                
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Move/rename a file or directory (auto-backup on overwrite)

Usage: ore.exe mv [OPTIONS] <SRC> <DST>

Arguments:
  <SRC>  Source file or directory
  <DST>  Destination path

Options:
  -y, --yes            Bypass confirmation for overwrites
      --force          Force overwrite even if target exists (no backup)
      --no-backup      Skip backing up target on overwrite
  -l, --label <LABEL>  Backup label
      --dry-run        Dry run
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Recursive dependency neighborhood around a file (with optional pack)

Usage: ore.exe neighbors [OPTIONS] <FILE> [PATH]

Arguments:
  <FILE>  
  [PATH]  [default: .]

Options:
  -d, --depth <DEPTH>      Max recursion depth [default: 2]
  -u, --upstream           Include upstream (files that import this)
  -D, --downstream         Include downstream (files this imports) ΓÇö default true
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -p, --pack               Pack neighbors into a bundle (like ore pack)
  -o, --output <OUTPUT>    Output file for pack mode
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe newlines [OPTIONS] <FILE>

Arguments:
  <FILE>  File to inspect or convert

Options:
  -t, --to <TO>        Target newline style (omit for check-only) [possible values: lf, crlf, cr]
      --no-backup      Skip backup
  -l, --label <LABEL>  Backup label
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Persistent project notes ΓÇö key-value memory across sessions

Usage: ore.exe notes [OPTIONS] <COMMAND>

Commands:
  set     Set a note: ore notes set "key" "value"
  get     Get a note by key
  rm      Remove a note
  list    List all notes
  clear   Remove all notes
  search  Search notes by key or value substring
  help    Print this message or the help of the given subcommand(s)

Options:
      --dir <DIR>  Working directory (default: current dir)
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Send an OS notification

Usage: ore.exe notify [OPTIONS] <MESSAGE>

Arguments:
  <MESSAGE>  

Options:
  -t, --title <TITLE>  [default: ore]
  -e, --echo           
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run a fallback command if the first fails

Usage: ore.exe on-error [OPTIONS] --then <THEN> <COMMAND>

Arguments:
  <COMMAND>  First command

Options:
      --then <THEN>  Command to run if first fails
  -s, --stream       Stream output
  -q, --silent       Silent per-step
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run a follow-up command if the first succeeds

Usage: ore.exe on-success [OPTIONS] --then <THEN> <COMMAND>

Arguments:
  <COMMAND>  

Options:
      --then <THEN>  Command to run if first succeeds
  -s, --stream       
  -q, --silent       
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Open a file or folder in default OS handler or a specified editor

Usage: ore.exe open-file [OPTIONS] <PATH>

Arguments:
  <PATH>  File or directory to open

Options:
  -e, --editor <EDITOR>  Editor / handler to use (default: OS default)
  -F, --folder           Open the containing folder in Explorer instead of the file
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Analyze and optionally reorganize top-level files into folders

Usage: ore.exe organize [OPTIONS] [PATH]

Arguments:
  [PATH]  Root directory (files at top level get grouped, subdirs are analyzed) [default: .]

Options:
  -b, --by <BY>            Grouping: type | feature (default type) [default: type]
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
      --apply              Actually perform the moves (default: plan only)
      --no-backup          
  -l, --label <LABEL>      
  -y, --yes                
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Outline one file's structure

Usage: ore.exe outline [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -E, --exported-only  
  -j, --json           
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Pack files into an AI-ready blob (md/xml/tag/plain, with tree, strip, truncate)

Usage: ore.exe pack [OPTIONS] [INPUTS]...

Arguments:
  [INPUTS]...  Glob patterns or directories to include

Options:
  -e, --ext <EXT>
          Extensions to include
  -x, --exclude <EXCLUDE>
          Excludes
  -f, --format <FORMAT>
          Output format [default: md] [possible values: md, xml, tag, plain]
  -o, --output <OUTPUT>
          Write to file instead of stdout
      --copy
          Also copy to clipboard (Windows only for now)
      --max-lines-per-file <MAX_LINES_PER_FILE>
          Truncate each file to N lines (0 = no limit) [default: 0]
      --strip-blanks
          Skip blank lines (saves tokens)
      --strip-comments
          Strip // and # comment lines (saves tokens)
      --include-tree
          Prepend the directory tree
  -H, --hidden
          Include hidden
      --no-ignore
          Ignore .gitignore
      --binary
          Include binary files
  -N, --no-numbers
          Suppress line numbers
  -h, --help
          Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Pack all files changed since a git ref (default: HEAD)

Usage: ore.exe pack-changed [OPTIONS] [SINCE]

Arguments:
  [SINCE]  Git ref to compare against (default: HEAD) [default: HEAD]

Options:
      --format <FORMAT>  Output format: tag (default), md, plain [default: tag]
  -n, --numbers          Show line numbers
      --dir <DIR>        Working directory
      --untracked        Include untracked files
  -e, --ext <EXT>        Only include files matching this extension (e.g. ts, rs)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Pack specific line ranges from multiple files: file:N-M file2:A-B

Usage: ore.exe pack-lines [OPTIONS] <SPECS>...

Arguments:
  <SPECS>...  File specs: path, path:N, path:N-M, or path:N:M Examples: src/foo.ts:80-120  src/bar.ts:1-50  src/baz.ts:200

Options:
      --format <FORMAT>  Output format: tag (default), md, plain [default: tag]
  -n, --numbers          Show line numbers in output
      --label            Show file+range label above each block (always on for tag/md)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run multiple commands in parallel

Usage: ore.exe parallel [OPTIONS] [COMMANDS]...

Arguments:
  [COMMANDS]...  Commands to run in parallel (each argument is one command)

Options:
  -l, --limit <LIMIT>  Max concurrent jobs (default: unlimited)
  -s, --stream         Stream output live (interleaved)
  -q, --silent         Suppress per-job output
      --fail-fast      Stop all if any fails
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe patch [OPTIONS] [FILE]

Arguments:
  [FILE]  File to patch (required unless --patch-file or --stdin)

Options:
  -f, --find <FIND>              Text to find
  -r, --replace <REPLACE>        Text to replace with
      --patch-file <PATCH_FILE>  Load patches from a .orepatch file
      --stdin                    Read patch spec from stdin
  -a, --all                      Replace all occurrences (default: fail if not exactly 1 match)
  -n, --nth <NTH>                Replace only the Nth occurrence (1-indexed)
      --first                    Replace only the first occurrence
      --last                     Replace only the last occurrence
      --no-backup                Skip creating a backup
  -l, --label <LABEL>            Backup label (default: timestamp)
      --dry-run                  Dry run: show what would change, don't write
      --literal                  Literal mode: do not unescape \n \t \\ in find/replace (use for paths/verbatim strings)
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Apply a .orepatch file with --atomic (all-or-nothing) and --report (pre-flight) support

Usage: ore.exe patch-batch [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>  Path to .orepatch file, or - to read from stdin

Options:
      --atomic         All-or-nothing: if any find fails, no files are written
      --report         Pre-flight report: show which hunks apply/fail without writing
      --mode <MODE>    Patch mode for all operations: once, all, first, last [default: once]
      --no-backup      Skip backups for all operations
  -l, --label <LABEL>  Backup label for all operations (default: timestamp)
      --stop-on-fail   Stop on first failure (default: attempt all, report failures)
      --literal        Literal mode: do not unescape \n \t \\ in find/replace strings from the patch file
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Insert text before or after a specific line number

Usage: ore.exe patch-insert [OPTIONS] <FILE> <LINE> <TEXT>

Arguments:
  <FILE>  File to modify
  <LINE>  Line number to insert relative to (1-indexed; 0 = prepend to file)
  <TEXT>  Text to insert. Use \n for multi-line content

Options:
      --before         Insert before the specified line (default: after)
      --after          Insert after the specified line (this is the default)
      --no-backup      Skip creating a backup
  -l, --label <LABEL>  Backup label (default: timestamp)
      --dry-run        Dry run: show what would change, don't write
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Replace an exact line or inclusive line range with new text

Usage: ore.exe patch-lines [OPTIONS] <FILE> <RANGE> <TEXT>

Arguments:
  <FILE>   File to patch
  <RANGE>  Line number or inclusive range: N, N:M, or N-M
  <TEXT>   Replacement text. Use \n for multi-line content

Options:
      --no-backup      Skip creating a backup
  -l, --label <LABEL>  Backup label (default: timestamp)
      --dry-run        Dry run: show what would change, don't write
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Preview a patch as a unified diff without writing (exits 0=found, 1=not found)

Usage: ore.exe patch-preview [OPTIONS] --find <FIND> <FILE>

Arguments:
  <FILE>  File to preview the patch on

Options:
  -f, --find <FIND>        Text to find (supports \n for multiline)
  -r, --replace <REPLACE>  Replacement text (supports \n for multiline) [default: ""]
  -a, --all                Replace all occurrences (default: exactly 1)
  -n, --nth <NTH>          Replace only the Nth occurrence (1-indexed)
      --first              Replace only the first occurrence
      --last               Replace only the last occurrence
      --no-color           Disable color output (for piping)
  -C, --context <CONTEXT>  Number of context lines in the diff (default: 3) [default: 3]
      --literal            Literal mode: do not unescape \n \t \\ in find/replace
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe patch-project [OPTIONS] --find <FIND> --replace <REPLACE> [PATH]

Arguments:
  [PATH]  Root path [default: .]

Options:
  -f, --find <FIND>        Literal text to find
  -r, --replace <REPLACE>  Replacement text
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -H, --hidden             
      --no-ignore          
      --binary             
  -a, --all                Replace all occurrences per file (default)
      --exact-one          Only replace files where exactly one match is found (safer)
      --dry-run            
      --no-backup          
  -l, --label <LABEL>      
      --keep-going         
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Patch a file using a regular expression with capture group support

Usage: ore.exe patch-regex [OPTIONS] --find <FIND> <FILE>

Arguments:
  <FILE>  File to patch

Options:
  -f, --find <FIND>        Regex pattern to find (Rust regex syntax; supports capture groups)
  -r, --replace <REPLACE>  Replacement string (supports $1, $2, ${name} capture group refs) [default: ""]
  -a, --all                Replace all matches (default: fail if not exactly 1)
  -n, --nth <NTH>          Replace only the Nth match (1-indexed)
      --first              Replace only the first match
      --last               Replace only the last match
  -i, --ignore-case        Case-insensitive matching (or use (?i) inline in pattern)
      --no-backup          Skip creating a backup
  -l, --label <LABEL>      Backup label (default: timestamp)
      --dry-run            Dry run: show match count, don't write
      --preview            Show unified diff preview before writing (implies --dry-run if combined)
  -C, --context <CONTEXT>  Context lines for --preview diff (default: 3) [default: 3]
      --literal            Literal mode: do not unescape \n \t \\ in replacement string
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TCP ping (host:port reachability test)

Usage: ore.exe ping [OPTIONS] <HOST>

Arguments:
  <HOST>  

Options:
  -p, --port <PORT>          [default: 80]
  -n, --count <COUNT>        [default: 4]
  -t, --timeout <TIMEOUT>    [default: 2]
  -i, --interval <INTERVAL>  [default: 1.0]
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Pluck exports/imports/types/hooks/components from a file

Usage: ore.exe pluck [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --exports     
      --imports     
      --types       
      --interfaces  
      --signatures  
      --hooks       
      --components  
  -N, --number      Include line numbers
  -h, --help        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
HTTP POST/PUT/PATCH/DELETE with body from string/file/json

Usage: ore.exe post [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -d, --data <DATA>        
      --file <FILE>        
  -j, --json <JSON>        
  -F, --form <FORM>        Form field, repeatable: --form key=value
  -X, --method <METHOD>    [default: POST]
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 60]
      --no-redirect        
      --proxy <PROXY>      
  -i, --include-headers    
  -q, --no-body            
      --pretty             
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe purge-backups [OPTIONS] [PATH]

Arguments:
  [PATH]  Root path (default: current dir) [default: .]

Options:
      --label <LABEL>            Only match this label suffix (e.g. "CAMFIX" -> *.bakCAMFIX)
      --older-than <OLDER_THAN>  Only backups older than this many minutes
      --newer-than <NEWER_THAN>  Only backups newer than this many minutes (useful for session-only)
      --matching <MATCHING>      Restrict to files matching this substring
  -H, --hidden                   Include hidden dirs
      --no-ignore                Don't respect .gitignore
      --dry-run                  Dry run
  -y, --yes                      Bypass confirmation
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Mark undone operations as redone (does not replay changes)

Usage: ore.exe redo [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -n, --count <COUNT>  [default: 1]
  -y, --yes            
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find every reference to a symbol across a path

Usage: ore.exe refs [OPTIONS] <SYMBOL> [PATH]

Arguments:
  <SYMBOL>  
  [PATH]    [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -C, --context <CONTEXT>  Show N context lines around each match [default: 0]
  -l, --files-only         Files only
  -D, --include-defs       Include definitions (default: skip lines that look like definitions)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Files that "go together" with a given file (siblings + imports + git co-change)

Usage: ore.exe related [OPTIONS] <FILE> [ROOT]

Arguments:
  <FILE>  
  [ROOT]  [default: .]

Options:
  -e, --ext <EXT>  
  -n, --top <TOP>  [default: 15]
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Remove a named import or an entire import line

Usage: ore.exe remove-import [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -n, --name <NAME>    Named import to remove
  -s, --from <FROM>    Remove entire import line for this source
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe rename-bulk [OPTIONS] <PATTERN> <REPLACEMENT> [PATH]

Arguments:
  <PATTERN>      Regex pattern applied to filenames
  <REPLACEMENT>  Replacement (supports $1, $2 capture groups)
  [PATH]         Root path [default: .]

Options:
  -R, --recursive          Recurse into subdirectories
  -e, --ext <EXT>          Extensions to include
  -x, --exclude <EXCLUDE>  Excludes
  -H, --hidden             Include hidden
      --no-ignore          Don't respect gitignore
      --dry-run            Dry run
  -i, --ignore-case        Case-insensitive
      --full-path          Match against full path, not just filename
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Rename a symbol, run verify, auto-rollback on failure

Usage: ore.exe rename-safe [OPTIONS] <OLD> <NEW> [PATH]

Arguments:
  <OLD>   
  <NEW>   
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -v, --verify <VERIFY>    Verify command to run after rename (default: auto-detect tsc/cargo)
  -y, --yes                
      --dry-run            
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Rename a symbol across the codebase (word-boundary regex, all files)

Usage: ore.exe rename-symbol [OPTIONS] <OLD> <NEW> [PATH]

Arguments:
  <OLD>   Old symbol name
  <NEW>   New symbol name
  [PATH]  Path to scan [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
      --no-backup          
  -l, --label <LABEL>      
      --dry-run            
  -y, --yes                Bypass confirmation
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace [OPTIONS] <PATTERN> <REPLACEMENT> <FILE>

Arguments:
  <PATTERN>      Regex pattern to find
  <REPLACEMENT>  Replacement string (supports $1, $2 capture groups)
  <FILE>         File to modify

Options:
  -F, --literal        Treat pattern as literal string, not regex
  -i, --ignore-case    Case-insensitive
  -w, --word           Whole word match only
  -m, --multiline      Multi-line mode (^ and $ match line boundaries)
      --no-backup      Skip backup
  -l, --label <LABEL>  Backup label
      --dry-run        Dry run: show matches, don't write
  -n, --max <MAX>      Replace only the first N matches (0 = all) [default: 0]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace-dir [OPTIONS] <PATTERN> <REPLACEMENT> <DIR>

Arguments:
  <PATTERN>      Regex pattern
  <REPLACEMENT>  Replacement
  <DIR>          Directory to search within

Options:
  -e, --ext <EXT>          
  -F, --literal            
  -i, --ignore-case        
  -w, --word               
  -m, --multiline          
  -H, --hidden             
      --no-ignore          
      --binary             
      --dry-run            
      --no-backup          
  -l, --label <LABEL>      
  -x, --exclude <EXCLUDE>  
      --keep-going         
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace-ext [OPTIONS] <PATTERN> <REPLACEMENT> <EXT> [PATH]

Arguments:
  <PATTERN>      Regex pattern to find
  <REPLACEMENT>  Replacement
  <EXT>          Extensions to target (comma-separated, e.g. "ts,tsx")
  [PATH]         Root path [default: .]

Options:
  -F, --literal            
  -i, --ignore-case        
  -w, --word               
  -m, --multiline          
  -H, --hidden             
      --no-ignore          
      --binary             
      --dry-run            
      --no-backup          
  -l, --label <LABEL>      
  -x, --exclude <EXCLUDE>  
      --keep-going         
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace-line [OPTIONS] <FILE> <LINE> <TEXT>

Arguments:
  <FILE>  File to modify
  <LINE>  Line number to replace (1-indexed)
  <TEXT>  New content for that line. Use \n for multi-line replacement

Options:
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace-project [OPTIONS] <PATTERN> <REPLACEMENT> [PATH]

Arguments:
  <PATTERN>      Regex pattern to find
  <REPLACEMENT>  Replacement string (supports $1, $2 capture groups)
  [PATH]         Root path (default: current dir) [default: .]

Options:
  -e, --ext <EXT>          Extensions to include (comma-separated, e.g. "ts,tsx,rs")
  -x, --exclude <EXCLUDE>  Exclude substrings (comma-separated, e.g. "test,mock")
  -F, --literal            Treat pattern as literal
  -i, --ignore-case        Case-insensitive
  -w, --word               Whole word only
  -m, --multiline          Multi-line mode (^ and $ match line boundaries)
  -H, --hidden             Include hidden files
      --no-ignore          Do NOT respect .gitignore
      --binary             Include binary files
      --dry-run            Dry run ΓÇö show matches per file, don't write
      --no-backup          Skip backups (dangerous)
  -l, --label <LABEL>      Backup label
      --keep-going         Continue on error (don't stop on first failure)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe replace-range [OPTIONS] <FILE> <RANGE> <TEXT>

Arguments:
  <FILE>   File to modify
  <RANGE>  Range to replace: "42", "10:20", "10-20"
  <TEXT>   Replacement text. Use \n for multiple lines

Options:
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: public API surface as markdown

Usage: ore.exe report-api [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -o, --output <OUTPUT>    
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: recent git changes as markdown

Usage: ore.exe report-changes [OPTIONS]

Options:
  -s, --since <SINCE>    
  -u, --until <UNTIL>    
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: git contributors as markdown

Usage: ore.exe report-contributors [OPTIONS]

Options:
  -s, --since <SINCE>    
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: structural test coverage as markdown

Usage: ore.exe report-coverage [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -o, --output <OUTPUT>    
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: last cached compile errors as markdown

Usage: ore.exe report-errors [OPTIONS]

Options:
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: codebase health as markdown

Usage: ore.exe report-health [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -o, --output <OUTPUT>  
  -e, --ext <EXT>        
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: import graph as markdown

Usage: ore.exe report-imports [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -o, --output <OUTPUT>    
  -n, --top <TOP>          [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Report: all TODO/FIXME/HACK comments as markdown

Usage: ore.exe report-todos [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -o, --output <OUTPUT>    
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe restore [OPTIONS] <FILE>

Arguments:
  <FILE>  File to restore

Options:
  -l, --label <LABEL>  Label of the backup to restore (e.g. "CAMFIX"). If omitted, uses most recent
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Resumable download using HTTP Range

Usage: ore.exe resume-download [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -o, --output <OUTPUT>    
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 600]
      --restart            Force restart (ignore existing partial file)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Retry a command until success (with backoff)

Usage: ore.exe retry [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to run

Options:
  -n, --max <MAX>            Max attempts (default 5) [default: 5]
  -i, --interval <INTERVAL>  Wait between attempts (seconds) [default: 1.0]
  -b, --backoff <BACKOFF>    Exponential backoff multiplier (default 1.0 = constant) [default: 1.0]
  -q, --silent               Suppress per-attempt logs
  -s, --stream               Stream command output
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Revert (reverse-apply) a .patch/.diff file

Usage: ore.exe revert-patch [OPTIONS] <PATCH>

Arguments:
  <PATCH>  

Options:
  -p, --path <PATH>    
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Delete files/directories with confirmation and backup

Usage: ore.exe rm [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Files or directories to delete

Options:
  -r, --recursive      Recursive (required for directories)
  -y, --yes            Bypass confirmation
  -f, --force          Force (ignore missing, no backup)
      --no-backup      Skip backup before delete
  -l, --label <LABEL>  Backup label
      --dry-run        Dry run
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Caller/callee tree for a file (upstream + downstream)

Usage: ore.exe route [OPTIONS] <FILE> [ROOT]

Arguments:
  <FILE>  
  [ROOT]  [default: .]

Options:
  -e, --ext <EXT>      
  -d, --depth <DEPTH>  Depth to expand callers/callees [default: 2]
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run a command with capture/stream/silent options

Usage: ore.exe run [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to execute (via cmd.exe /C on Windows). Quote it

Options:
  -s, --stream                   Stream output live (default: capture and print after)
  -q, --silent                   Suppress all output
      --fail-on-error            Fail (exit non-zero) if command exits non-zero
  -v, --verbose                  Print timing/exit summary
  -o, --output <OUTPUT>          Write stdout to this file
      --err-output <ERR_OUTPUT>  Write stderr to this file
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a new project from a template

Usage: ore.exe scaffold [OPTIONS] <TEMPLATE> <NAME>

Arguments:
  <TEMPLATE>  Project template to scaffold [possible values: react-app, next-app, vite-app, electron-app, rust-cli, rust-lib, node-app, node-lib, typescript-lib, python-app, monorepo, static]
  <NAME>      Project name / directory

Options:
  -p, --parent <PARENT>  Parent directory (default: current dir) [default: .]
      --pm <PM>          Package manager for JS templates (npm | yarn | pnpm) [default: npm]
      --no-install       Skip installing dependencies
      --no-git           Skip git init
      --dry-run          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Add a feature (tailwind, zustand, prettier, eslint, etc.) to a project

Usage: ore.exe scaffold-add [OPTIONS] <FEATURE> [DIR]

Arguments:
  <FEATURE>  Feature to add [possible values: tailwind, zustand, router, prettier, eslint, vitest, jest, playwright]
  [DIR]      Project dir (default: current) [default: .]

Options:
      --pm <PM>     Package manager [default: npm]
      --no-install  
  -h, --help        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a REST API client module

Usage: ore.exe scaffold-api [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -o, --out-dir <OUT_DIR>    [default: src/lib/api]
  -u, --base-url <BASE_URL>  Base URL [default: /api]
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a React component

Usage: ore.exe scaffold-component [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -o, --out-dir <OUT_DIR>  [default: src/components]
      --with-css           Include CSS module
      --with-test          Include test file
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a React context + provider + hook

Usage: ore.exe scaffold-context [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -o, --out-dir <OUT_DIR>  [default: src/contexts]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a React hook

Usage: ore.exe scaffold-hook [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -o, --out-dir <OUT_DIR>  [default: src/hooks]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a Zustand store

Usage: ore.exe scaffold-store [OPTIONS] <NAME>

Arguments:
  <NAME>  

Options:
  -o, --out-dir <OUT_DIR>  [default: src/store]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Scaffold a test file for an existing source file

Usage: ore.exe scaffold-test [OPTIONS] <FILE>

Arguments:
  <FILE>  Existing file to create a test for

Options:
  -f, --framework <FRAMEWORK>  Test framework [default: vitest]
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Windows Task Scheduler wrapper (create/list/rm/run)

Usage: ore.exe schedule <COMMAND>

Commands:
  create  Create a scheduled task (Windows Task Scheduler)
  list    List tasks matching a prefix (default: ore-)
  rm      Delete a scheduled task
  run     Run a task now
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find files containing ALL given patterns

Usage: ore.exe search-and [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to search [default: .]

Options:
  -p, --pattern <PATTERNS>  Patterns (all must be found in the file). Repeat -p N times
  -F, --literal             
  -i, --ignore-case         
  -e, --ext <EXT>           
  -x, --exclude <EXCLUDE>   
  -H, --hidden              
      --no-ignore           
  -l, --files-only          Paths only, no decoration
  -v, --verbose             Show which patterns matched per file
  -h, --help                Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Search only in git-changed files (staged/unstaged/untracked filters)

Usage: ore.exe search-changed [OPTIONS] <PATTERN>

Arguments:
  <PATTERN>  Pattern to search for

Options:
  -F, --literal            
  -i, --ignore-case        
  -w, --word               
      --staged             Only staged changes
      --unstaged           Only unstaged changes
      --untracked          Only untracked changes
  -C, --context <CONTEXT>  Show context lines [default: 0]
  -l, --files-only         Files only
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Typo-tolerant fuzzy search (filenames + content)

Usage: ore.exe search-fuzzy [OPTIONS] <QUERY> [PATH]

Arguments:
  <QUERY>  Query (case-insensitive, typo-tolerant)
  [PATH]   [default: .]

Options:
  -d, --distance <DISTANCE>    Max edit distance (default 2). Higher = more permissive [default: 2]
  -f, --filenames-only         Search filenames only (not content)
      --min-token <MIN_TOKEN>  Min token length to bother matching (default 3) [default: 3]
  -e, --ext <EXT>              
  -x, --exclude <EXCLUDE>      
  -H, --hidden                 
      --no-ignore              
  -n, --limit <LIMIT>          Max results [default: 50]
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Search across git history (pickaxe / regex)

Usage: ore.exe search-history [OPTIONS] <QUERY>

Arguments:
  <QUERY>  String to search for (finds commits where it was added or removed)

Options:
  -p, --path <PATH>    Restrict to a specific path
  -n, --limit <LIMIT>  Max commits to scan [default: 100]
  -d, --diff           Show the actual diff hunks
  -r, --regex          Regex mode instead of literal
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Search patterns that span multiple lines

Usage: ore.exe search-multiline [OPTIONS] <PATTERN> [PATH]

Arguments:
  <PATTERN>  Pattern (can span multiple lines with .* and \n)
  [PATH]     [default: .]

Options:
  -i, --ignore-case            
  -e, --ext <EXT>              
  -x, --exclude <EXCLUDE>      
  -H, --hidden                 
      --no-ignore              
  -p, --print-matches          Show the actual matched text (not just filename)
      --max-lines <MAX_LINES>  Show first N lines of each match [default: 5]
  -l, --files-only             
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find files that do NOT contain a pattern (optionally require another)

Usage: ore.exe search-negative [OPTIONS] <PATTERN> [PATH]

Arguments:
  <PATTERN>  Pattern that should NOT appear
  [PATH]     [default: .]

Options:
  -F, --literal            
  -i, --ignore-case        
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -H, --hidden             
      --no-ignore          
  -r, --require <REQUIRE>  Also require this pattern to be present (find files that contain X but NOT Y)
  -l, --files-only         
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Find files containing ANY given patterns

Usage: ore.exe search-or [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -p, --pattern <PATTERNS>  Patterns (any one match counts). Repeat -p N times
  -F, --literal             
  -i, --ignore-case         
  -e, --ext <EXT>           
  -x, --exclude <EXCLUDE>   
  -H, --hidden              
      --no-ignore           
  -l, --files-only          
  -v, --verbose             
  -h, --help                Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run commands sequentially (stop or continue on fail)

Usage: ore.exe sequence [OPTIONS] [COMMANDS]...

Arguments:
  [COMMANDS]...  Commands to run sequentially

Options:
  -c, --continue-on-error       Continue on failure (default: stop on first failure)
      --rollback-on-fail <CMD>  One or more rollback commands to run if any step fails. All rollback commands run in order regardless of their own exit codes. Can be specified multiple times: --rollback-on-fail "cmd1" --rollback-on-fail "cmd2"
  -s, --stream                  Stream each command's output live (default: buffer and print after)
  -q, --silent                  Silent (no per-step logs, only errors)
  -h, --help                    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Session tracking (start/end/log/notes)

Usage: ore.exe session <COMMAND>

Commands:
  start    Start a session (all backup ops after this can be tracked)
  end      End the current session
  current  Show the currently active session
  list     List all saved sessions
  log      Show the log of events in a session (or the current one)
  note     Add a manual note to the current session
  record   Manually record a backup event (usually done automatically)
  rm       Delete a session's log
  path     Show the sessions directory
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Export session handoff document (git status, notes, history, modified files)

Usage: ore.exe session-export [OPTIONS]

Options:
  -o, --output <OUTPUT>  Output file (default: stdout)
      --dir <DIR>        Working directory (default: current dir)
      --limit <LIMIT>    Include last N history entries (default: 50) [default: 50]
      --git              Include git status summary
      --notes            Include notes
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Verify a toolchain is installed (rust/node/git/python/env)

Usage: ore.exe setup <TOOL>

Arguments:
  <TOOL>  [possible values: rust, node, git, python, env]

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Interactive ore shell (no encoding corruption, built-in pipes)

Usage: ore.exe shell [OPTIONS]

Options:
  -d, --dir <DIR>  Working directory (default: current)
      --no-banner  Don't show the banner
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show content (stdin or file) in an editor (default: notepad). Strips ANSI colors

Usage: ore.exe show [OPTIONS] [FILE]

Arguments:
  [FILE]  File to show (omit to read from stdin)

Options:
  -e, --editor <EDITOR>  Editor to open with (default: notepad) [default: notepad]
  -p, --prefix <PREFIX>  Custom filename prefix for the temp file [default: ore]
  -x, --ext <EXT>        File extension for temp file [default: txt]
  -d, --detached         Detach (don't wait)
  -P, --print-path       Print temp file path
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Everything that changed since <date or ref>

Usage: ore.exe since [OPTIONS] <WHEN>

Arguments:
  <WHEN>  A date, duration, or git ref. Examples: "yesterday", "3 days ago", "HEAD~10", "v1.0.0"

Options:
  -s, --stat         Include diff stats
  -p, --path <PATH>  Only these paths
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Slice content between pattern markers (start/end regex)

Usage: ore.exe slice [OPTIONS] --start <START> <FILE>

Arguments:
  <FILE>  File to slice

Options:
  -s, --start <START>    Start pattern (regex)
  -e, --end <END>        End pattern (regex). If omitted, slices from start to EOF
      --include-start    Include the start line
      --include-end      Include the end line
  -a, --all              Extract every occurrence, not just the first
  -L, --label            Print with a header for each slice
  -N, --number           Show line numbers
  -i, --ignore-case      Case-insensitive
  -o, --output <OUTPUT>  Write to file
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Snippet manager (save/load/list/copy/find/export/import)

Usage: ore.exe snip <COMMAND>

Commands:
  save    Save a snippet from stdin or --file
  load    Load a snippet to stdout
  list    List all snippets
  rm      Delete a snippet
  path    Show snippet path
  find    Search snippet contents for text
  export  Export all snippets to a directory
  import  Import snippets from a directory
  copy    Copy a snippet's contents to clipboard
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract a function/class/type by name from a file

Usage: ore.exe snippet [OPTIONS] <FILE> <SYMBOL>

Arguments:
  <FILE>    
  <SYMBOL>  

Options:
  -N, --number           Show line numbers
  -L, --label            Print a header with file:line-range
  -o, --output <OUTPUT>  Write to file
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe sort-lines [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -r, --reverse        Reverse (descending)
  -i, --ignore-case    Case-insensitive
  -n, --numeric        Numeric sort
  -u, --unique         Unique (dedupe after sorting)
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Split a multi-symbol file into per-symbol files (with optional barrel hub)

Usage: ore.exe split-file [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --output-dir <OUTPUT_DIR>  Output directory (default: same dir, "<file-stem>/")
  -k, --keep-hub                 Also keep the original file as a barrel hub re-exporting each split (so imports don't break)
  -b, --by <BY>                  What to split by: fn | class | export | all (default all-exported) [default: export]
  -e, --ext <EXT>                File extension for output files (default: same as input)
  -n, --naming <NAMING>          Naming: kebab (default) or exact [default: kebab]
  -i, --carry-imports            Include imports from source file in each output
      --no-backup                
  -l, --label <LABEL>            
      --dry-run                  
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Files nobody has touched in a long time

Usage: ore.exe stale-files [OPTIONS]

Options:
  -o, --older-than <OLDER_THAN>  "180 days ago", "1 year ago", etc [default: "180 days ago"]
  -p, --path <PATH>              
  -n, --top <TOP>                [default: 50]
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe stats [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to analyze [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -H, --hidden             
      --no-ignore          
  -n, --top <TOP>          Top N largest files by size [default: 0]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Show HTTP status code only

Usage: ore.exe status [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -t, --timeout <TIMEOUT>  [default: 10]
  -q, --quiet              
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract printable strings (ASCII + optional UTF-16)

Usage: ore.exe strings [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -n, --min <MIN>  Minimum string length [default: 4]
  -o, --offsets    Show offset before each string
  -u, --utf16      Also include UTF-16 LE strings
  -m, --max <MAX>  Max results (0 = all) [default: 0]
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe strip-blank-lines [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe surround [OPTIONS] --before <BEFORE> --after <AFTER> <FILE> <RANGE>

Arguments:
  <FILE>   File to modify
  <RANGE>  Range to surround: "10:20"

Options:
  -B, --before <BEFORE>  Text to insert BEFORE the range. Use \n for multi-line
  -A, --after <AFTER>    Text to insert AFTER the range. Use \n for multi-line
      --no-backup        
  -l, --label <LABEL>    
      --dry-run          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List all exported/named symbols across a path (regex-based, TS/JS/Rust/Python)

Usage: ore.exe symbols [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -k, --kind <KIND>        Filter by kind (fn, class, hook, comp, const, type, enum, interface, struct, trait, mod)
  -E, --exported           Only exported symbols (default: everything)
  -n, --name <NAME>        Filter by name substring
  -j, --json               JSON output
  -g, --group              Group by file
  -c, --count              Just count per kind
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Tag files with labels for session tracking (read, patched, reviewed, etc.)

Usage: ore.exe tag [OPTIONS] <COMMAND>

Commands:
  add         Add tag(s) to a file: ore tag add <file> <tags...>
  rm          Remove tag(s) from a file
  get         List all tags for a file
  files       List all files with a specific tag
  list        List all files and their tags
  clear-file  Clear all tags from a file
  clear-all   Clear all tags
  summary     Show tag summary (counts per tag)
  help        Print this message or the help of the given subcommand(s)

Options:
      --dir <DIR>  Working directory (default: current dir)
  -h, --help       Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe tail [OPTIONS] <FILE>

Arguments:
  <FILE>  File to read

Options:
  -n, --lines <LINES>  Number of lines (default 10) [default: 10]
  -N, --number         Show line numbers
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Template manager with variable interpolation ({{var}})

Usage: ore.exe template <COMMAND>

Commands:
  save  Save a template from a file or stdin
  load  Load and render a template with variables
  list  List all templates
  rm    Delete a template
  path  Show template file path
  vars  List variables required by a template
  test  Test-render a template (shows what would be produced)
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Countdown timer with optional notification and follow-up command

Usage: ore.exe timer [OPTIONS] <DURATION>

Arguments:
  <DURATION>  Duration: 30s, 5m, 1h, or plain seconds

Options:
  -m, --message <MESSAGE>  Message to show when done [default: "Timer done"]
  -n, --notify [<NOTIFY>]  Also fire notification when done (pass `-n` or `-n false` to disable) [default: true] [possible values: true, false]
  -c, --command <COMMAND>  Command to run when done
  -s, --silent             Silent (no ticks printed)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TOML: format

Usage: ore.exe toml-fmt [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --no-backup        
  -l, --label <LABEL>    
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TOML: get value by path

Usage: ore.exe toml-get [OPTIONS] <FILE> <PATH>

Arguments:
  <FILE>  
  <PATH>  

Options:
  -p, --pretty  
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TOML: set value by path

Usage: ore.exe toml-set [OPTIONS] <FILE> <PATH> <VALUE>

Arguments:
  <FILE>   
  <PATH>   
  <VALUE>  

Options:
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
TOML: convert to JSON

Usage: ore.exe toml-to-json [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --output <OUTPUT>  
  -c, --compact          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Write stdin to a temp file and print its path (chain with ore open)

Usage: ore.exe to-temp [OPTIONS]

Options:
  -x, --ext <EXT>        File extension for the temp file [default: txt]
  -p, --prefix <PREFIX>  Custom filename prefix [default: ore]
  -s, --strip            Strip ANSI color codes before writing
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Create empty file(s) or update mtime

Usage: ore.exe touch [OPTIONS] [FILES]...

Arguments:
  [FILES]...  File(s) to create or update mtime

Options:
  -p, --parents  Create parent directories if missing
  -h, --help     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Every call site of a function/method with context

Usage: ore.exe trace [OPTIONS] <NAME> [PATH]

Arguments:
  <NAME>  Function/method name to trace
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -C, --context <CONTEXT>  Context lines around each call site [default: 1]
  -D, --include-defs       Include definition lines
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe tree [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to display [default: .]

Options:
  -d, --depth <DEPTH>  Max depth
  -H, --hidden         Include hidden files
      --no-ignore      Don't respect .gitignore
  -s, --size           Show file sizes
  -e, --ext <EXT>      Filter by extension (comma-separated, e.g. "ts,rs")
  -D, --dirs-only      Show only directories
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe trim [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -t, --trailing       Trim only trailing whitespace on each line (default)
  -L, --leading        Trim only leading whitespace on each line
  -b, --both           Trim both leading and trailing
      --no-backup      
  -l, --label <LABEL>  
      --dry-run        
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Strip `export` keyword from unused exports (with backup + dry-run)

Usage: ore.exe trim-dead [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -k, --keep <KEEP>        Patterns of files to preserve (e.g. "index" "main")
      --dry-run            
  -y, --yes                
      --no-backup          
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Launch interactive TUI (file tree, preview, search, command palette, git panel)

Usage: ore.exe tui [OPTIONS]

Options:
  -p, --path <PATH>  Root path (default: focus setting, or current dir)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Undo the last N recorded operations (restores from backup)

Usage: ore.exe undo [OPTIONS] [ROOT]

Arguments:
  [ROOT]  [default: .]

Options:
  -n, --count <COUNT>  Undo the last N operations (default 1) [default: 1]
  -f, --file <FILE>    Only undo entries for this file
      --dry-run        Preview what would be undone
  -y, --yes            
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Unlock file(s)

Usage: ore.exe unlock [OPTIONS] [FILES]...

Arguments:
  [FILES]...  

Options:
  -a, --all   Unlock all
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Multipart file upload (with fields, headers)

Usage: ore.exe upload [OPTIONS] --file <FILES> <URL>

Arguments:
  <URL>  URL to upload to

Options:
  -f, --file <FILES>       File(s) as "fieldname=path"; repeat for multiple
  -F, --field <FIELDS>     Additional form fields "key=value"
  -X, --method <METHOD>    [default: POST]
  -H, --header <HEADERS>   
  -t, --timeout <TIMEOUT>  [default: 600]
      --proxy <PROXY>      
  -i, --include-headers    
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List files that import from a given file

Usage: ore.exe used-by [OPTIONS] <FILE> [PATH]

Arguments:
  <FILE>  
  [PATH]  [default: .]

Options:
  -e, --ext <EXT>          
  -x, --exclude <EXCLUDE>  
  -n, --names              Show which named import(s) each importer uses
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Run typecheck + lint + tests in sequence

Usage: ore.exe verify [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -t, --kind <KIND>  Detect project type auto (default: auto). Can force: ts, rust, node [default: auto]
      --no-test      Skip tests
      --no-lint      Skip lint
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Check that an exact text anchor exists in a file before patching (exits 0=found, 1=not found)

Usage: ore.exe verify-anchor [OPTIONS] --find <FIND> <FILE>

Arguments:
  <FILE>  File to search

Options:
  -f, --find <FIND>  Text to find (supports \n for multiline anchors)
  -q, --quiet        Quiet: no output, only exit code (0=found, 1=not found)
  -c, --count        Print match count instead of found/not-found
  -n, --line         Print first match line number only
  -i, --ignore-case  Case-insensitive matching
  -x, --regex        Treat find as a regular expression
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Verify a file against an expected checksum

Usage: ore.exe verify-checksum <FILE> <EXPECTED>

Arguments:
  <FILE>      File to verify
  <EXPECTED>  Expected hash

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Validate UTF-8 encoding of one or more files

Usage: ore.exe verify-encoding [OPTIONS] [FILES]...

Arguments:
  [FILES]...  

Options:
  -b, --strict-bom  Also flag BOM presence as warning
  -h, --help        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Verify relative imports resolve (JS/TS)

Usage: ore.exe verify-imports [OPTIONS] [FILES]...

Arguments:
  [FILES]...  

Options:
  -r, --resolve-ext  Also try resolving with common extensions (.ts, .tsx, .js, .jsx, /index.ts, etc.)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Validate one or more JSON files

Usage: ore.exe verify-json [OPTIONS] [FILES]...

Arguments:
  [FILES]...  

Options:
  -f, --format-info  Show format info (compact vs pretty, size)
  -L, --lenient      Accept JSON5-style comments and trailing commas (tsconfig-friendly)
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Basic syntax check (JSON, TOML, brace-balance for code)

Usage: ore.exe verify-syntax [FILES]...

Arguments:
  [FILES]...  

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Wait for a condition (file, port, url, command output, time)

Usage: ore.exe wait [OPTIONS]

Options:
      --file <FILE>                        Wait for file to exist
      --file-missing <FILE_MISSING>        Wait for file to be MISSING
      --file-changed <FILE_CHANGED>        Wait for file to change (mtime)
      --port <PORT>                        Wait for TCP port to be open (localhost:PORT)
      --port-closed <PORT_CLOSED>          Wait for port to be closed
      --time <TIME>                        Wait N seconds
      --command <COMMAND>                  Run this command repeatedly until it succeeds (exit 0)
      --output-contains <OUTPUT_CONTAINS>  Run this command until its stdout contains this text
      --url <URL>                          Wait for HTTP URL to return 200 (uses curl)
      --status <STATUS>                    Expected HTTP status (default 200) [default: 200]
  -i, --interval <INTERVAL>                Polling interval in seconds [default: 0.5]
  -t, --timeout <TIMEOUT>                  Timeout in seconds (0 = no timeout) [default: 0]
  -q, --silent                             Suppress polling output
  -h, --help                               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Watch a path and run a command when it changes

Usage: ore.exe watch [OPTIONS] <PATH> <COMMAND>

Arguments:
  <PATH>     Path to watch (file or directory)
  <COMMAND>  Command to run when a change is detected

Options:
  -n, --no-recursive         Non-recursive
  -d, --debounce <DEBOUNCE>  Debounce (ms) [default: 300]
  -e, --ext <EXT>            Extension filter (comma-separated)
  -s, --stream               Stream command output
      --initial              Run once at start (before any change)
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Watch multiple paths with different commands per path

Usage: ore.exe watch-multi [OPTIONS] --watch <WATCHES>...

Options:
  -w, --watch <WATCHES>...   Watch specs: repeat "-w path=command" (e.g. -w "src=cargo check" -w "tests=npm test")
  -d, --debounce <DEBOUNCE>  [default: 300]
  -e, --ext <EXT>            
  -s, --stream               
      --initial              
  -n, --no-recursive         
  -h, --help                 Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Usage: ore.exe wc [OPTIONS] [FILES]...

Arguments:
  [FILES]...  File(s) to count

Options:
  -l, --lines-only  Show only line counts
  -w, --words-only  Show only word counts
  -c, --bytes-only  Show only byte counts
  -m, --chars-only  Show only character counts
  -h, --help        Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Bulk headless render check across many URLs

Usage: ore.exe web-check [OPTIONS] [URLS]...

Arguments:
  [URLS]...  URLs (inline)

Options:
  -f, --file <FILE>        URL list file
  -F, --failures-only      Only show failures
  -t, --timeout <TIMEOUT>  [default: 15]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Click an element and inspect the resulting state

Usage: ore.exe web-click [OPTIONS] <URL> <SELECTOR>

Arguments:
  <URL>       
  <SELECTOR>  

Options:
  -d, --delay <DELAY>            Wait after click (ms) [default: 500]
      --screenshot <SCREENSHOT>  Screenshot after click (path)
  -V, --visible                  Show browser window
  -t, --timeout <TIMEOUT>        [default: 30]
  -h, --help                     Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Dump all cookies for a URL

Usage: ore.exe web-cookies [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -t, --timeout <TIMEOUT>  [default: 30]
  -j, --json               
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Evaluate JavaScript on a page and print the return value

Usage: ore.exe web-eval [OPTIONS] <URL> <EXPRESSION>

Arguments:
  <URL>         
  <EXPRESSION>  JS expression to evaluate. Return value is printed

Options:
  -t, --timeout <TIMEOUT>  [default: 30]
  -j, --json               
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Fetch a URL and strip to article text (removes nav/scripts/styles)

Usage: ore.exe web-fetch-clean [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -m, --max-chars <MAX_CHARS>  Max chars to keep (default from config)
  -o, --output <OUTPUT>        
  -h, --help                   Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract rendered HTML (optionally per-selector)

Usage: ore.exe web-html [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -s, --selector <SELECTOR>            CSS selector to extract (default: entire document)
  -t, --timeout <TIMEOUT>              [default: 30]
  -o, --output <OUTPUT>                
  -w, --wait-selector <WAIT_SELECTOR>  
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract all links from a page (with filters + same-domain)

Usage: ore.exe web-links [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -f, --filter <FILTER>    Only include links matching this substring
  -s, --same-domain        Only same-domain links
  -t, --timeout <TIMEOUT>  [default: 30]
  -o, --output <OUTPUT>    
  -j, --json               
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Open a URL in a headless browser (or --visible)

Usage: ore.exe web-open [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -V, --visible                        Show the browser window (default: headless)
  -w, --wait-selector <WAIT_SELECTOR>  Wait for this CSS selector before returning
  -t, --timeout <TIMEOUT>              [default: 30]
  -k, --keep-open <KEEP_OPEN>          Keep the browser open for N seconds after loading (useful with --visible) [default: 0]
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Render a page to PDF

Usage: ore.exe web-pdf [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -o, --output <OUTPUT>                [default: page.pdf]
  -L, --landscape                      Landscape orientation
  -b, --background                     Print backgrounds (CSS colors + images)
  -m, --margin <MARGIN>                Margin in inches (all sides) [default: 0.4]
  -t, --timeout <TIMEOUT>              [default: 60]
  -w, --wait-selector <WAIT_SELECTOR>  Wait for selector before printing
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Structured scrape: field=selector pairs, optional repeating container

Usage: ore.exe web-scrape [OPTIONS] --field <FIELDS>... <URL>

Arguments:
  <URL>  

Options:
  -f, --field <FIELDS>...              Field spec: name=selector. Repeat. e.g. -f "title=h1" -f "price=.price"
  -r, --repeat <REPEAT>                If set, treat this selector as a repeating container; extract fields relative to each match
  -a, --attr <ATTR>                    Extract this attribute (default: innerText). Applies to all fields
  -t, --timeout <TIMEOUT>              [default: 30]
  -w, --wait-selector <WAIT_SELECTOR>  
  -o, --output <OUTPUT>                
  -F, --format <FORMAT>                Output format: json (default) or csv [default: json]
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Screenshot a page (viewport / full-page / per-selector, device presets)

Usage: ore.exe web-screenshot [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -o, --output <OUTPUT>                [default: screenshot.png]
  -f, --full-page                      Full page (default: viewport only)
  -s, --selector <SELECTOR>            CSS selector: screenshot just this element
      --viewport <VIEWPORT>            Viewport: WIDTHxHEIGHT (e.g. 1920x1080)
  -d, --device <DEVICE>                Device preset (iphone-14, ipad, desktop, fhd, 4k, ...)
  -w, --wait-selector <WAIT_SELECTOR>  Wait for selector before capture
  -F, --format <FORMAT>                Format: png (default) or jpeg [default: png]
  -q, --quality <QUALITY>              JPEG quality (1..=100), ignored for PNG [default: 90]
  -t, --timeout <TIMEOUT>              [default: 30]
  -k, --delay <DELAY>                  [default: 0]
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Screenshot many URLs to a directory

Usage: ore.exe web-screenshot-many [OPTIONS] [URLS]...

Arguments:
  [URLS]...  Or inline URLs

Options:
  -f, --file <FILE>        URL list file (one URL per line, # comments allowed)
  -o, --out-dir <OUT_DIR>  [default: ./screenshots]
  -F, --full-page          
  -t, --timeout <TIMEOUT>  [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Screenshot one URL at multiple viewport widths (responsive audit)

Usage: ore.exe web-screenshot-set [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -s, --sizes <SIZES>      Comma-separated widths (heights auto-scale via viewport aspect ratio) [default: 375,768,1024,1440,1920]
  -a, --aspect <ASPECT>    Aspect ratio for viewport height (height = width / ratio). Default 16/9 [default: 1.7777]
  -o, --out-dir <OUT_DIR>  [default: ./screenshots]
  -F, --full-page          
  -t, --timeout <TIMEOUT>  [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Search the web via SearXNG with DuckDuckGo fallback

Usage: ore.exe web-search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  

Options:
  -j, --json             JSON output
  -o, --output <OUTPUT>  Write to file
  -q, --quiet            Suppress progress events (result-only)
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Configure search endpoint, fallbacks, limits

Usage: ore.exe web-search-config <COMMAND>

Commands:
  list   
  get    
  set    
  reset  Reset all search-* fields to defaults
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
List/test SearXNG instances (primary + fallbacks) with latency

Usage: ore.exe web-search-instances [OPTIONS]

Options:
  -t, --test  Show latencies (probes each instance with a 3s HEAD)
  -h, --help  Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Extract visible text (optionally from a selector)

Usage: ore.exe web-text [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -s, --selector <SELECTOR>            CSS selector to limit extraction (default: body) [default: body]
  -t, --timeout <TIMEOUT>              [default: 30]
  -o, --output <OUTPUT>                Write to file
  -w, --wait-selector <WAIT_SELECTOR>  
  -h, --help                           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Print the page title

Usage: ore.exe web-title [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -t, --timeout <TIMEOUT>  [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Type into an input (with optional --submit and --clear)

Usage: ore.exe web-type [OPTIONS] <URL> <SELECTOR> <TEXT>

Arguments:
  <URL>       
  <SELECTOR>  
  <TEXT>      

Options:
      --submit             Press Enter after typing
  -c, --clear              Clear existing value before typing
  -V, --visible            
  -t, --timeout <TIMEOUT>  [default: 30]
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Wait for a selector / text / URL substring

Usage: ore.exe web-wait [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
      --selector <SELECTOR>          Wait for CSS selector to appear
      --text <TEXT>                  Wait for text to appear anywhere on the page
      --url-contains <URL_CONTAINS>  Wait for URL to contain this substring (e.g. after a login redirect)
  -t, --timeout <TIMEOUT>            Timeout in seconds [default: 30]
  -i, --interval <INTERVAL>          Poll interval (ms) [default: 500]
  -h, --help                         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Quick ready-state check for a URL (exits 1 if not ready)

Usage: ore.exe web-ws-status [OPTIONS] <URL>

Arguments:
  <URL>  

Options:
  -t, --timeout <TIMEOUT>  [default: 30]
  -q, --quiet              
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Full workspace snapshot (health + structure + git + analysis) as markdown

Usage: ore.exe workspace-report [OPTIONS] [PATH]

Arguments:
  [PATH]  [default: .]

Options:
  -o, --output <OUTPUT>  [default: workspace-report.md]
  -e, --ext <EXT>        
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
WebSocket client (send/receive/listen)

Usage: ore.exe ws [OPTIONS] <URL>

Arguments:
  <URL>  WebSocket URL (ws:// or wss://)

Options:
  -m, --message <MESSAGE>  Message to send after connecting
  -n, --count <COUNT>      Send N messages then close
  -r, --read <READ>        Read this many messages then exit
      --listen             Read forever (Ctrl+C to stop)
  -h, --help               Print help
\\\`n
---

## \$cmd\`n
\\\	ext
XML: reformat with indentation

Usage: ore.exe xml-fmt [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -w, --width <WIDTH>    Indent width [default: 2]
      --no-backup        
  -l, --label <LABEL>    
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
XML: get element text or attribute value

Usage: ore.exe xml-get [OPTIONS] <FILE> <ELEMENT>

Arguments:
  <FILE>     
  <ELEMENT>  Element name to extract text from (all occurrences)

Options:
  -a, --attr <ATTR>  Optional attribute name ΓÇö print only this attribute value instead of text
  -h, --help         Print help
\\\`n
---

## \$cmd\`n
\\\	ext
XML: convert to JSON

Usage: ore.exe xml-to-json [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --output <OUTPUT>  
  -c, --compact          
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
Raw xxd-style hex dump

Usage: ore.exe xxd [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --offset <OFFSET>  
  -l, --length <LENGTH>  [default: 0]
  -w, --width <WIDTH>    [default: 16]
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
YAML: format

Usage: ore.exe yaml-fmt [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
      --no-backup        
  -l, --label <LABEL>    
  -o, --output <OUTPUT>  
  -h, --help             Print help
\\\`n
---

## \$cmd\`n
\\\	ext
YAML: get value by path

Usage: ore.exe yaml-get [OPTIONS] <FILE> <PATH>

Arguments:
  <FILE>  
  <PATH>  

Options:
  -p, --pretty  
  -h, --help    Print help
\\\`n
---

## \$cmd\`n
\\\	ext
YAML: set value by path

Usage: ore.exe yaml-set [OPTIONS] <FILE> <PATH> <VALUE>

Arguments:
  <FILE>   
  <PATH>   
  <VALUE>  

Options:
      --no-backup      
  -l, --label <LABEL>  
  -h, --help           Print help
\\\`n
---

## \$cmd\`n
\\\	ext
YAML: convert to JSON

Usage: ore.exe yaml-to-json [OPTIONS] <FILE>

Arguments:
  <FILE>  

Options:
  -o, --output <OUTPUT>  
  -c, --compact          
  -h, --help             Print help
\\\`n
---


