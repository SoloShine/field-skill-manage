use std::sync::Mutex;

use crate::models::config::{AgentType, AppConfig};

pub struct AppState {
    pub config: Mutex<AppConfig>,
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    *cfg = config;
    Ok(())
}

/// Get all agent types (built-in + custom) with display names
#[tauri::command]
pub fn get_agent_types() -> Vec<AgentInfo> {
    AgentType::all()
        .into_iter()
        .map(|a| {
            let id = serde_json::to_string(&a)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            AgentInfo {
                id: a.id().to_string(),
                display_name: a.display_name().to_string(),
            }
        })
        .collect()
}

/// Get all agent IDs (built-in + custom) with display names
#[tauri::command]
pub fn get_all_agents(state: tauri::State<'_, AppState>) -> Vec<AgentInfo> {
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    let mut agents = get_agent_types();
    // Append custom agents
    for custom_id in &config.custom_agent_ids {
        let display_name = config.agent_display_names.get(custom_id).cloned().unwrap_or_default();
        agents.push(AgentInfo {
            id: custom_id.clone(),
            display_name,
        });
    }
    agents
}

/// Add a custom agent
#[tauri::command]
pub fn add_custom_agent(
    state: tauri::State<'_, AppState>,
    id: String,
    display_name: String,
    global_path: String,
    project_pattern: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    // Don't allow overriding built-in ids
    for agent in AgentType::all() {
        if agent.id() == id {
            return Err(format!("Cannot use built-in agent id '{}'", id));
        }
    }
    config.add_custom_agent(id, display_name, global_path, project_pattern);
    Ok(())
}

/// Remove a custom agent
#[tauri::command]
pub fn remove_custom_agent(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.remove_custom_agent(&id);
    Ok(())
}

#[derive(serde::Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
}
