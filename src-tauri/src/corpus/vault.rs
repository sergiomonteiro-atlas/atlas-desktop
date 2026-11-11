//! Atlas Vault: local Markdown graph, lexical search and constrained CRM writes.
//!
//! This is an independent source-level synthesis of the corpus boundary, not a
//! wrapper around gbrain or a claim of model-backed semantic search. The existing
//! catalog remains intact. No operation writes outside an existing Vault note.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::AppError;

const MAX_NOTE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_NOTES: usize = 10_000;
const MAX_DEPTH: usize = 32;
const LEAD_PREFIX: &str = "packs/crm-sales-os/leads/";
const LEAD_STATUSES: &[&str] = &["new", "qualified", "proposal", "won", "lost"];
static WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub path: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    pub tags: Vec<String>,
    pub frontmatter: Map<String, Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub link_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnresolvedLink {
    pub source: String,
    pub target: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphPayload {
    pub version: u8,
    pub generated_at: String,
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
    pub unresolved: Vec<UnresolvedLink>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHit {
    pub node: GraphNode,
    pub score: f64,
    pub excerpt: String,
}

struct Note {
    node: GraphNode,
    body: String,
    targets: Vec<String>,
}

fn invalid(message: impl Into<String>) -> AppError {
    AppError::InvalidArgument { message: message.into() }
}

pub fn configured_root() -> Result<PathBuf, AppError> {
    let root = match std::env::var_os("ATLAS_VAULT_PATH") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => dirs::home_dir()
            .ok_or_else(|| invalid("Cannot locate home directory; set ATLAS_VAULT_PATH"))?
            .join(".atlas")
            .join("vault"),
    };
    canonical_root(&root)
}

fn canonical_root(root: &Path) -> Result<PathBuf, AppError> {
    let canonical = fs::canonicalize(root).map_err(|e| AppError::Io {
        message: format!("Atlas Vault unavailable at {}: {e}", root.display()),
    })?;
    if !canonical.is_dir() { return Err(invalid("Atlas Vault root must be a directory")); }
    Ok(canonical)
}

/// Reject traversal, absolute paths, hidden configuration and symlink/reparse
/// escapes. A configured root can itself be a user-selected symlink; everything
/// below its canonical location must be an ordinary directory or Markdown file.
fn safe_note(root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    if relative.is_empty() || relative.contains('\\') || relative.contains(':') || relative.contains('\0') {
        return Err(invalid("Expected a relative Markdown path using '/'"));
    }
    let input = Path::new(relative);
    if input.extension().and_then(|x| x.to_str()) != Some("md") {
        return Err(invalid("Only .md Vault notes can be read or updated"));
    }
    let mut full = root.to_path_buf();
    for component in input.components() {
        match component {
            Component::Normal(name) if !name.to_string_lossy().starts_with('.') => full.push(name),
            _ => return Err(invalid("Absolute, hidden and traversal paths are forbidden")),
        }
        let metadata = fs::symlink_metadata(&full)?;
        if metadata.file_type().is_symlink() { return Err(invalid("Vault symlinks are not followed")); }
        let actual = fs::canonicalize(&full)?;
        if !actual.starts_with(root) { return Err(invalid("Vault path escapes the configured root")); }
    }
    if !full.is_file() { return Err(invalid("Vault note is not a regular file")); }
    Ok(full)
}

fn read_note(path: &Path) -> Result<String, AppError> {
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > MAX_NOTE_BYTES {
        return Err(invalid("Vault note exceeds the 2 MiB limit"));
    }
    // The bounded reader also handles a file growing after the metadata check.
    let mut bytes = Vec::new();
    file.take(MAX_NOTE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_NOTE_BYTES { return Err(invalid("Vault note exceeds the 2 MiB limit")); }
    String::from_utf8(bytes).map_err(|_| invalid("Vault note must be valid UTF-8"))
}

fn split_frontmatter(raw: &str) -> Result<(Map<String, Value>, String), AppError> {
    let raw = raw.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    if !raw.starts_with("---\n") { return Ok((Map::new(), raw)); }
    let remainder = &raw[4..];
    let end = remainder.find("\n---\n").or_else(|| remainder.strip_suffix("\n---").map(str::len))
        .ok_or_else(|| invalid("Unclosed YAML frontmatter"))?;
    let yaml = &remainder[..end];
    let value: serde_yaml::Value = serde_yaml::from_str(yaml)
        .map_err(|e| invalid(format!("Invalid YAML frontmatter: {e}")))?;
    let json = serde_json::to_value(value).map_err(|e| invalid(format!("Invalid frontmatter values: {e}")))?;
    let map = match json {
        Value::Object(map) => map,
        Value::Null => Map::new(),
        _ => return Err(invalid("YAML frontmatter must be a mapping")),
    };
    let body_start = (end + 5).min(remainder.len());
    Ok((map, remainder[body_start..].to_owned()))
}

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
        Some(Value::String(text)) => vec![text.to_owned()],
        _ => Vec::new(),
    }
}

