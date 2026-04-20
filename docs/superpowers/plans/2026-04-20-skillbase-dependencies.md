# skillbase.json 依赖管理 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 支持读取项目根目录的 `skillbase.json` 作为 Skill 依赖声明，实现依赖解析、状态可视化和一键同步安装。

**Architecture:** 在 Rust 后端新增 `SkillbaseManifest` 模型、semver 范围匹配和依赖解析服务。前端在项目详情页新增 `SkillbasePanel` 组件展示依赖状态，支持一键同步缺失依赖和从已安装 Skill 反向生成 `skillbase.json`。

**Tech Stack:** Rust + serde_json, Vue 3 + TypeScript + Naive UI

---

## Design Decisions

1. **Semver 范围匹配**: 实现轻量级 semver 匹配，支持 `^`、`~`、`>=`、精确匹配和 `*`。不引入额外 crate。
2. **Author → Repo 映射**: 通过扫描所有已配置仓库，匹配 skill 的 `author` 字段和 `name` 字段来解析 `@author/name`。无 author 前缀时退化为按名称匹配。
3. **Personas**: Phase 2 仅存储 `personas` 字段用于往返传递，不实现 persona 解析或安装。
4. **自动更新 skillbase.json**: 不自动更新。提供手动"从已安装生成"功能。
5. **Spec 合规**: 严格遵循 [skillbase.json 官方规范](https://skillbase.space/docs/skill-json-spec)，包含所有字段（`schema_version`、`name`、`version`、`skills`、`personas`、`registry`、`spm`）。
6. **双格式兼容**: `parse_skill_reference` 同时处理 `@author/name`（skillbase.json 格式）和 `author/name`（SKILL.md `dependencies:` 格式）。

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src-tauri/src/models/skill.rs` | 新增 SpmConfig、SkillbaseManifest、DependencyStatus、DependencyEntry、SkillbaseResolution |
| Modify | `src-tauri/src/services/skill_service.rs` | 新增 semver 匹配、parse_skillbase_manifest、resolve_skillbase_dependencies、generate_skillbase_manifest |
| Modify | `src-tauri/src/commands/skill.rs` | 新增 get_skillbase_resolution、sync_skillbase_dependencies、generate_skillbase_json、write_skillbase_json |
| Modify | `src-tauri/src/lib.rs` | 注册 4 个新 IPC 命令 |
| Modify | `src/types/index.ts` | 新增 SkillbaseManifest、DependencyStatus、DependencyEntry、SkillbaseResolution 类型 |
| Modify | `src/stores/skill.ts` | 新增 skillbase 状态和方法 |
| Create | `src/components/common/SkillbasePanel.vue` | 依赖状态可视化面板组件 |
| Modify | `src/views/ProjectDetailView.vue` | 集成 SkillbasePanel |
| Modify | `src/i18n/locales/zh-CN.json` | 新增 skillbase 翻译键 |
| Modify | `src/i18n/locales/en-US.json` | 新增 skillbase 翻译键 |

---

### Task 1: 添加 Rust 数据模型

**Files:**
- Modify: `src-tauri/src/models/skill.rs`

- [ ] **Step 1: 在文件末尾（`SkillComparison` 结构体之后）添加新模型**

在 `src-tauri/src/models/skill.rs` 文件末尾追加：

```rust
/// spm-specific config inside skillbase.json
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SpmConfig {
    #[serde(default)]
    pub default_instance: Option<String>,
}

/// Root of skillbase.json — project-level dependency manifest
/// Spec: https://skillbase.space/docs/skill-json-spec
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillbaseManifest {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub skills: HashMap<String, String>,
    #[serde(default)]
    pub personas: HashMap<String, String>,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub spm: Option<SpmConfig>,
}

/// Status of a single dependency resolution
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DependencyStatus {
    /// Installed and version satisfies the declared range
    Satisfied,
    /// No matching skill found in any repo
    Missing,
    /// Installed but version doesn't match the declared range
    VersionMismatch,
    /// Installed but a newer compatible version is available
    Outdated,
}

/// A single resolved dependency entry
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyEntry {
    pub reference: String,
    pub author: String,
    pub skill_name: String,
    pub version_range: String,
    pub resolved: Option<SkillMeta>,
    pub installed: Option<SkillMeta>,
    pub status: DependencyStatus,
}

/// Complete resolution result for a project's skillbase.json
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillbaseResolution {
    pub manifest: SkillbaseManifest,
    pub dependencies: Vec<DependencyEntry>,
    pub satisfied_count: usize,
    pub missing_count: usize,
    pub mismatch_count: usize,
}
```

- [ ] **Step 2: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功（新类型尚未被使用，但不影响编译）

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models/skill.rs
git commit -m "feat: add SkillbaseManifest and dependency resolution models"
```

---

### Task 2: 添加依赖解析服务

**Files:**
- Modify: `src-tauri/src/services/skill_service.rs`

- [ ] **Step 1: 更新 use 声明**

在文件顶部的 `use crate::models::skill::{...};` 中添加新的类型导入。将整行替换为：

```rust
use crate::models::skill::{
    DependencyEntry, DependencyStatus, FileDiff, FileDiffStatus, InstallStatus, SkillDiff,
    SkillManifestEntry, SkillMeta, SkillbaseManifest, SkillbaseResolution, SkillsManifest,
};
```

在 `use crate::models::skill::{...};` 行之后添加：

```rust
use crate::models::config::RepoConfig;
```

- [ ] **Step 2: 添加 semver 解析和匹配工具函数**

在文件中 `list_installed_skills` 函数之后（`ProjectSkillSummary` 结构体之前）插入：

```rust
/// Parse "X.Y.Z" into (major, minor, patch)
fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let v = version.trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Check if a version string satisfies a semver range
/// Supports: "*", "^X.Y.Z", "~X.Y.Z", ">=X.Y.Z", exact "X.Y.Z"
fn semver_matches(version: &str, range: &str) -> bool {
    let ver = match parse_semver(version) {
        Some(v) => v,
        None => return false,
    };

    let range = range.trim();

    if range == "*" || range.is_empty() {
        return true;
    }

    if let Some(rest) = range.strip_prefix('^') {
        let target = match parse_semver(rest) {
            Some(v) => v,
            None => return false,
        };
        if target.0 != 0 {
            ver.0 == target.0 && ver >= target
        } else if target.1 != 0 {
            ver.0 == 0 && ver.1 == target.1 && ver >= target
        } else {
            ver == target
        }
    } else if let Some(rest) = range.strip_prefix('~') {
        let target = match parse_semver(rest) {
            Some(v) => v,
            None => return false,
        };
        ver.0 == target.0 && ver.1 == target.1 && ver >= target
    } else if let Some(rest) = range.strip_prefix(">=") {
        let target = match parse_semver(rest.trim()) {
            Some(v) => v,
            None => return false,
        };
        ver >= target
    } else {
        parse_semver(range).map(|t| ver == t).unwrap_or(false)
    }
}

/// Parse "@author/name" or "author/name" into (author, name).
/// Returns ("", name) if no slash is found.
/// Handles both skillbase.json format ("@author/name") and
/// SKILL.md dependencies format ("author/name").
fn parse_skill_reference(reference: &str) -> (String, String) {
    let trimmed = reference.trim_start_matches('@');
    if let Some(slash_pos) = trimmed.find('/') {
        return (
            trimmed[..slash_pos].to_string(),
            trimmed[slash_pos + 1..].to_string(),
        );
    }
    (String::new(), reference.to_string())
}
```

- [ ] **Step 3: 添加 skillbase.json 解析函数**

在 `parse_skill_reference` 函数之后插入：

```rust
/// Parse skillbase.json from a project root directory
pub fn parse_skillbase_manifest(project_path: &str) -> Result<SkillbaseManifest, String> {
    let path = Path::new(project_path).join("skillbase.json");
    if !path.exists() {
        return Err("skillbase.json not found in project root".to_string());
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Read skillbase.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Parse skillbase.json: {}", e))
}
```

- [ ] **Step 4: 添加依赖解析函数**

在 `parse_skillbase_manifest` 函数之后插入：

```rust
/// Resolve all dependencies declared in a project's skillbase.json
pub fn resolve_skillbase_dependencies(
    project_path: &str,
    repos: &[RepoConfig],
    agent_project_patterns: &std::collections::HashMap<String, String>,
    active_agent_id: &str,
) -> Result<SkillbaseResolution, String> {
    let manifest = parse_skillbase_manifest(project_path)?;

    let pattern = agent_project_patterns
        .get(active_agent_id)
        .cloned()
        .unwrap_or_else(|| "{project}/.claude/skills".to_string());
    let local_dir = pattern.replace("{project}", project_path);

    // Build index of all remote skills: (author, name) -> SkillMeta
    let mut remote_index: std::collections::HashMap<(String, String), SkillMeta> =
        std::collections::HashMap::new();
    let mut remote_by_name: std::collections::HashMap<String, SkillMeta> =
        std::collections::HashMap::new();
    for repo in repos {
        if !repo.enabled {
            continue;
        }
        let entries = load_skill_entries(&repo.cache_path);
        for entry in &entries {
            if let Ok(meta) = build_remote_skill_meta(&repo.cache_path, entry, Some(&repo.id)) {
                let author = meta.author.clone().unwrap_or_default();
                remote_index.insert((author.clone(), meta.name.clone()), meta.clone());
                remote_by_name.insert(meta.name.clone(), meta);
            }
        }
    }

    // Build index of locally installed skills: name -> SkillMeta
    let local_names = list_installed_skills(&local_dir)?;
    let mut local_map: std::collections::HashMap<String, SkillMeta> =
        std::collections::HashMap::new();
    for name in &local_names {
        if let Ok(meta) = build_local_skill_meta(&local_dir, name) {
            local_map.insert(meta.name.clone(), meta);
        }
    }

    let mut dependencies = Vec::new();
    let mut satisfied_count = 0;
    let mut missing_count = 0;
    let mut mismatch_count = 0;

    for (reference, version_range) in &manifest.skills {
        let (author, skill_name) = parse_skill_reference(reference);

        // Try to find remote match: first by (author, name), then by name only
        let resolved = if !author.is_empty() {
            remote_index
                .get(&(author.clone(), skill_name.clone()))
                .cloned()
                .or_else(|| remote_by_name.get(&skill_name).cloned())
        } else {
            remote_by_name.get(&skill_name).cloned()
        };

        let installed = local_map.get(&skill_name).cloned();

        let status = match &installed {
            Some(inst) => {
                if semver_matches(&inst.version, version_range) {
                    DependencyStatus::Satisfied
                } else {
                    DependencyStatus::VersionMismatch
                }
            }
            None => DependencyStatus::Missing,
        };

        match &status {
            DependencyStatus::Satisfied => satisfied_count += 1,
            DependencyStatus::Missing => missing_count += 1,
            DependencyStatus::VersionMismatch => mismatch_count += 1,
            DependencyStatus::Outdated => {}
        }

        dependencies.push(DependencyEntry {
            reference: reference.clone(),
            author,
            skill_name,
            version_range: version_range.clone(),
            resolved,
            installed,
            status,
        });
    }

    Ok(SkillbaseResolution {
        manifest,
        dependencies,
        satisfied_count,
        missing_count,
        mismatch_count,
    })
}
```

- [ ] **Step 5: 添加生成 skillbase.json 函数**

在 `resolve_skillbase_dependencies` 函数之后插入：

```rust
/// Generate skillbase.json content from currently installed skills
pub fn generate_skillbase_manifest(
    project_path: &str,
    project_name: &str,
    agent_project_patterns: &std::collections::HashMap<String, String>,
    active_agent_id: &str,
) -> Result<String, String> {
    let pattern = agent_project_patterns
        .get(active_agent_id)
        .cloned()
        .unwrap_or_else(|| "{project}/.claude/skills".to_string());
    let local_dir = pattern.replace("{project}", project_path);

    let local_names = list_installed_skills(&local_dir)?;
    let mut skills = std::collections::HashMap::new();

    for name in &local_names {
        if let Ok(meta) = build_local_skill_meta(&local_dir, name) {
            let author = meta.author.as_deref().unwrap_or("local");
            let version = if meta.version.is_empty() {
                "*".to_string()
            } else {
                format!("^{}", meta.version)
            };
            skills.insert(format!("@{}/{}", author, name), version);
        }
    }

    let manifest = SkillbaseManifest {
        schema_version: 1,
        name: project_name.to_string(),
        version: "1.0.0".to_string(),
        skills,
        personas: std::collections::HashMap::new(),
        registry: None,
        spm: None,
    };

    serde_json::to_string_pretty(&manifest).map_err(|e| format!("Serialize: {}", e))
}
```

- [ ] **Step 6: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/services/skill_service.rs
git commit -m "feat: add skillbase.json parsing, dependency resolution and generation services"
```

---

### Task 3: 添加 IPC 命令

**Files:**
- Modify: `src-tauri/src/commands/skill.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 `commands/skill.rs` 顶部更新 use 声明**

将现有的 `use crate::models::skill::SkillComparison;` 替换为：

```rust
use crate::models::skill::{SkillComparison, SkillbaseResolution};
```

- [ ] **Step 2: 在 `commands/skill.rs` 文件末尾（`uninstall_skill` 函数之后）添加 4 个新命令**

```rust
/// Resolve skillbase.json dependencies for a project
#[tauri::command]
pub fn get_skillbase_resolution(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<SkillbaseResolution, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    skill_service::resolve_skillbase_dependencies(&project_path, &repos, &patterns, &active_id)
}

/// Install all missing/mismatched dependencies from skillbase.json
#[tauri::command]
pub fn sync_skillbase_dependencies(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<Vec<String>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let repos = config.repos.clone();
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    let target_dir = config.active_project_dir(&project_path);
    drop(config);

    let resolution = skill_service::resolve_skillbase_dependencies(
        &project_path,
        &repos,
        &patterns,
        &active_id,
    )?;

    let mut results = Vec::new();
    for dep in &resolution.dependencies {
        match dep.status {
            skill_service::DependencyStatus::Missing
            | skill_service::DependencyStatus::VersionMismatch => {
                let mut installed = false;
                for repo in &repos {
                    if !repo.enabled {
                        continue;
                    }
                    let entries = skill_service::load_skill_entries(&repo.cache_path);
                    if let Some(entry) = entries.iter().find(|s| s.name == dep.skill_name) {
                        match skill_service::install_skill_to_dir(
                            &repo.cache_path,
                            &entry.path,
                            &target_dir,
                        ) {
                            Ok(()) => {
                                results.push(format!("{}: OK", dep.reference));
                                installed = true;
                                break;
                            }
                            Err(e) => {
                                results.push(format!("{}: FAILED - {}", dep.reference, e));
                            }
                        }
                    }
                }
                if !installed && !results.iter().any(|r| r.starts_with(&dep.reference)) {
                    results.push(format!("{}: FAILED - not found in any repo", dep.reference));
                }
            }
            _ => results.push(format!("{}: SKIP (satisfied)", dep.reference)),
        }
    }
    Ok(results)
}

/// Generate skillbase.json content from currently installed skills
#[tauri::command]
pub fn generate_skillbase_json(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let active_id = config.active_agent_id.clone();
    let patterns = config.agent_project_patterns.clone();
    drop(config);

    let project_name = std::path::Path::new(&project_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-project".to_string());

    skill_service::generate_skillbase_manifest(&project_path, &project_name, &patterns, &active_id)
}

/// Write skillbase.json content to project root
#[tauri::command]
pub fn write_skillbase_json(project_path: String, content: String) -> Result<(), String> {
    let path = std::path::Path::new(&project_path).join("skillbase.json");
    std::fs::write(&path, &content).map_err(|e| format!("Write skillbase.json: {}", e))
}
```

- [ ] **Step 3: 在 `lib.rs` 注册新命令**

在 `lib.rs` 的 `invoke_handler` 宏中，在 `skill::uninstall_skill,` 行之后添加：

```rust
            skill::get_skillbase_resolution,
            skill::sync_skillbase_dependencies,
            skill::generate_skillbase_json,
            skill::write_skillbase_json,
```

- [ ] **Step 4: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/skill.rs src-tauri/src/lib.rs
git commit -m "feat: add IPC commands for skillbase.json operations"
```

---

### Task 4: 添加 TypeScript 类型和 Store 方法

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/stores/skill.ts`

- [ ] **Step 1: 在 `src/types/index.ts` 文件末尾添加新类型**

在文件末尾（`OperationRecord` 接口之后）追加：

```typescript
// Skillbase types
export interface SpmConfig {
  default_instance?: string
}

export interface SkillbaseManifest {
  schema_version: number
  name: string
  version: string
  skills: Record<string, string>
  personas: Record<string, string>
  registry?: string
  spm?: SpmConfig
}

export type DependencyStatus = 'Satisfied' | 'Missing' | 'VersionMismatch' | 'Outdated'

export interface DependencyEntry {
  reference: string
  author: string
  skillName: string
  versionRange: string
  resolved: SkillMeta | null
  installed: SkillMeta | null
  status: DependencyStatus
}

export interface SkillbaseResolution {
  manifest: SkillbaseManifest
  dependencies: DependencyEntry[]
  satisfiedCount: number
  missingCount: number
  mismatchCount: number
}
```

- [ ] **Step 2: 在 `src/stores/skill.ts` 中添加 skillbase 状态和方法**

将 import 行替换为：

```typescript
import type { SkillComparison, ProjectSkillSummary, SyncResult, SkillDiff, OperationRecord, SkillbaseResolution } from '@/types'
```

在 `skillDiff` ref 声明之后添加新状态：

```typescript
  const skillbaseResolution = ref<SkillbaseResolution | null>(null)
  const skillbaseSyncing = ref(false)
```

在 `clearHistory` 函数之后、`return` 语句之前添加新方法：

```typescript
  async function loadSkillbase(projectPath: string) {
    try {
      skillbaseResolution.value = await invoke<SkillbaseResolution>('get_skillbase_resolution', {
        projectPath,
      })
    } catch {
      skillbaseResolution.value = null
    }
  }

  async function syncSkillbase(projectPath: string): Promise<string[]> {
    skillbaseSyncing.value = true
    try {
      return await invoke<string[]>('sync_skillbase_dependencies', { projectPath })
    } finally {
      skillbaseSyncing.value = false
    }
  }

  async function generateSkillbase(projectPath: string): Promise<string> {
    return await invoke<string>('generate_skillbase_json', { projectPath })
  }

  async function writeSkillbase(projectPath: string, content: string): Promise<void> {
    await invoke('write_skillbase_json', { projectPath, content })
  }
```

更新 `return` 语句，添加新的导出。将现有的 return 替换为：

```typescript
  return {
    globalComparisons,
    projectComparisons,
    projectsOverview,
    syncing,
    loading,
    lastSyncResult,
    skillDiff,
    skillbaseResolution,
    skillbaseSyncing,
    syncRemote,
    loadGlobalSkills,
    loadProjectSkills,
    loadProjectsOverview,
    installSkill,
    updateSkill,
    batchUpdate,
    uninstallSkill,
    loadSkillDiff,
    getOperationHistory,
    rollbackOperation,
    clearHistory,
    loadSkillbase,
    syncSkillbase,
    generateSkillbase,
    writeSkillbase,
  }
```

- [ ] **Step 3: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts src/stores/skill.ts
git commit -m "feat: add SkillbaseResolution types and store methods"
```

---

### Task 5: 创建 SkillbasePanel 组件

**Files:**
- Create: `src/components/common/SkillbasePanel.vue`

- [ ] **Step 1: 创建 SkillbasePanel.vue**

创建 `src/components/common/SkillbasePanel.vue`：

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NTag, NText, NModal, NInput, NSpace } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import type { SkillbaseResolution, DependencyStatus } from '@/types'

const { t } = useI18n()

const props = defineProps<{
  resolution: SkillbaseResolution
  syncing: boolean
}>()

const emit = defineEmits<{
  sync: []
  generate: []
}>()

const hasUnsatisfied = computed(() => {
  return props.resolution.missingCount + props.resolution.mismatchCount > 0
})

function statusType(status: DependencyStatus): 'success' | 'warning' | 'error' | 'default' {
  switch (status) {
    case 'Satisfied': return 'success'
    case 'Missing': return 'error'
    case 'VersionMismatch': return 'warning'
    case 'Outdated': return 'warning'
  }
}

function statusLabel(status: DependencyStatus): string {
  switch (status) {
    case 'Satisfied': return t('skillbase.satisfied')
    case 'Missing': return t('skillbase.missing')
    case 'VersionMismatch': return t('skillbase.mismatch')
    case 'Outdated': return t('skillbase.outdated')
  }
}
</script>

<template>
  <div class="skillbase-panel">
    <div class="panel-header">
      <div class="panel-title">
        <NText strong>skillbase.json</NText>
        <NText depth="3" style="font-size: 12px; margin-left: 8px">
          {{ resolution.manifest.name }}
        </NText>
      </div>
      <div class="panel-stats">
        <NTag size="small" round type="success">
          {{ resolution.satisfiedCount }} {{ t('skillbase.satisfied') }}
        </NTag>
        <NTag v-if="resolution.missingCount > 0" size="small" round type="error">
          {{ resolution.missingCount }} {{ t('skillbase.missing') }}
        </NTag>
        <NTag v-if="resolution.mismatchCount > 0" size="small" round type="warning">
          {{ resolution.mismatchCount }} {{ t('skillbase.mismatch') }}
        </NTag>
      </div>
      <div class="panel-actions">
        <NButton
          v-if="hasUnsatisfied"
          size="small"
          type="primary"
          :loading="syncing"
          @click="emit('sync')"
        >
          {{ t('skillbase.syncDeps') }}
        </NButton>
      </div>
    </div>
    <div class="dep-list">
      <div
        v-for="dep in resolution.dependencies"
        :key="dep.reference"
        class="dep-item"
      >
        <span class="dep-dot" :class="dep.status.toLowerCase()"></span>
        <span class="dep-ref">{{ dep.reference }}</span>
        <NText depth="3" style="font-size: 12px">{{ dep.versionRange }}</NText>
        <NTag size="tiny" round :type="statusType(dep.status)">
          {{ statusLabel(dep.status) }}
        </NTag>
        <NText v-if="dep.installed" depth="3" style="font-size: 11px; margin-left: auto">
          v{{ dep.installed.version || '?' }}
        </NText>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skillbase-panel {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 12px;
}
.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-secondary);
  flex-wrap: wrap;
}
.panel-title {
  display: flex;
  align-items: baseline;
  gap: 4px;
}
.panel-stats {
  display: flex;
  gap: 4px;
  margin-left: auto;
}
.panel-actions {
  display: flex;
  gap: 4px;
}
.dep-list {
  padding: 4px 0;
}
.dep-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  font-size: 13px;
}
.dep-item:hover {
  background: var(--color-bg-hover);
}
.dep-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dep-dot.satisfied { background: var(--color-status-same, #52c41a); }
.dep-dot.missing { background: var(--color-status-remote, #ff4d4f); }
.dep-dot.versionmismatch { background: var(--color-status-outdated, #faad14); }
.dep-dot.outdated { background: var(--color-status-outdated, #faad14); }
.dep-ref {
  font-family: var(--font-mono);
  font-weight: 500;
}
</style>
```

- [ ] **Step 2: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src/components/common/SkillbasePanel.vue
git commit -m "feat: create SkillbasePanel component for dependency status visualization"
```

---

### Task 6: 集成到 ProjectDetailView

**Files:**
- Modify: `src/views/ProjectDetailView.vue`

- [ ] **Step 1: 添加 SkillbasePanel 导入**

在 `<script setup>` 中，在 `SkillPreviewModal` 导入行之后添加：

```typescript
import SkillbasePanel from '@/components/common/SkillbasePanel.vue'
```

- [ ] **Step 2: 添加 skillbase 相关状态和方法**

在 `firstLoaded` ref 之后添加：

```typescript
const showGenerateModal = ref(false)
const generatedContent = ref('')
```

在 `handleBatchUninstall` 函数之后添加：

```typescript
async function loadSkillbaseData() {
  if (!projectStore.projectPath) return
  await skillStore.loadSkillbase(projectStore.projectPath)
}

async function handleSkillbaseSync() {
  if (!projectStore.projectPath) return
  try {
    const results = await skillStore.syncSkillbase(projectStore.projectPath)
    await loadProjectSkills()
    await loadSkillbaseData()
    const failed = results.filter(r => r.includes('FAILED'))
    if (failed.length > 0) {
      message.warning(t('skillbase.syncPartial', { count: failed.length }))
    } else {
      message.success(t('skillbase.syncSuccess'))
    }
  } catch (e: any) {
    message.error(t('skillbase.syncFailed', { error: e }))
  }
}

async function handleGenerateSkillbase() {
  if (!projectStore.projectPath) return
  try {
    generatedContent.value = await skillStore.generateSkillbase(projectStore.projectPath)
    showGenerateModal.value = true
  } catch (e: any) {
    message.error(t('skillbase.generateFailed', { error: e }))
  }
}

async function handleSaveSkillbase() {
  if (!projectStore.projectPath) return
  try {
    await skillStore.writeSkillbase(projectStore.projectPath, generatedContent.value)
    showGenerateModal.value = false
    await loadSkillbaseData()
    message.success(t('skillbase.saveSuccess'))
  } catch (e: any) {
    message.error(t('skillbase.saveFailed', { error: e }))
  }
}
```

- [ ] **Step 3: 在 onMounted 中加载 skillbase 数据**

在 `onMounted` 回调中，在 `firstLoaded.value = true` 之前添加：

```typescript
  await loadSkillbaseData()
```

- [ ] **Step 4: 在 `loadProjectSkills` 完成后也刷新 skillbase**

在 `loadProjectSkills` 函数末尾（`catch` 块之后、函数关闭大括号之前）不修改——改用 `handleSync` 来触发刷新。在 `handleSync` 函数中，在 `await loadProjectSkills()` 之后添加：

```typescript
    await loadSkillbaseData()
```

- [ ] **Step 5: 在模板中添加 SkillbasePanel 和生成按钮**

在 `<template>` 的 `<div class="view-header">` 结束标签 `</div>` 之前、`</template>` 的 `toolbar` div 之后，`</template>` 之前的位置——即在 `</template>` 的 `</NSpin>` 之前、在 `toolbar` div 和 `stats-bar` div 之间——需要找到正确的插入位置。

找到 `<div class="toolbar">` 块的结束 `</div>`，在该行之后、`</div>` 关闭 `view-header` 之前，添加：

```html
        <div v-if="skillStore.skillbaseResolution" class="skillbase-section">
          <SkillbasePanel
            :resolution="skillStore.skillbaseResolution"
            :syncing="skillStore.skillbaseSyncing"
            @sync="handleSkillbaseSync"
          />
        </div>
        <div v-else class="skillbase-empty-action">
          <NButton size="tiny" quaternary @click="handleGenerateSkillbase">
            {{ t('skillbase.generateLabel') }}
          </NButton>
        </div>
```

然后在文件末尾、`</template>` 关闭标签之前（即 `OperationHistoryPanel` 之后、`</div>` 关闭 `project-detail-view` 之前）添加生成模态框：

```html
    <NModal v-model:show="showGenerateModal" preset="dialog" :title="t('skillbase.generateTitle')" positive-text="Save" negative-text="Cancel" @positive-click="handleSaveSkillbase">
      <NInput type="textarea" :value="generatedContent" readonly :rows="12" style="font-family: var(--font-mono); font-size: 12px" />
    </NModal>
```

同时在 `<script setup>` 的 import 中确认 `NModal` 已被导入。在 naive-ui 的导入行中添加 `NModal`：

将 naive-ui 导入行替换为：

```typescript
import { NButton, NInput, NSpin, NSkeleton, NBreadcrumb, NBreadcrumbItem, NText, NModal, useMessage } from 'naive-ui'
```

- [ ] **Step 6: 添加样式**

在 `<style scoped>` 中，`.skeleton-wrapper` 规则之后添加：

```css
.skillbase-section {
  margin-top: 8px;
}
.skillbase-empty-action {
  margin-top: 4px;
}
```

- [ ] **Step 7: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 8: Commit**

```bash
git add src/views/ProjectDetailView.vue
git commit -m "feat: integrate SkillbasePanel into ProjectDetailView"
```

---

### Task 7: 添加 i18n 翻译

**Files:**
- Modify: `src/i18n/locales/zh-CN.json`
- Modify: `src/i18n/locales/en-US.json`

- [ ] **Step 1: 在 `zh-CN.json` 中添加 skillbase 翻译键**

在 `history` 对象的结束 `}` 之后、`guide` 对象之前添加（注意保持 JSON 格式正确，在 `history` 的 `}` 后加逗号）：

```json
  "skillbase": {
    "satisfied": "已满足",
    "missing": "缺失",
    "mismatch": "版本不匹配",
    "outdated": "可更新",
    "syncDeps": "同步依赖",
    "syncSuccess": "依赖同步完成",
    "syncPartial": "{count} 个依赖同步失败",
    "syncFailed": "同步失败: {error}",
    "generateLabel": "生成 skillbase.json",
    "generateTitle": "生成 skillbase.json",
    "generateFailed": "生成失败: {error}",
    "saveSuccess": "skillbase.json 已保存",
    "saveFailed": "保存失败: {error}"
  },
