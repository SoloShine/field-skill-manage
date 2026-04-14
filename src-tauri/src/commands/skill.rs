use tauri::State;

use crate::commands::config::AppState;
use crate::models::skill::SkillComparison;
use crate::services::skill_service;

/// Get overview of skills for multiple projects
#[tauri::command]
pub fn get_projects_overview(
    state: State<'_, AppState>,
    project_paths: Vec<String>,
) -> Result<Vec<skill_service::ProjectSkillSummary>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    skill_service::build_projects_overview(&project_paths, &repos, &active_id, &patterns)
}

/// Get skill comparison for the active agent's global directory
#[tauri::command]
pub fn get_global_skills(state: State<'_, AppState>) -> Result<Vec<SkillComparison>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let repos = config.repos.clone();
    drop(config);

    skill_service::build_skill_comparisons(&global_path, &repos)
}

/// Get skill comparison for a specific project
#[tauri::command]
pub fn get_project_skills(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<SkillComparison>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let project_skills_dir = config.active_project_dir(&project_path);
    drop(config);

    skill_service::build_skill_comparisons(&project_skills_dir, &repos)
}

#[tauri::command]
pub fn install_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
    repo_id: Option<String>,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let global_path = config.active_global_path();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    // Find the skill in the specified repo or search all repos
    let target_repos: Vec<_> = if let Some(ref rid) = repo_id {
        repos.iter().filter(|r| &r.id == rid).collect()
    } else {
        repos.iter().filter(|r| r.enabled).collect()
    };

    for repo in target_repos {
        let entries = skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            return skill_service::install_skill_to_dir(&repo.cache_path, &entry.path, &target_dir);
        }
    }

    Err(format!("Skill '{}' not found in any repository", skill_name))
}

#[tauri::command]
pub fn update_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
    repo_id: Option<String>,
) -> Result<(), String> {
    install_skill(state, skill_name, target, repo_id)
}

#[tauri::command]
pub fn batch_update(
    state: State<'_, AppState>,
    skill_names: Vec<String>,
    target: String,
    repo_id: Option<String>,
) -> Result<Vec<String>, String> {
    let mut results = Vec::new();
    for name in &skill_names {
        match install_skill(state.clone(), name.clone(), target.clone(), repo_id.clone()) {
            Ok(()) => results.push(format!("{}: OK", name)),
            Err(e) => results.push(format!("{}: FAILED - {}", name, e)),
        }
    }
    Ok(results)
}

#[tauri::command]
pub fn uninstall_skill(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    let skill_path = std::path::Path::new(&target_dir).join(&skill_name);
    if skill_path.exists() {
        std::fs::remove_dir_all(&skill_path)
            .map_err(|e| format!("Failed to remove {}: {}", skill_path.display(), e))?;
        Ok(())
    } else {
        Err(format!("Skill '{}' not found at {}", skill_name, skill_path.display()))
    }
}