/// Markdown code examples are documentation, not graph edges. Exclude fenced
/// blocks and inline code before scanning [[target#section|display label]].
fn prose(body: &str) -> String {
    let mut in_fence: Option<char> = None;
    let mut result = String::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        let fence = if trimmed.starts_with("```") { Some('`') }
            else if trimmed.starts_with("~~~") { Some('~') } else { None };
        if let Some(marker) = fence {
            if in_fence == Some(marker) { in_fence = None; }
            else if in_fence.is_none() { in_fence = Some(marker); }
            continue;
        }
        if in_fence.is_some() { continue; }
        let mut inline = false;
        for ch in line.chars() {
            if ch == '`' { inline = !inline; }
            else if !inline { result.push(ch); }
        }
        result.push('\n');
    }
    result
}

fn wiki_targets(body: &str) -> Vec<String> {
    let text = prose(body);
    let mut rest = text.as_str();
    let mut targets = BTreeSet::new();
    while let Some(start) = rest.find("[[") {
        let embedded = start > 0 && rest.as_bytes()[start - 1] == b'!';
        rest = &rest[start + 2..];
        let Some(end) = rest.find("]]") else { break };
        let target = rest[..end].split('|').next().unwrap_or("").split('#').next().unwrap_or("").trim();
        if !embedded && !target.is_empty() { targets.insert(target.to_owned()); }
        rest = &rest[end + 2..];
    }
    targets.into_iter().collect()
}

fn parse_note(relative: String, raw: &str) -> Result<Note, AppError> {
    let (frontmatter, body) = split_frontmatter(raw)?;
    let id = relative.strip_suffix(".md").unwrap_or(&relative).to_owned();
    let parts: Vec<&str> = relative.split('/').collect();
    let kind = if relative.starts_with(LEAD_PREFIX) { "lead" } else {
        match parts.first().copied() {
            Some("agents") => "agent", Some("souls") => "soul", Some("memory") => "memory",
            Some("projects") => "project", _ => "note",
        }
    };
    let label = frontmatter.get("name").or_else(|| frontmatter.get("title")).and_then(Value::as_str)
        .map(str::to_owned).or_else(|| body.lines().find_map(|l| l.strip_prefix("# ").map(str::to_owned)))
        .unwrap_or_else(|| id.rsplit('/').next().unwrap_or(&id).to_owned());
    let mut tags: BTreeSet<String> = strings(frontmatter.get("tags")).into_iter()
        .map(|tag| tag.trim_start_matches('#').to_owned()).collect();
    for token in prose(&body).split_whitespace() {
        if let Some(tag) = token.strip_prefix('#') {
            let tag = tag.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '/');
            if !tag.is_empty() && tag.chars().all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '/')) {
                tags.insert(tag.to_owned());
            }
        }
    }
    let division = if kind == "agent" { parts.get(1).map(|v| (*v).to_owned()) } else { None };
    Ok(Note { node: GraphNode { id, path: relative, label, kind: kind.to_owned(), division,
        tags: tags.into_iter().collect(), frontmatter }, targets: wiki_targets(&body), body })
}

