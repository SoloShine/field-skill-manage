use serde::Serialize;
use std::path::Path;
use tauri::State;

use crate::commands::config::AppState;

#[derive(Serialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

/// List all files in a skill directory as a tree
#[tauri::command]
pub fn get_skill_file_tree(
    state: State<'_, AppState>,
    skill_name: String,
    target: String,
) -> Result<Vec<FileNode>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let cache_path = config.cache_path.clone();
    let global_path = config.active_global_path();
    let project_dir = config.active_project_dir(&target);
    drop(config);

    let target_dir = if target == "global" { global_path } else { project_dir };

    let local_path = Path::new(&target_dir).join(&skill_name);
    let remote_path = Path::new(&cache_path).join("skills").join(&skill_name);

    let skill_dir = if local_path.exists() {
        local_path
    } else if remote_path.exists() {
        remote_path
    } else {
        return Err(format!("Skill '{}' not found locally or remotely", skill_name));
    };

    build_file_tree(&skill_dir, &skill_dir)
}

fn build_file_tree(base: &Path, current: &Path) -> Result<Vec<FileNode>, String> {
    let mut nodes = Vec::new();
    let entries = std::fs::read_dir(current).map_err(|e| format!("Read dir: {}", e))?;

    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for entry in entries {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let is_dir = path.is_dir();

        let children = if is_dir {
            Some(build_file_tree(base, &path)?)
        } else {
            None
        };

        nodes.push(FileNode {
            name,
            path: relative,
            is_dir,
            children,
        });
    }

    Ok(nodes)
}

/// Read a single file's content from a skill directory
#[tauri::command]
pub fn read_skill_file(
    state: State<'_, AppState>,
    skill_name: String,
    file_path: String,
    target: String,
) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let cache_path = config.cache_path.clone();
    let global_path = config.active_global_path();
    let project_dir = config.active_project_dir(&target);
    drop(config);

    let target_dir = if target == "global" { global_path } else { project_dir };

    let local_base = Path::new(&target_dir).join(&skill_name);
    let remote_base = Path::new(&cache_path).join("skills").join(&skill_name);

    let full_path = local_base.join(&file_path);
    let full_path = if full_path.exists() {
        full_path
    } else {
        remote_base.join(&file_path)
    };

    if !full_path.exists() {
        return Err(format!("File '{}' not found", file_path));
    }

    std::fs::read_to_string(&full_path)
        .map_err(|e| format!("Read file {}: {}", full_path.display(), e))
}
