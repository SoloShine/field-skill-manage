use std::sync::Mutex;

use crate::models::config::{AgentType, AppConfig};

/// Placeholder used in exported config files to represent the user's home directory
const HOME_PLACEHOLDER: &str = "${HOME}";

/// Get the current user's home directory
fn home_dir() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

/// Normalize path separators to forward slashes
fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Replace the home directory in all path fields with ${HOME}
fn make_portable(config: &mut AppConfig) {
    let home = normalize_path(&home_dir());
    let replace_home = |s: &str| normalize_path(s).replace(&home, HOME_PLACEHOLDER);

    config.cache_path = replace_home(&config.cache_path);
    for repo in &mut config.repos {
        repo.cache_path = replace_home(&repo.cache_path);
    }
    for v in config.agent_global_paths.values_mut() {
        *v = replace_home(v);
    }
}

/// Replace ${HOME} placeholders with the current machine's home directory
fn resolve_portable(config: &mut AppConfig) {
    let home = normalize_path(&home_dir());
    let resolve = |s: &str| normalize_path(s).replace(HOME_PLACEHOLDER, &home);

    config.cache_path = resolve(&config.cache_path);
    for repo in &mut config.repos {
        repo.cache_path = resolve(&repo.cache_path);
    }
    for v in config.agent_global_paths.values_mut() {
        *v = resolve(v);
    }
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub config_path: String,
}

/// Read config from disk, or return default if file doesn't exist
pub fn load_config_from_disk(path: &str) -> AppConfig {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return AppConfig::default(),
    };
    match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Warning: Failed to parse config file {}: {}. Using defaults.", path, e);
            AppConfig::default()
        }
    }
}

/// Save config to disk
fn save_config_to_disk(path: &str, config: &AppConfig) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write config file: {}", e))
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.migrate_repos();
    config.migrate_opencode_path();
    Ok(config.clone())
}

#[tauri::command]
pub fn set_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    // Save to disk first
    save_config_to_disk(&state.config_path, &config)?;

    // Then update in-memory
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
    // Don't allow overriding built-in ids
    for agent in AgentType::all() {
        if agent.id() == id {
            return Err(format!("Cannot use built-in agent id '{}'", id));
        }
    }
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.add_custom_agent(id, display_name, global_path, project_pattern);
    drop(config);
    // Persist: re-read and save
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    save_config_to_disk(&state.config_path, &cfg)
}

/// Remove a custom agent
#[tauri::command]
pub fn remove_custom_agent(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.remove_custom_agent(&id);
    drop(config);
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    save_config_to_disk(&state.config_path, &cfg)
}

#[derive(serde::Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub display_name: String,
}

/// Export config to a file, replacing home dir with ${HOME} for portability
#[tauri::command]
pub fn export_config(state: tauri::State<'_, AppState>, file_path: String) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let mut portable = config.clone();
    drop(config);

    make_portable(&mut portable);

    let json = serde_json::to_string_pretty(&portable)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&file_path, json)
        .map_err(|e| format!("Failed to write file: {}", e))
}

/// Import config from a file, resolving ${HOME} to current machine's home dir
#[tauri::command]
pub fn import_config(state: tauri::State<'_, AppState>, file_path: String) -> Result<(), String> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    resolve_portable(&mut config);
    config.migrate_repos();
    config.migrate_opencode_path();

    // Save and apply
    save_config_to_disk(&state.config_path, &config)?;
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    *cfg = config;
    Ok(())
}