fn collect_paths(root: &Path, current: &Path, depth: usize, result: &mut Vec<String>) -> Result<(), AppError> {
    if depth > MAX_DEPTH { return Err(invalid("Vault exceeds the maximum directory depth of 32")); }
    let mut entries = fs::read_dir(current)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry.file_name().to_string_lossy().starts_with('.') { continue; }
        let kind = entry.file_type()?;
        if kind.is_symlink() { continue; }
        let path = entry.path();
        if !fs::canonicalize(&path)?.starts_with(root) { return Err(invalid("Vault entry escapes root")); }
        if kind.is_dir() { collect_paths(root, &path, depth + 1, result)?; }
        else if kind.is_file() && path.extension().and_then(|v| v.to_str()) == Some("md") {
            if result.len() >= MAX_NOTES { return Err(invalid("Vault exceeds 10,000 Markdown notes")); }
            let rel = path.strip_prefix(root).map_err(|_| invalid("Vault path escapes root"))?;
            result.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn index(root: &Path) -> Result<Vec<Note>, AppError> {
    let mut paths = Vec::new();
    collect_paths(root, root, 0, &mut paths)?;
    let mut notes = Vec::with_capacity(paths.len());
    let mut total = 0;
    for relative in paths {
        let raw = read_note(&safe_note(root, &relative)?)?;
        total += raw.len() as u64;
        if total > MAX_TOTAL_BYTES { return Err(invalid("Vault exceeds the 64 MiB indexing limit")); }
        notes.push(parse_note(relative.clone(), &raw).map_err(|e| invalid(format!("{relative}: {e}")))?);
    }
    notes.sort_by(|a, b| a.node.id.cmp(&b.node.id));
    Ok(notes)
}

fn strip_md(value: &str) -> String {
    if value.to_lowercase().ends_with(".md") { value[..value.len() - 3].to_owned() }
    else { value.to_owned() }
}

fn normalize_alias(value: &str) -> String { strip_md(value).to_lowercase() }

pub fn graph_at(root: &Path) -> Result<GraphPayload, AppError> {
    let root = canonical_root(root)?;
    let notes = index(&root)?;
    let mut aliases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for note in &notes {
        let id = &note.node.id;
        ids.insert(id.clone());
        let mut names = match note.node.frontmatter.get("aliases") {
            Some(Value::Array(_)) => strings(note.node.frontmatter.get("aliases")),
            _ => Vec::new(),
        };
        names.push(id.rsplit('/').next().unwrap_or(id).to_owned());
        for name in names { aliases.entry(normalize_alias(&name)).or_default().insert(id.clone()); }
    }
    let mut links = BTreeSet::new();
    let mut unresolved = BTreeSet::new();
    for note in &notes {
        for target in &note.targets {
            let normalized = strip_md(target);
            let resolved = if ids.contains(&normalized) { Some(normalized) }
                else { aliases.get(&normalize_alias(target)).filter(|v| v.len() == 1).and_then(|v| v.first().cloned()) };
            match resolved {
                Some(target) => { links.insert(GraphLink { source: note.node.id.clone(), target, link_type: "wikilink".into() }); }
                None => { unresolved.insert(UnresolvedLink { source: note.node.id.clone(), target: target.clone() }); }
            }
        }
    }
    Ok(GraphPayload { version: 1, generated_at: chrono::Utc::now().to_rfc3339(),
        nodes: notes.into_iter().map(|n| n.node).collect(), links: links.into_iter().collect(),
        unresolved: unresolved.into_iter().collect() })
}

pub fn read_at(root: &Path, relative: &str) -> Result<String, AppError> {
    let root = canonical_root(root)?;
    read_note(&safe_note(&root, relative)?)
}

pub fn search_at(root: &Path, query: &str) -> Result<Vec<SearchHit>, AppError> {
    if query.len() > 4096 { return Err(invalid("Search query exceeds 4096 bytes")); }
    let terms: BTreeSet<String> = query.to_lowercase().split(|c: char| !c.is_alphanumeric())
        .filter(|v| !v.is_empty()).map(str::to_owned).collect();
    if terms.is_empty() { return Ok(Vec::new()); }
    let root = canonical_root(root)?;
    let mut hits = Vec::new();
    for note in index(&root)? {
        let label = note.node.label.to_lowercase();
        let body = note.body.to_lowercase();
        let metadata = serde_json::to_string(&note.node.frontmatter)?.to_lowercase();
        let score: f64 = terms.iter().map(|t| {
            (if label.contains(t) { 4.0 } else { 0.0 }) +
            (if body.contains(t) { 1.0 } else { 0.0 }) +
            (if metadata.contains(t) { 2.0 } else { 0.0 })
        }).sum();
        if score > 0.0 {
            let excerpt = note.body.lines().find(|line| terms.iter().any(|t| line.to_lowercase().contains(t)))
                .unwrap_or(&note.body).chars().take(240).collect();
            hits.push(SearchHit { node: note.node, score, excerpt });
        }
    }
    hits.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.node.id.cmp(&b.node.id)));
    hits.truncate(100);
    Ok(hits)
}

