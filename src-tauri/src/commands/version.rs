use serde::Serialize;
use tauri::State;

use crate::commands::config::AppState;
use crate::models::skill::{Checksum, SkillDiff, SkillMeta};
use crate::services::hash_service;
use std::path::Path;

/// Content pair for a single file in a diff
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFileContent {
    pub local_content: Option<String>,
    pub remote_content: Option<String>,
}

#[tauri::command]
pub fn calculate_skill_hash(skill_path: String) -> Result<Checksum, String> {
    let path = Path::new(&skill_path);
    hash_service::aggregate_sha256(path)
}

#[tauri::command]
pub fn compare_skill_versions(
    local: SkillMeta,
    remote: SkillMeta,
) -> Result<String, String> {
    // 1. Compare version strings
    if local.version != remote.version {
        return Ok("outdated".to_string());
    }

    // 2. Compare checksums
    match (&local.checksum, &remote.checksum) {
        (Some(lc), Some(rc)) => {
            if lc.value == rc.value {
                Ok("installed".to_string())
            } else {
                Ok("modified".to_string())
            }
        }
        _ => {
            // No checksums, version matches
            Ok("installed".to_string())
        }
    }
}

#[tauri::command]
pub fn verify_skill_integrity(
    skill_path: String,
    expected_hash: String,
) -> Result<bool, String> {
    let path = Path::new(&skill_path);
    let checksum = hash_service::aggregate_sha256(path)?;
    Ok(checksum.value == expected_hash)
}

#[tauri::command]
pub fn get_skill_diff(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
) -> Result<SkillDiff, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let repos = config.repos.clone();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    let local_dir = Path::new(&target_dir).join(&skill_name);
    if !local_dir.exists() {
        return Err(format!("Local skill '{}' not found at {}", skill_name, local_dir.display()));
    }

    // Find remote skill directory
    for repo in &repos {
        if !repo.enabled {
            continue;
        }
        let entries = crate::services::skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            let remote_dir = Path::new(&repo.cache_path).join(&entry.path);
            if remote_dir.exists() {
                return crate::services::skill_service::build_skill_diff(&local_dir, &remote_dir);
            }
        }
    }

    Err(format!("Remote skill '{}' not found in any repository", skill_name))
}

/// Read both local and remote content of a single file for content diff
#[tauri::command]
pub fn get_diff_file_content(
    state: State<'_, AppState>,
    skill_name: String,
    file_path: String,
    target: String,
) -> Result<DiffFileContent, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let global_path = config.active_global_path();
    let repos = config.repos.clone();
    let target_dir = if target == "global" {
        global_path
    } else {
        config.active_project_dir(&target)
    };
    drop(config);

    let local_dir = Path::new(&target_dir).join(&skill_name);
    let local_content = if local_dir.exists() {
        let full_path = local_dir.join(&file_path);
        if full_path.exists() {
            Some(std::fs::read_to_string(&full_path).map_err(|e| format!("Read local file: {}", e))?)
        } else {
            None
        }
    } else {
        None
    };

    let mut remote_content: Option<String> = None;
    for repo in &repos {
        if !repo.enabled {
            continue;
        }
        let entries = crate::services::skill_service::load_skill_entries(&repo.cache_path);
        if let Some(entry) = entries.iter().find(|s| s.name == skill_name) {
            let remote_dir = Path::new(&repo.cache_path).join(&entry.path);
            let full_path = remote_dir.join(&file_path);
            if full_path.exists() {
                remote_content = Some(std::fs::read_to_string(&full_path).map_err(|e| format!("Read remote file: {}", e))?);
            }
            break;
        }
    }

    Ok(DiffFileContent {
        local_content,
        remote_content,
    })
}
