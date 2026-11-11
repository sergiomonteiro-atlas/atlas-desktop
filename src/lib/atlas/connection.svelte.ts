import { invoke, isTauri } from "@tauri-apps/api/core";
import { indexHealthWarnings, isAtlasEvent, isGraph, type AtlasEvent, type GraphNode, type GraphPayload } from "./contracts";

const endpoint = "http://127.0.0.1:18790";
class AtlasConnection {
  graph: GraphPayload | null = $state(null);
  events: AtlasEvent[] = $state([]);
  agents: Record<string, AtlasEvent> = $state({});
  connected = $state(false);
  loading = $state(false);
  error = $state("");
  indexWarnings: string[] = $state([]);
  source = $state("Nenhuma ligação");
  token = $state("");
  private socket: WebSocket | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private refs = 0;
  private failures = 0;
  private refreshing: Promise<void> | null = null;
  private seen = new Set<string>();

  start(): () => void { this.refs++; if (this.refs === 1) { void this.refresh(); this.connect(); } return () => { this.refs--; if (!this.refs) { clearTimeout(this.timer); this.socket?.close(); this.socket = null; this.connected = false; } }; }
  configure(token: string) { const candidate = token.trim(); if (candidate && !/^[A-Za-z0-9._~-]{1,512}$/.test(candidate)) { this.error = "O token contém caracteres inválidos para a ligação local."; return; } this.token = candidate; this.graph = null; this.events = []; this.agents = {}; this.seen.clear(); clearTimeout(this.timer); const old = this.socket; this.socket = null; old?.close(); this.connected = false; this.failures = 0; this.connect(); void this.refresh(); }
  private connect() { if (!this.refs || !this.token) return; const socket = new WebSocket("ws://127.0.0.1:18790/ws", ["atlas-v1", "atlas-token." + this.token]); this.socket = socket;
    socket.onopen = () => { this.connected = true; this.failures = 0; void this.refresh(); };
    socket.onmessage = event => { if (typeof event.data !== "string" || event.data.length > 262144) return; try { const value: unknown = JSON.parse(event.data); if (!isAtlasEvent(value) || this.seen.has(value.id)) return; this.seen.add(value.id); if (this.seen.size > 500) this.seen.delete(this.seen.values().next().value!); this.events = [value, ...this.events].slice(0, 60); this.agents = { ...this.agents, [value.agentSlug]: value }; if (Object.keys(this.agents).length > 500) this.agents = Object.fromEntries(Object.entries(this.agents).slice(-500)); if (value.type === "graph:updated" || value.type === "agent:done") void this.refresh(); } catch { /* malformed external message ignored */ } };
    socket.onerror = () => { this.connected = false; };
    socket.onclose = () => { if (this.socket !== socket) return; this.connected = false; this.socket = null; if (this.refs) this.timer = setTimeout(() => this.connect(), Math.min(30000, 1000 * 2 ** Math.min(this.failures++, 5))); };
  }
  async refresh(): Promise<void> { if (this.refreshing) return this.refreshing; this.refreshing = this.load(); try { await this.refreshing; } finally { this.refreshing = null; } }
  private async load() {
    this.loading = true; this.error = ""; this.indexWarnings = [];
    try {
      let result: unknown;
      if (isTauri()) { result = await invoke("vault_graph"); this.source = "Vault nativo · Rust"; }
      else {
        if (!this.token) throw new Error("Introduza o ATLAS_TOKEN do daemon para abrir o Vault neste navegador.");
        const headers = { Authorization: "Bearer " + this.token };
        const [response, health] = await Promise.all([
          fetch(endpoint + "/api/graph", { headers, signal: AbortSignal.timeout(5000) }),
          fetch(endpoint + "/api/health", { headers, signal: AbortSignal.timeout(5000) }).then(async response => {
            if (!response.ok) throw new Error(`Diagnóstico HTTP ${response.status}`);
            return indexHealthWarnings(await response.json());
          }).catch(() => ["Diagnóstico do índice indisponível. A cobertura do grafo não foi confirmada."]),
        ]);
        if (!response.ok) throw new Error(`Daemon HTTP ${response.status}`);
        result = await response.json(); this.indexWarnings = health; this.source = "Atlas Agent · loopback";
      }
      if (!isGraph(result)) throw new Error("Contrato de grafo inválido");
      this.graph = result;
    } catch (error) { this.error = String(error); }
    finally { this.loading = false; }
  }
  async read(path: string): Promise<string> { if (isTauri()) return invoke<string>("vault_read", { path }); throw new Error("A leitura integral de Markdown está disponível no Atlas Desktop nativo. O navegador apresenta os metadados do grafo."); }
  async updateStatus(node: GraphNode, status: string): Promise<void> { if (!isTauri()) throw new Error("Alterações ao CRM exigem a aplicação Atlas Desktop nativa."); await invoke("vault_update_lead_status", { path: node.path, status }); await this.refresh(); }
}
export const atlas = new AtlasConnection();
