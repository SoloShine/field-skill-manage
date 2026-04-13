use crate::models::update::GithubRelease;

const GITHUB_REPO: &str = "SoloShine/field-skill-manage";

pub async fn check_github_release() -> Result<GithubRelease, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 优先查询正式版（非 prerelease）
    let latest_url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let response = client
        .get(&latest_url)
        .header("User-Agent", "SPM-Manager")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().is_success() {
        let release: GithubRelease = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        return Ok(release);
    }

    // 正式版 404 时回退到 /releases 列表，取第一个（含 prerelease）
    let list_url = format!(
        "https://api.github.com/repos/{}/releases?per_page=1",
        GITHUB_REPO
    );

    let response = client
        .get(&list_url)
        .header("User-Agent", "SPM-Manager")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status().as_u16() == 404 {
        return Err("NO_RELEASES".to_string());
    }
    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let releases: Vec<GithubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    releases
        .into_iter()
        .next()
        .ok_or_else(|| "NO_RELEASES".to_string())
}
