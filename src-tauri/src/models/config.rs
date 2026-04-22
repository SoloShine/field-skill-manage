use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a single remote Git repository
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepoConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub cache_path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Supported built-in agent types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum AgentType {
    Claude,
    OpenCode,
    Codex,
    Cursor,
    Windsurf,
}

impl AgentType {
    pub fn id(&self) -> &str {
        match self {
            AgentType::Claude => "claude",
            AgentType::OpenCode => "opencode",
            AgentType::Codex => "codex",
            AgentType::Cursor => "cursor",
            AgentType::Windsurf => "windsurf",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            AgentType::Claude => "Claude Code",
            AgentType::OpenCode => "OpenCode",
            AgentType::Codex => "Codex (OpenAI)",
            AgentType::Cursor => "Cursor",
            AgentType::Windsurf => "Windsurf / Cline",
        }
    }

    /// Default global skill directory
    pub fn default_global_dir(&self, home: &str) -> String {
        match self {
            AgentType::Claude => format!("{}/.claude/skills", home),
            AgentType::OpenCode => format!("{}/.config/opencode/skills", home),
            AgentType::Codex => format!("{}/.codex/skills", home),
            AgentType::Cursor => format!("{}/.cursor/skills", home),
            AgentType::Windsurf => format!("{}/.windsurf/skills", home),
        }
    }

    /// Default project-level skill directory pattern ({project} placeholder)
    pub fn default_project_pattern(&self) -> &'static str {
        match self {
            AgentType::Claude => "{project}/.claude/skills",
            AgentType::OpenCode => "{project}/.opencode/skills",
            AgentType::Codex => "{project}/.codex/skills",
            AgentType::Cursor => "{project}/.cursor/skills",
            AgentType::Windsurf => "{project}/.windsurf/skills",
        }
    }

    /// All built-in agent types
    pub fn all() -> Vec<AgentType> {
        vec![
            AgentType::Claude,
            AgentType::OpenCode,
            AgentType::Codex,
            AgentType::Cursor,
            AgentType::Windsurf,
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub remote_url: String,
    #[serde(default)]
    pub cache_path: String,
    pub auto_sync: bool,
    /// Active agent ID (built-in id like "claude" or custom id)
    pub active_agent_id: String,
    /// Global skill paths keyed by agent id (built-in + custom)
    pub agent_global_paths: HashMap<String, String>,
    /// Project dir patterns keyed by agent id, "{project}" is replaced at runtime
    pub agent_project_patterns: HashMap<String, String>,
    /// Display names keyed by agent id (for custom agents)
    pub agent_display_names: HashMap<String, String>,
    /// Ordered list of custom agent ids
    pub custom_agent_ids: Vec<String>,
    /// Multiple remote repositories
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs_home();
        let agents = AgentType::all();
        let mut agent_global_paths = HashMap::new();
        let mut agent_project_patterns = HashMap::new();

        for agent in &agents {
            agent_global_paths.insert(
                agent.id().to_string(),
                agent.default_global_dir(&home),
            );
            agent_project_patterns.insert(
                agent.id().to_string(),
                agent.default_project_pattern().to_string(),
            );
        }

        let default_cache = format!("{}/.spm/cache", home);
        let default_repo_cache = format!("{}/.spm/cache/default", home);
        Self {
            remote_url: "https://github.com/anthropics/skills.git".to_string(),
            cache_path: default_cache,
            auto_sync: false,
            active_agent_id: "claude".to_string(),
            agent_global_paths,
            agent_project_patterns,
            agent_display_names: HashMap::new(),
            custom_agent_ids: Vec::new(),
            repos: vec![RepoConfig {
                id: "default".to_string(),
                name: "默认仓库".to_string(),
                url: "https://github.com/anthropics/skills.git".to_string(),
                cache_path: default_repo_cache,
                enabled: true,
            }],
        }
    }
}

impl AppConfig {
    /// Migrate from legacy single-repo config (remote_url/cache_path) to multi-repo (repos)
    pub fn migrate_repos(&mut self) {
        if self.repos.is_empty() && !self.remote_url.is_empty() {
            self.repos.push(RepoConfig {
                id: "default".to_string(),
                name: "默认仓库".to_string(),
                url: self.remote_url.clone(),
                cache_path: self.cache_path.clone(),
                enabled: true,
            });
        }
    }

    /// Migrate OpenCode global path from old default (~/.opencode/skills)
    /// to correct path (~/.config/opencode/skills)
    pub fn migrate_opencode_path(&mut self) {
        let home = dirs_home();
        let old_path = format!("{}/.opencode/skills", home);
        let new_path = AgentType::OpenCode.default_global_dir(&home);
        if let Some(path) = self.agent_global_paths.get_mut("opencode") {
            if *path == old_path {
                *path = new_path;
            }
        }
    }

    /// Resolve the global skill path for the active agent
    pub fn active_global_path(&self) -> String {
        self.agent_global_paths
            .get(&self.active_agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Resolve the project skill dir for the active agent by substituting {project}
    pub fn active_project_dir(&self, project_path: &str) -> String {
        let pattern = self
            .agent_project_patterns
            .get(&self.active_agent_id)
            .cloned()
            .unwrap_or_else(|| "{project}/.claude/skills".to_string());
        pattern.replace("{project}", project_path)
    }

    /// Get display name for an agent id
    pub fn agent_display_name(&self, id: &str) -> String {
        // Check custom display names first
        if let Some(name) = self.agent_display_names.get(id) {
            return name.clone();
        }
        // Fall back to built-in
        for agent in AgentType::all() {
            if agent.id() == id {
                return agent.display_name().to_string();
            }
        }
        id.to_string()
    }

    /// Add a custom agent
    pub fn add_custom_agent(&mut self, id: String, display_name: String, global_path: String, project_pattern: String) {
        self.agent_global_paths.insert(id.clone(), global_path);
        self.agent_project_patterns.insert(id.clone(), project_pattern);
        self.agent_display_names.insert(id.clone(), display_name);
        if !self.custom_agent_ids.contains(&id) {
            self.custom_agent_ids.push(id);
        }
    }

    /// Remove a custom agent
    pub fn remove_custom_agent(&mut self, id: &str) {
        self.agent_global_paths.remove(id);
        self.agent_project_patterns.remove(id);
        self.agent_display_names.remove(id);
        self.custom_agent_ids.retain(|i| i != id);
        // If active agent was removed, fallback to claude
        if self.active_agent_id == id {
            self.active_agent_id = "claude".to_string();
        }
    }
}

fn dirs_home() -> String {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstallTarget {
    Global,
    Project { path: String },
}
