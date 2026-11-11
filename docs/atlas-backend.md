# Atlas Desktop native Vault boundary

The native backend extends the source corpus and command boundaries from
`GoWorld-Digital/agency-agents-app`, rather than replacing its catalog,
installation renderer or local reconciliation ledger. The source baseline was
`fdab206071bca40e56bf90c717ad612a5d154c70` during this synthesis. Its MIT
copyright remains applicable. The Atlas Vault implementation is original Rust
in `src-tauri/src/corpus/vault.rs`.

The REA investigation follows the official `morluto/rea` staged questions,
evidence and unknowns workflow. Available complete source establishes the
corpus parsing and IPC boundary. Static source inspection is not evidence of
native execution. Parent-level REA engine reports and final CI results are
recorded separately in `atlas-blueprint`.

| IPC command | Arguments | Result |
|---|---|---|
| `vault_graph` | none | `GraphPayload` version 1 |
| `vault_leads` | none | Graph nodes whose kind is `lead` |
| `vault_read` | `{ path: string }` | UTF-8 Markdown |
| `vault_search` | `{ query: string }` | `{ node, score, excerpt }[]` |
| `vault_update_lead_status` | `{ path: string, status: string }` | Updated graph node |

`ATLAS_VAULT_PATH` selects an existing Vault directory. The default is
`~/.atlas/vault`. The command surface does not accept an arbitrary filesystem
root and does not fabricate a Vault when one is missing. Use the workforce
starter explicitly for an initial Vault. All disk scans execute outside the
webview thread through `spawn_blocking`.

Graph nodes have `id`, `path`, `label`, `kind`, `tags`, `frontmatter` and an
optional agent `division`. IDs are relative slash-separated paths without the
`.md` suffix. Graph edges are `{ source, target, type: "wikilink" }`; unresolved
or ambiguous targets remain visible in `unresolved`. Exact relative paths take
precedence; basename and YAML-array aliases resolve case-insensitively only
when unique. `.md` target suffix matching is case-insensitive. Headings and
display aliases in `[[target#heading|Label]]` do not change the target. Embedded
images `![[image]]` and Markdown code examples do not create edges. A YAML
scalar tag remains one tag, matching the runtime contract.

Native search is lexical, with label and frontmatter weighting. It does not
advertise vector/semantic retrieval. Node runtime and native lexical ranking
can differ; the graph contract is shared, the ranking algorithm is not.

CRM writes modify only the `status` field on an existing note under
`packs/crm-sales-os/leads/`. Allowed values are exactly `new`, `qualified`,
`proposal`, `won`, `lost`. Unknown YAML fields, numeric values and Markdown
bodies survive the update. YAML formatting can be normalized during the
round trip. An exclusive temporary sibling is synced and renamed; the writer
checks the previous content and rejects an external editor conflict before
publication. A process-local write mutex serializes native CRM changes.

Paths reject absolute/traversal/hidden/non-Markdown inputs and symlinks.
Every existing component is canonicalized and checked below the configured
root. Indexing ignores hidden configuration and symbolic entries. Bounds are
2 MiB per note, 64 MiB total, 10,000 notes, directory depth 32, query length
4,096 bytes and 100 search results. Native indexing fails explicitly when a
bound or invalid YAML is encountered; the Node indexer can instead report
warnings for skipped files. Hostile concurrent filesystem mutation by another
local process is not a security boundary; the canonical-path checks do not
claim descriptor-relative race resistance.

Atlas replaces the source bundle ID, app name, homepage and catalog origin.
The inherited upstream signer and updater endpoint are removed. Auto-updater
registration and the background scheduler remain disabled, while explicit
update IPC requests return a configuration error. The code does not trust the
upstream release signing key. `ATLAS_UPDATER_PUBKEY` is a future build-time
configuration hook, not an enabled release channel.

GitHub OAuth also requires an Atlas-owned build-time
`ATLAS_GITHUB_OAUTH_CLIENT_ID`; the inherited OAuth client ID is not reused.
Anonymous catalog operations remain available. Native release signing and
platform packaging require separately provisioned Atlas credentials.

Rust tests cover graph aliases, ambiguity, contract serialization, path
traversal, hidden and non-Markdown reads, metadata-preserving CRM writes,
invalid statuses, phantom code edges, case/embed/tag parity, malformed YAML,
oversized notes and lexical results. Unix additionally tests symlink escape
rejection. Final `cargo check`/`cargo test` results must be read from the CI
run; source completion alone is not a passing native build.
