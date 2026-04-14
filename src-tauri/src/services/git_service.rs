use std::path::Path;
use std::process::{Command, Stdio};

/// Check if a directory is a valid git repository
fn is_valid_git_repo(path: &Path) -> bool {
    if !path.join(".git").exists() {
        return false;
    }
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Perform a fresh git clone
fn git_clone(remote_url: &str, cache: &Path) -> Result<(), String> {
    // Remove any existing (possibly corrupted) directory
    if cache.exists() {
        std::fs::remove_dir_all(cache)
            .map_err(|e| format!("Failed to clean cache dir: {}", e))?;
    }

    // Ensure parent directory exists
    let parent = cache.parent().ok_or("Invalid cache path")?;
    std::fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create parent dir: {}", e))?;

    let dir_name = cache
        .file_name()
        .ok_or("Invalid cache path")?
        .to_string_lossy()
        .to_string();

    let output = Command::new("git")
        .args(["clone", "--depth", "1", remote_url, &dir_name])
        .current_dir(parent)
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run git clone: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Clean up partial clone
        let _ = std::fs::remove_dir_all(cache);
        return Err(format!("git clone failed: {}", stderr));
    }

    Ok(())
}

/// Clone or pull a remote repository to a local cache directory
pub fn sync_repo(remote_url: &str, cache_path: &str) -> Result<(), String> {
    let cache = Path::new(cache_path);

    if is_valid_git_repo(cache) {
        // Valid repo exists → pull
        let output = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(cache)
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run git pull: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If pull fails, try fetch + reset
            let fetch_output = Command::new("git")
                .args(["fetch", "origin"])
                .current_dir(cache)
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to run git fetch: {}", e))?;

            if !fetch_output.status.success() {
                // fetch also failed — repo may be corrupted, fall back to fresh clone
                eprintln!("Pull/fetch failed for '{}', re-cloning: {}", cache_path, stderr);
                return git_clone(remote_url, cache);
            }

            let reset_output = Command::new("git")
                .args(["reset", "--hard", "origin/HEAD"])
                .current_dir(cache)
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("Failed to run git reset: {}", e))?;

            if !reset_output.status.success() {
                let rs = String::from_utf8_lossy(&reset_output.stderr);
                return Err(format!("git reset failed: {}", rs));
            }
        }
    } else {
        // No valid repo → clone
        git_clone(remote_url, cache)?;
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
