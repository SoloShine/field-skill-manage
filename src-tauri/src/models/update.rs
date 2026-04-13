use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub html_url: String,
    pub body: Option<String>,
    pub published_at: String,
    pub prerelease: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub has_update: bool,
    pub release_url: String,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub error: Option<String>,
}
