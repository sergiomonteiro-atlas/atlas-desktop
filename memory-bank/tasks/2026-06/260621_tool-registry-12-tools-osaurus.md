# 260621_tool-registry-12-tools-osaurus

## Objective
Make tool knowledge a single source of truth (vs. scattered across Rust match arms +
frontend maps), surface all recognized tools, wire Osaurus end-to-end, and add the
in-app Playbook + Teams/Projects detail + a Projects-aware dashboard. Then consolidate
to the upstream-owned canonical `tools.json` so the app and the `agency-agents` CLI
share one contract.

## Outcome
- ✅ `cargo test` 264/0 (byte-parity render tests intact), `svelte-check` 0, build clean.
- ✅ Merged: **PR #18** (registry arc) + **PR #19** (single `tools.json`). `main` @ `1df932c`.
- ✅ Upstream `agency-agents` (same machine, `AgentLand/`): Osaurus transformer + `tools.json`
  + `check-tools.sh` + `check-tools.yml` landed (aa-side PRs #605/#606; `check-tools.yml` staged).

## What shipped
### Tool registry = single source of truth
- `src-tauri/data/tools.json` (bundled baseline of the upstream canonical) + `registry.rs`
  load it; the frontend `toolRegistry.ts` reads the same file. **The Rust `Tool` enum is gone** —
  a tool is a `String` id; `label`/`detect`/`version`/`dests`/`scope` are registry lookups and
  `render()` dispatches on the JSON `format` key. Frontend deleted `ACCENTS`/`ICONS_SVG`/`SHORT`/
  the hardcoded `SUPPORTED_TOOLS`. **Adding a tool = editing one JSON file** (+ a Rust formatter
  only for a brand-new output format).
- **All 12→13 tools** modeled (incl. Kimi, Osaurus); Tools panel lists installable + recognized-only
  (dimmed, "not yet installable"). Real **brand logos** vendored from Lobe Icons (MIT) under
  `src/lib/assets/tools/`, tinted on the accent tile; letter fallback for the iconless.

### Osaurus (the payoff)
- New `skill-md` render format = the Anthropic Agent-Skills `SKILL.md` (frontmatter `name`+`description`
  + persona body), byte-identical to the upstream `convert_osaurus`. `slugPrefix:"agency-"` namespaces
  the skill dir + `name`. Verified live: catalog agents run as native Osaurus skills.
- The transformer was contributed UPSTREAM first (`scripts/convert.sh` `convert_osaurus` + `install.sh`),
  per the "the catalog repo owns transforms" layering; our Rust render mirrors it (parity test).

### Installability is derived, not stored
- `tools.json` carries upstream truth only. Whether THIS app can install a tool is
  `installable(tool) = tool.format ∈ IMPLEMENTED_FORMATS` ({identity, codex-toml, gemini-md, qwen-md,
  cursor-mdc, opencode-md, skill-md}) — same 8-installable / 5-recognized split, self-maintaining
  (ship a renderer → add its format → those tools light up). No catalog flag to sync.

### IA additions
- **Playbook** — in-app practices + copyable starter prompts + per-team/division examples
  (`playbook.ts`, `StarterPrompt.svelte`, `PlaybookModal.svelte`, title-bar book icon, ⌘K) + `docs/USING-AGENTS.md`.
- **Teams & Projects master/detail** driven by the system back arrow (`ui.projectsSelected`/`teamsSelected`);
  division-grouped rosters; **division overview** + deploy.
- **Dashboard** — two-ring Global-vs-Projects install **sunburst** (`InstallSunburst.svelte`) so totals
  reconcile; cross-tool coverage **merged** with catalog-by-division into one card with linked hover;
  uniform donut sizing; equal-height cards.

## Decisions (see decisions.md)
- Tool registry as single source of truth; `Tool` = string id; render dispatch on `format`.
- `tools.json` is **upstream-owned** (catalog repo, like `divisions.json`), CI-guarded by `check-tools.sh`;
  the app consumes a bundled baseline (follow-up: refresh from the clone).
- Installability derived app-side from `format`, not stored in the catalog.

## Files
- Backend: `registry.rs` (NEW), `render/mod.rs`, `install/mod.rs`, `types.rs` (Tool=String), `lib.rs`,
  `Cargo.toml` (`include_dir` added then removed), `data/tools.json` (NEW; replaced `data/tools/*.json`).
- Frontend: `data/{toolRegistry,playbook}.ts` (NEW), `components/{StarterPrompt,PlaybookModal,InstallSunburst}.svelte`
  (NEW), `util/toolBadge.ts` (slim re-export), `components/{ToolsView,Teams,Projects,AgentsWorkspace,
  InstallModal,DeployBrowser,CoverageDonuts,CatalogByDivision,AgencyDashboard,CommandPalette,TitlebarControls}.svelte`,
  `stores/{ui,install}.svelte.ts`, `types.ts` (Tool=string), `routes/+page.svelte`, `assets/tools/*.svg` (NEW).
- Docs: `docs/USING-AGENTS.md` (NEW), `README.md`.
- Upstream (`AgentLand/agency-agents`): `scripts/{convert,install}.sh`, `tools.json`, `scripts/check-tools.sh`,
  `.github/workflows/check-tools.yml`.

## Artifacts
- PRs: #18, #19 (this repo). aa: #605, #606 (+ staged `check-tools.yml`).

## Deferred
- Refresh `tools.json` from the catalog clone (vs. bundling a copy) — cleaner after aa #605 prunes stale output.
- Foreign-sweep for nested `…/<dir>/SKILL.md` skills (CLI-installed Osaurus/Antigravity not auto-detected).
- Antigravity wiring once upstream makes its skill deterministic (drops `date_added`).
- Release prep: version bump + release notes + README "Loadouts" → Teams.
