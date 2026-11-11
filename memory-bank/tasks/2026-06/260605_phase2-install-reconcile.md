# 260605_phase2-install-reconcile

## Objective
Build the cross-tool install + state-tracking layer — the app's differentiator. Install agents
into AI coding tools natively (Rust, deterministic) and track what's installed where via a ledger
reconciled against disk + the corpus index (the 5 states).

## Outcome
- ✅ `cargo test` 630 passed / 0 failed · vite GREEN · app launches healthy (210 agents).
- ✅ Install works end-to-end (verified by tempdir tests: render → write file → reconcile through
  Current/Outdated/Modified/Removed; identity tools write verbatim; project tools write to project root).

## Files
- `src-tauri/src/render/mod.rs` — deterministic per-tool converters (ported `scripts/convert.sh`):
  claude-code/copilot (identity), cursor (.mdc), codex (TOML + escape), gemini-cli, qwen,
  opencode (color→hex). `dests()` per tool/scope, Copilot dual-write. 4 multi-file tools error.
- `src-tauri/src/install/mod.rs` — ledger (`installs.json`), `classify()` 5-state reconcile, tools
  detection, projects registry; commands install/update/adopt/uninstall_agent, installs_reconcile,
  installs_for_agent, tools_list, projects_list. `write_agent_files()` extracted for testability.
- `src-tauri/src/corpus/mod.rs` — exposed `entry()`, `version()`, `read_source()`, pub(crate) paths.
- `src-tauri/src/lib.rs` — `mod render; mod install;` + 8 commands registered.
- `src/lib/stores/install.svelte.ts` — install store (reconcile/tools/install/uninstall).
- `src/lib/components/PersonaDiscover.svelte` — persona-detail Install menu + installed chips + toasts.

## Patterns / decisions
- Provenance = hash-match only; never mutate agent content. Determinism in `render/` is load-bearing
  (rendered_hash must be reproducible for reconcile). See systemPatterns.md §3–4.
- Tauri auto-maps camelCase JS args → snake_case Rust params (`projectPath` → `project_path`).

## Follow-ups
- Dedicated **Library** + **Tools** nav sections (browse all installs / manage per-tool).
- `update_kind` cosmetic-vs-substantive split (store body_hash in the ledger record).
- **Foreign**-file scanning + Adopt flow (scan each tool dir for recognized-but-unledgered agents).
- Antigravity/openclaw/aider/windsurf renderers (multi-file / accumulated shapes).
