//! Atlas Vault IPC. Disk parsing runs off the webview/UI thread.
use crate::corpus::vault::{self, GraphNode, GraphPayload, SearchHit};
use crate::error::AppError;

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> Result<T, AppError> + Send + 'static) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|e| AppError::Internal { message: e.to_string() })?
}

#[tauri::command]
pub async fn vault_graph() -> Result<GraphPayload, AppError> {
    blocking(|| vault::graph_at(&vault::configured_root()?)).await
}

#[tauri::command]
pub async fn vault_leads() -> Result<Vec<GraphNode>, AppError> {
    blocking(|| Ok(vault::graph_at(&vault::configured_root()?)?.nodes.into_iter().filter(|n| n.kind == "lead").collect())).await
}

#[tauri::command]
pub async fn vault_read(path: String) -> Result<String, AppError> {
    blocking(move || vault::read_at(&vault::configured_root()?, &path)).await
}

#[tauri::command]
pub async fn vault_search(query: String) -> Result<Vec<SearchHit>, AppError> {
    blocking(move || vault::search_at(&vault::configured_root()?, &query)).await
}

#[tauri::command]
pub async fn vault_update_lead_status(path: String, status: String) -> Result<GraphNode, AppError> {
    blocking(move || vault::update_lead_at(&vault::configured_root()?, &path, &status)).await
}
