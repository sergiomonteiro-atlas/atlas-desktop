export type NodeKind = "agent" | "soul" | "memory" | "lead" | "project" | "note";
export interface GraphNode { id: string; path: string; label: string; kind: NodeKind; division?: string; tags: string[]; frontmatter: Record<string, unknown>; }
export interface GraphPayload { version: 1; generatedAt: string; nodes: GraphNode[]; links: { source: string; target: string; type: "wikilink" }[]; unresolved: { source: string; target: string }[]; }
export interface AtlasEvent { version: 1; id: string; runId: string; agentSlug: string; type: "agent:start" | "agent:tool_call" | "agent:critic_review" | "agent:done" | "agent:error" | "graph:updated"; timestamp: string; data: Record<string, unknown>; }
export interface IndexHealth { version: 1; ready: boolean; indexComplete: boolean; warnings: string[]; }
export const statuses = ["new", "qualified", "proposal", "won", "lost"] as const;
export const kindColors: Record<NodeKind, string> = { agent: "#e8be6c", soul: "#dba9fa", memory: "#66d4d7", lead: "#80d5a7", project: "#88a9ff", note: "#a4adbd" };
const record = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
export function indexHealthWarnings(value: unknown): string[] {
  if (!record(value) || value.version !== 1 || typeof value.ready !== "boolean" || typeof value.indexComplete !== "boolean" || !Array.isArray(value.warnings) || !value.warnings.every(warning => typeof warning === "string")) return ["Não foi possível validar o diagnóstico do índice. A cobertura do grafo não foi confirmada."];
  const warnings = value.warnings.slice(0, 100).map(warning => warning.slice(0, 2000));
  if ((!value.indexComplete || !value.ready) && !warnings.length) warnings.push("O runtime indica uma indexação incompleta. O grafo apresenta apenas os documentos disponíveis.");
  if (value.warnings.length > 100) warnings.push(`Mais ${value.warnings.length - 100} avisos disponíveis no diagnóstico do runtime.`);
  return warnings;
}
export function isGraph(value: unknown): value is GraphPayload {
  if (!record(value) || value.version !== 1 || typeof value.generatedAt !== "string" || !Number.isFinite(Date.parse(value.generatedAt)) || !Array.isArray(value.nodes) || !Array.isArray(value.links) || !Array.isArray(value.unresolved) || value.nodes.length > 50000 || value.links.length > 250000) return false;
  if (!value.nodes.every(n => record(n) && typeof n.id === "string" && typeof n.path === "string" && typeof n.label === "string" && typeof n.kind === "string" && Object.hasOwn(kindColors, n.kind) && Array.isArray(n.tags) && n.tags.every(t => typeof t === "string") && record(n.frontmatter))) return false;
  const ids = new Set(value.nodes.map(n => n.id));
  return ids.size === value.nodes.length && value.links.every(l => record(l) && l.type === "wikilink" && typeof l.source === "string" && typeof l.target === "string" && ids.has(l.source) && ids.has(l.target)) && value.unresolved.every(l => record(l) && typeof l.source === "string" && typeof l.target === "string");
}
export function isAtlasEvent(value: unknown): value is AtlasEvent { const e = value as AtlasEvent; return record(value) && e.version === 1 && typeof e.id === "string" && typeof e.runId === "string" && typeof e.agentSlug === "string" && typeof e.timestamp === "string" && Number.isFinite(Date.parse(e.timestamp)) && record(e.data) && ["agent:start", "agent:tool_call", "agent:critic_review", "agent:done", "agent:error", "graph:updated"].includes(e.type); }
export function formatValue(node: GraphNode): string { const n = Number(node.frontmatter.value); const currency = String(node.frontmatter.currency ?? "BRL"); if (!Number.isFinite(n)) return "Valor não definido"; try { return new Intl.NumberFormat("pt-BR", { style: "currency", currency }).format(n); } catch { return `${n.toLocaleString("pt-BR")} ${currency}`; } }