pub fn update_lead_at(root: &Path, relative: &str, status: &str) -> Result<GraphNode, AppError> {
    if !LEAD_STATUSES.contains(&status) { return Err(invalid(format!("Unsupported lead status: {status}"))); }
    if !relative.starts_with(LEAD_PREFIX) { return Err(invalid("Only Sales OS lead status may be changed")); }
    let _guard = WRITE_LOCK.lock().map_err(|_| invalid("Vault write lock unavailable"))?;
    let root = canonical_root(root)?;
    let path = safe_note(&root, relative)?;
    let original = read_note(&path)?;
    let mut note = parse_note(relative.to_owned(), &original)?;
    if note.node.frontmatter.is_empty() { return Err(invalid("Lead requires YAML frontmatter")); }
    note.node.frontmatter.insert("status".into(), Value::String(status.into()));
    let yaml = serde_yaml::to_string(&note.node.frontmatter).map_err(|e| invalid(e.to_string()))?;
    let output = format!("---\n{yaml}---\n{}", note.body);
    if output.len() as u64 > MAX_NOTE_BYTES { return Err(invalid("Updated lead exceeds note limit")); }
    let parent = path.parent().ok_or_else(|| invalid("Lead has no parent directory"))?;
    let temporary = parent.join(format!(".atlas-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| -> Result<(), AppError> {
        let mut file = OpenOptions::new().create_new(true).write(true).open(&temporary)?;
        file.write_all(output.as_bytes())?;
        file.sync_all()?;
        // Preserve an external editor's changes rather than overwrite stale data.
        if safe_note(&root, relative)? != path || read_note(&path)? != original {
            return Err(invalid("Lead changed during update; reload and retry"));
        }
        fs::rename(&temporary, &path)?;
        if let Ok(directory) = fs::File::open(parent) { let _ = directory.sync_all(); }
        Ok(())
    })();
    if result.is_err() { let _ = fs::remove_file(&temporary); }
    result?;
    Ok(note.node)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, path: &str, body: &str) {
        let file = root.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, body).unwrap();
    }

    #[test]
    fn resolves_wikilinks_aliases_and_leaves_ambiguity_visible() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "agents/atlashub/architect.md", "---\nname: Atlas Architect\naliases: [Architect]\ntags: [atlas]\n---\n# Architect\n[[client|Client]] [[Architect#Identity]]\n");
        write(tmp.path(), "packs/crm-sales-os/leads/client.md", "---\nstatus: qualified\nvalue: 2490\nowner_agent: architect\n---\n# Client\n[[Architect]] [[shared]]\n#sales\n");
        write(tmp.path(), "memory/shared.md", "# Shared A\n");
        write(tmp.path(), "projects/shared.md", "# Shared B\n");
        let graph = graph_at(tmp.path()).unwrap();
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.links.len(), 3);
        assert_eq!(graph.unresolved.len(), 1);
        assert_eq!(graph.unresolved[0].target, "shared");
        let lead = graph.nodes.iter().find(|n| n.kind == "lead").unwrap();
        assert_eq!(lead.frontmatter["value"], 2490);
        assert!(lead.tags.contains(&"sales".to_owned()));
        assert_eq!(serde_json::to_value(&graph).unwrap()["version"], 1);
        assert!(serde_json::to_value(&graph).unwrap().get("generatedAt").is_some());
    }

    #[test]
    fn rejects_traversal_hidden_and_non_markdown_reads() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "memory/a.md", "note");
        for path in ["../outside.md", "/etc/passwd.md", "C:/private.md", "memory/../../outside.md", ".obsidian/config.md", "memory/a.txt", "memory\\a.md"] {
            assert!(read_at(tmp.path(), path).is_err(), "must reject {path}");
        }
        assert_eq!(read_at(tmp.path(), "memory/a.md").unwrap(), "note");
    }

    #[test]
    fn lead_write_retains_unknown_metadata_and_markdown() {
        let tmp = tempfile::tempdir().unwrap();
        let path = "packs/crm-sales-os/leads/test.md";
        write(tmp.path(), path, "---\nname: Example\nstatus: new\nvalue: 20.5\ncustom: {keep: true}\n---\n# Example\n[[atlas-sales-os-commander]]\n");
        update_lead_at(tmp.path(), path, "won").unwrap();
        let updated = read_at(tmp.path(), path).unwrap();
        let note = parse_note(path.into(), &updated).unwrap();
        assert_eq!(note.node.frontmatter["status"], "won");
        assert_eq!(note.node.frontmatter["custom"]["keep"], true);
        assert_eq!(note.node.frontmatter["value"], 20.5);
        assert!(note.body.contains("[[atlas-sales-os-commander]]"));
        assert!(update_lead_at(tmp.path(), path, "invented").is_err());
        assert!(update_lead_at(tmp.path(), "projects/test.md", "won").is_err());
    }

    #[test]
    fn code_examples_do_not_create_phantom_edges() {
        assert_eq!(wiki_targets("[[real#heading|Label]]\n```md\n[[fake]]\n```\n`[[inline]]`\n[[real]]"), vec!["real"]);
    }

    #[test]
    fn graph_alias_case_embed_and_scalar_tag_match_runtime_contract() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "agents/atlashub/architect.md", "---\nname: Atlas\naliases: [Principal]\ntags: atlas team\n---\n# Architect\n");
        write(tmp.path(), "projects/genesis.md", "[[ARCHITECT.MD]] [[pRiNcIpAl]] ![[image]]\n");
        let graph = graph_at(tmp.path()).unwrap();
        assert_eq!(graph.links.len(), 1);
        assert!(graph.unresolved.is_empty());
        assert_eq!(graph.nodes[0].tags, vec!["atlas team"]);
    }

    #[test]
    fn malformed_yaml_and_oversize_notes_fail_explicitly() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "bad.md", "---\nname: [not closed\n---\nbody");
        assert!(graph_at(tmp.path()).is_err());
        fs::remove_file(tmp.path().join("bad.md")).unwrap();
        write(tmp.path(), "large.md", &"x".repeat(MAX_NOTE_BYTES as usize + 1));
        assert!(graph_at(tmp.path()).is_err());
    }

    #[test]
    fn lexical_search_returns_actual_matching_notes() {
        let tmp = tempfile::tempdir().unwrap();
        write(tmp.path(), "agents/atlashub/sales.md", "---\nname: Sales Architect\n---\nCRM revenue pipeline\n");
        write(tmp.path(), "projects/editorial.md", "# Editorial\nA book\n");
        let hits = search_at(tmp.path(), "CRM").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].node.id, "agents/atlashub/sales");
        assert!(search_at(tmp.path(), "missing").unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_outside_vault_is_neither_indexed_nor_read() {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "secret.md", "private");
        std::os::unix::fs::symlink(outside.path().join("secret.md"), tmp.path().join("escape.md")).unwrap();
        assert!(read_at(tmp.path(), "escape.md").is_err());
        assert!(graph_at(tmp.path()).unwrap().nodes.is_empty());
    }
}
