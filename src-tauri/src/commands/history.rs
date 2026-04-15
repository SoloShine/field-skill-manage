use tauri::State;

use crate::commands::config::AppState;
use crate::models::history::{OperationRecord, OperationType};
use crate::services::history_service;

#[tauri::command]
pub fn get_operation_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<OperationRecord>, String> {
    let _config = state.config.lock().map_err(|e| e.to_string())?;
    drop(_config);

    let history = history_service::load_history();
    let records = match limit {
        Some(n) => history.records.into_iter().take(n as usize).collect(),
        None => history.records,
    };
    Ok(records)
}

#[tauri::command]
pub fn rollback_operation(
    state: State<'_, AppState>,
    operation_id: String,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let global_path = config.active_global_path();
    let patterns = config.agent_project_patterns.clone();
    let active_id = config.active_agent_id.clone();
    drop(config);

    history_service::rollback_operation(
        &operation_id,
        &repos,
        &global_path,
        &patterns,
        &active_id,
    )
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    let _config = state.config.lock().map_err(|e| e.to_string())?;
    drop(_config);

    let empty = crate::models::history::OperationHistory::default();
    history_service::save_history(&empty)
}