```

注意：需要在 `history` 块的最后一个 `}` 后面加逗号 `,`，然后在 `guide` 块之前插入上述内容。

- [ ] **Step 2: 在 `en-US.json` 中添加 skillbase 翻译键**

在 `history` 对象的结束 `}` 之后、`guide` 对象之前添加同样的结构（注意逗号）：

```json
  "skillbase": {
    "satisfied": "Satisfied",
    "missing": "Missing",
    "mismatch": "Mismatch",
    "outdated": "Outdated",
    "syncDeps": "Sync Dependencies",
    "syncSuccess": "Dependencies synced",
    "syncPartial": "{count} dependency sync(s) failed",
    "syncFailed": "Sync failed: {error}",
    "generateLabel": "Generate skillbase.json",
    "generateTitle": "Generate skillbase.json",
    "generateFailed": "Generate failed: {error}",
    "saveSuccess": "skillbase.json saved",
    "saveFailed": "Save failed: {error}"
  },
```

- [ ] **Step 3: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/i18n/locales/zh-CN.json src/i18n/locales/en-US.json
git commit -m "feat: add i18n keys for skillbase dependency management"
```

---

### Task 8: 集成构建验证

**Files:** 无变更

- [ ] **Step 1: 完整前端构建**

Run: `npm run build`
Expected: vue-tsc 类型检查 + vite build 成功

- [ ] **Step 2: Rust 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 3: 手动功能验证**

启动 `npm run tauri dev`，验证：

1. 应用正常启动，无崩溃
2. 进入项目详情页，无 skillbase.json 时显示"生成 skillbase.json"按钮
3. 创建一个测试 `skillbase.json` 文件到项目根目录：
   ```json
   {
     "schema_version": 1,
     "name": "test-project",
     "version": "1.0.0",
     "skills": {
       "@core/docx": "^1.0.0"
     },
     "personas": {},
     "registry": "https://registry.skillbase.space"
   }
   ```
4. 刷新页面，SkillbasePanel 正确显示依赖状态
5. 点击"同步依赖"按钮，缺失的 skill 被安装
6. 点击"生成 skillbase.json"按钮，弹出模态框显示生成内容
7. 保存后 skillbase.json 正确写入项目根目录
8. 中英文切换正常，翻译键正确显示
