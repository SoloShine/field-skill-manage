use crate::models::update::UpdateInfo;
use crate::services::update_service;

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    match update_service::check_github_release().await {
        Ok(release) => {
            let latest = release.tag_name.trim_start_matches('v').to_string();
            let has_update = is_newer_version(&current, &latest);
            Ok(UpdateInfo {
                current_version: current,
                latest_version: latest,
                has_update,
                release_url: release.html_url,
                release_notes: release.body,
                published_at: Some(release.published_at),
                error: None,
            })
        }
        Err(e) => {
            let error_msg = if e == "NO_RELEASES" {
                "NO_RELEASES".to_string()
            } else {
                e
            };
            Ok(UpdateInfo {
                current_version: current,
                latest_version: String::new(),
                has_update: false,
                release_url: String::new(),
                release_notes: None,
                published_at: None,
                error: Some(error_msg),
            })
        }
    }
}

#[tauri::command]
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    let parse_parts = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let cur_parts = parse_parts(current);
    let new_parts = parse_parts(latest);

    for i in 0..new_parts.len().max(cur_parts.len()) {
        let cur = cur_parts.get(i).unwrap_or(&0);
        let new = new_parts.get(i).unwrap_or(&0);
        if new > cur {
            return true;
        }
        if new < cur {
            return false;
        }
    }
    false
}
