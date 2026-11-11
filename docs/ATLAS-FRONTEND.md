# Atlas Command Surface

The Atlas extension preserves the upstream agent catalog, tools, teams, projects and installers. The first screen is now `Atlas Command` and the original catalog remains available from the sidebar.

## Real data and connection

Native Desktop reads `~/.atlas/vault` (or `ATLAS_VAULT_PATH`) through `vault_graph`, `vault_read` and `vault_update_lead_status` Rust commands. Browser mode reads the authenticated loopback daemon's `/api/graph`. It requires the daemon's `ATLAS_TOKEN`, supplied in the connection field. Tokens stay in memory for the session and are never persisted or put in URLs. WebSocket authentication uses the `atlas-v1` and `atlas-token.<token>` subprotocols; the server selects `atlas-v1`.

Without a real Vault or daemon the screen reports the missing connection and setup instructions. There are no synthetic production agents, counters, events or CRM results. Browser mode shows graph metadata and explicitly requires native Desktop for Markdown reads and CRM status writes.

## Views

- Galaxy 3D renders the real graph with Three.js, OrbitControls and pointer raycasting; colors distinguish agents, souls, memories, leads and projects. A searchable keyboard accessible document list accompanies the graph. Rendering is bounded to 1,500 nodes, the list to 500 results; search narrows the data.
- Command 3D maps real agent documents to spatial workstations with capsule avatars, screens and desks. Actual start/tool/review/done/error events affect agent illumination for 30 seconds.
- Sales OS displays the five contract stages (`new`, `qualified`, `proposal`, `won`, `lost`), lead-specific currencies and Markdown status updates through confined native commands. Errors and saved changes appear immediately.
- Vault & Brain provides local graph/document filtering, node metadata and plain text Markdown reading. Markdown is shown as text and never executed as HTML.

The daemon retries WebSocket connections with exponential backoff, retains at most 60 telemetry events and bounds deduplication/state maps. View teardown closes connections, timers, GPU geometry/materials, controls and observers. Reduced-motion preferences disable avatar pulses.

## Landing and verification

The official landing route is `/atlas` and links to the real source repositories and the Command Surface. No unreleased installer is advertised as downloadable. Build with `npm ci`, `npm run check`, `npm test`, `npm run build`. Windows CI also checks and tests the complete Rust/Tauri backend. Static browser build and native installer deployment remain separate operations.
