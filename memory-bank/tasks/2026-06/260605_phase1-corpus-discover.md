# 260605_phase1-corpus-discover

## Objective
Build the corpus subsystem (the maintained local copy + hash index of the agency-agents repo) and
the Agents catalog (Discover) view.

## Outcome
- ✅ `cargo test` 615 passed / 0 failed · `cargo check` + `vite build` green
- ✅ App launches into the Agents catalog and renders **210 agents / 16 categories** from a
  runtime-seeded corpus + `corpus-index.json`.

## Files (key)
- `src-tauri/src/corpus/{mod,parse}.rs` — frontmatter parse, sha256 split-hash index
  (source/frontmatter/body), `corpus_status/refresh/list/get/categories`, GitHub-tarball refresh.
- `src-tauri/src/types.rs` + `src/lib/types.ts` — all 12 contract types (+5 wire-format tests).
- `src-tauri/resources/corpus-baseline/**` — bundled 210-agent baseline (16 cats).
- `src-tauri/data/agency-categories.json` — corpus category metadata (decoupled from brew's).
- `src/lib/stores/corpus.svelte.ts`, `src/lib/components/PersonaDiscover.svelte`, nav wiring.

## Key findings / decisions
- **Catalog = 210 personas, not 251.** `strategy/` (NEXUS playbooks) + `examples/` (workflow docs)
  have no agent frontmatter → documentation, excluded from `CATEGORY_DIRS` (18→16).
- Nested agents (game-development/unity, etc.) flattened to top category during seeding so none
  are undercounted (added `real_bundled_baseline_parses_completely` test that caught this).

## Bugs fixed
- **Concurrent-seed race** in `ensure_corpus`: released the cache lock before the expensive
  seed+parse, so parallel `corpus_list`+`corpus_categories` both seeded and stomped each other's
  `<file>.tmp` (rename → ENOENT). Now holds the lock across the whole init.
- Rebrand: keychain service id, corpus/brew `categories.json` collision (decoupled).

## Follow-ups (Phase 1.5)
- Category tile icons render "?" fallback — map Lucide names. Sidebar + footer still brew. Vibrancy off.
