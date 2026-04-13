use std::path::Path;
use std::process::Command;

/// Clone or pull a remote repository to a local cache directory
pub fn sync_repo(remote_url: &str, cache_path: &str) -> Result<(), String> {
    let cache = Path::new(cache_path);

    if cache.join(".git").exists() {
        // git pull
        let output = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(cache)
            .output()
            .map_err(|e| format!("Failed to run git pull: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If pull fails, try fetch + reset
            let fetch_output = Command::new("git")
                .args(["fetch", "origin"])
                .current_dir(cache)
                .output()
                .map_err(|e| format!("Failed to run git fetch: {}", e))?;

            if !fetch_output.status.success() {
                return Err(format!("git fetch failed: {}", stderr));
            }

            let reset_output = Command::new("git")
                .args(["reset", "--hard", "origin/HEAD"])
                .current_dir(cache)
                .output()
                .map_err(|e| format!("Failed to run git reset: {}", e))?;

            if !reset_output.status.success() {
                let rs = String::from_utf8_lossy(&reset_output.stderr);
                return Err(format!("git reset failed: {}", rs));
            }
        }
    } else {
        // git clone
        std::fs::create_dir_all(cache)
            .map_err(|e| format!("Failed to create cache dir: {}", e))?;

        let parent = cache.parent().ok_or("Invalid cache path")?;
        let dir_name = cache
            .file_name()
            .ok_or("Invalid cache path")?
            .to_string_lossy()
            .to_string();

        let output = Command::new("git")
            .args(["clone", remote_url, &dir_name])
            .current_dir(parent)
            .output()
            .map_err(|e| format!("Failed to run git clone: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("git clone failed: {}", stderr));
        }
    }

    Ok(())
}

/// Get the latest commit timestamp from a git repo
pub fn get_latest_commit_time(repo_path: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%aI", // author date ISO 8601
        ])
        .current_dir(repo_path)
        .output()
        .map_err(|e| format!("Failed to run git log: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err("Failed to get commit time".to_string())
    }
}
