use crate::models::skill::{Checksum, SkillMeta};
use crate::services::hash_service;
use std::path::Path;

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
