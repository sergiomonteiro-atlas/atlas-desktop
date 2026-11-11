# 260605_phase3-loadouts-dashboard

## Objective
Phase 3: portable Loadouts (Agentfile save/restore) and an agency Dashboard rollup that
replaces the inherited brew Dashboard.

## Outcome
- ✅ `cargo test` 631 passed / 0 · vite GREEN · verified live (Dashboard renders real data).
- ✅ Dashboard shows: 210 available · 0 installed-by-you · 0 need-attention · 180 found-to-adopt
  (the Foreign agents from Michael's install.sh) · catalog-by-category bars · 6 detected tools.

## Files
- `src-tauri/src/install/mod.rs` — `loadout_export(path)` (writes Agentfile JSON from the ledger),
  `loadout_import(path)` (installs each entry, skips failures). `Agentfile`/`LoadoutEntry` types,
  `agentfile_roundtrips` test.
- `src-tauri/src/lib.rs` — registered loadout_export/import.
- `src/lib/stores/install.svelte.ts` — exportLoadout/importLoadout.
- `src/lib/components/Loadouts.svelte` — Export/Restore (dialog save/open) + current-loadout list.
- `src/lib/components/AgencyDashboard.svelte` — stat tiles (deep-link to Library) + category bars +
  detected-tools strip.
- `src/routes/+page.svelte` — dashboard→AgencyDashboard, +loadouts route, ⌘0-4/6 keymap.
- `src/lib/components/Sidebar.svelte`, `types.ts`, `ui.svelte.ts` — full agency nav
  (Dashboard/Agents/Library/Tools/Loadouts/Activity).

## Notes
- Agentfile = `{ "agentfile": 1, "installs": [{slug, tool, projectPath}] }` — diffable/shareable.
- Default landing kept as Agents (personas) per Michael's "catalog is the front door" preference;
  Dashboard is ⌘0 / first nav item.

## Next
- Phase 4 — Trending (new & updated from corpus git history) + GitHub (star/watch/issues, repo stats).
