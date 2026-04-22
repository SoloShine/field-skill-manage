# Local Skill Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add local skill migration between different AI agents' skill directories, allowing users to transfer skills from one agent (e.g., Cursor) to another (e.g., Claude Code) without a remote repository.

**Architecture:** 3-step wizard dialog (select source → select skills → confirm) triggered from toolbar buttons in GlobalView and ProjectDetailView. Backend adds 3 IPC commands reusing existing skill scanning, diff, and copy infrastructure.

**Tech Stack:** Rust (Tauri commands/services), Vue 3 + TypeScript (frontend), Naive UI (components), vue-i18n (localization)

**Design Spec:** `docs/superpowers/specs/2026-04-23-local-skill-migration-design.md`

---

## File Structure

| File | Responsibility | Action |
|------|---------------|--------|
| `src-tauri/src/models/skill.rs` | New migration types (MigrateConflictStatus, MigrateSkillEntry, ScanAgentSkillsResult, ConflictResolution, MigrateResult) | Modify (append) |
| `src-tauri/src/services/skill_service.rs` | scan_agent_skills_dir, migrate_skills_to_dir functions | Modify (append) |
| `src-tauri/src/commands/skill.rs` | 3 new IPC commands: scan_agent_skills, migrate_skills, get_migrate_skill_diff | Modify (append) |
| `src-tauri/src/lib.rs` | Register 3 new commands | Modify (lines ~62) |
| `src/types/index.ts` | TypeScript types matching Rust models | Modify (append) |
| `src/components/common/MigrateDialog.vue` | 3-step wizard dialog component | Create |
| `src/stores/skill.ts` | Migration state and methods | Modify |
| `src/views/GlobalView.vue` | Add "迁移" button to toolbar | Modify |
| `src/views/ProjectDetailView.vue` | Add "迁移" button to toolbar | Modify |
| `src/i18n/locales/zh-CN.json` | Chinese translations | Modify |
| `src/i18n/locales/en-US.json` | English translations | Modify |

---

### Task 1: Rust Models — Add Migration Types

**Files:**
- Modify: `src-tauri/src/models/skill.rs` (append after existing types, ~line 266)

- [ ] **Step 1: Add migration types to skill.rs**

Append the following at the end of `src-tauri/src/models/skill.rs`:

```rust
// --- Migration types ---

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum MigrateConflictStatus {
    NewTarget,
    SameContent,
    DifferentVersion,
    ContentDiffers,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MigrateSkillEntry {
    pub name: String,
    pub version: String,
    pub description: String,
    pub path: String,
    pub conflict_status: MigrateConflictStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanAgentSkillsResult {
    pub agent_id: String,
    pub agent_display_name: String,
    pub source_dir: String,
    pub skills: Vec<MigrateSkillEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum ConflictResolution {
    Skip,
    Overwrite,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateResult {
    pub migrated: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<(String, String)>,
}
```

- [ ] **Step 2: Verify Rust compilation**

Run: `cd d:/Project/field-skill-manage && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models/skill.rs
git commit -m "feat(migration): add Rust migration types"
```

---

### Task 2: Rust Service — Migration Logic

**Files:**
- Modify: `src-tauri/src/services/skill_service.rs` (append after existing functions)

- [ ] **Step 1: Add scan_agent_skills_dir function**

Append to `src-tauri/src/services/skill_service.rs`:

```rust
/// Scan a specific agent's skill directory for skills with SKILL.md.
/// Compares each found skill against the target directory using the same
/// 4-level comparison strategy as remote/local comparison.
pub fn scan_agent_skills_dir(
    source_dir: &str,
    target_dir: &str,
) -> Result<crate::models::skill::ScanAgentSkillsResult, String> {
    use crate::models::skill::{MigrateConflictStatus, MigrateSkillEntry, ScanAgentSkillsResult};

    let source_path = Path::new(source_dir);
    if !source_path.exists() {
        return Err(format!("Source directory does not exist: {}", source_dir));
    }

    let mut skills = Vec::new();

    // Scan subdirectories containing SKILL.md
    if let Ok(rd) = std::fs::read_dir(source_path) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') {
                    continue;
                }
            }

            let skill_md = p.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }

            let dir_name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let fm = parse_skill_frontmatter(p.to_string_lossy().as_ref()).ok();

            let name = fm
                .as_ref()
                .map(|f| f.name.clone())
                .filter(|n| !n.is_empty())
                .unwrap_or(dir_name);

            let version = fm
                .as_ref()
                .map(|f| f.version.clone())
                .unwrap_or_default();

            let description = fm
                .as_ref()
                .map(|f| f.description.clone())
                .unwrap_or_default();

            let target_skill_dir = Path::new(target_dir).join(&name);
            let conflict_status = if !target_skill_dir.exists() {
                MigrateConflictStatus::NewTarget
            } else {
                let src_hash = crate::services::hash_service::aggregate_sha256(&p).ok();
                let dst_hash = crate::services::hash_service::aggregate_sha256(&target_skill_dir).ok();

                match (&src_hash, &dst_hash) {
                    (Some(sh), Some(dh)) if sh.value == dh.value => {
                        MigrateConflictStatus::SameContent
                    }
                    _ => {
                        let src_version = version.clone();
                        let dst_fm = parse_skill_frontmatter(target_skill_dir.to_string_loss().as_ref()).ok();
                        let dst_version = dst_fm.map(|f| f.version).unwrap_or_default();

                        if !src_version.is_empty() && !dst_version.is_empty() && src_version != dst_version {
                            MigrateConflictStatus::DifferentVersion
                        } else {
                            MigrateConflictStatus::ContentDiffers
                        }
                    }
                }
            };

            skills.push(MigrateSkillEntry {
                name,
                version,
                description,
                path: p.to_string_lossy().to_string(),
                conflict_status,
            });
        }
    }

    Ok(ScanAgentSkillsResult {
        agent_id: String::new(),
        agent_display_name: String::new(),
        source_dir: source_dir.to_string(),
        skills,
    })
}

/// Migrate selected skills from source directory to target directory.
pub fn migrate_skills_to_dir(
    source_dir: &str,
    target_dir: &str,
    skill_names: &[String],
    conflict_map: &std::collections::HashMap<String, crate::models::skill::ConflictResolution>,
) -> Result<crate::models::skill::MigrateResult, String> {
    use crate::models::skill::{ConflictResolution, MigrateResult};

    let mut migrated = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();

    // Ensure target directory exists
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("Create target dir: {}", e))?;

    for name in skill_names {
        let src = Path::new(source_dir).join(name);
        let dst = Path::new(target_dir).join(name);

        if !src.exists() {
            failed.push((name.clone(), "Source skill not found".to_string()));
            continue;
        }

        if dst.exists() {
            let resolution = conflict_map.get(name);
            match resolution {
                Some(ConflictResolution::Skip) | None => {
                    skipped.push(name.clone());
                    continue;
                }
                Some(ConflictResolution::Overwrite) => {
                    std::fs::remove_dir_all(&dst)
                        .map_err(|e| format!("Remove old dir: {}", e))?;
                }
            }
        }

        match copy_dir_recursive(&src, &dst) {
            Ok(_) => migrated.push(name.clone()),
            Err(e) => failed.push((name.clone(), e)),
        }
    }

    Ok(MigrateResult {
        migrated,
        skipped,
        failed,
    })
}
```

- [ ] **Step 2: Verify Rust compilation**

Run: `cd d:/Project/field-skill-manage && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/services/skill_service.rs
git commit -m "feat(migration): add scan and migrate service functions"
```

---

### Task 3: Rust Commands — 3 IPC Commands + Registration

**Files:**
- Modify: `src-tauri/src/commands/skill.rs` (append)
- Modify: `src-tauri/src/lib.rs` (register commands)

- [ ] **Step 1: Add scan_agent_skills command**

Append to `src-tauri/src/commands/skill.rs`:

```rust
#[tauri::command]
pub fn scan_agent_skills(
    state: tauri::State<'_, crate::AppState>,
    agent_id: String,
    scope: String,
    project_path: Option<String>,
) -> Result<crate::models::skill::ScanAgentSkillsResult, String> {
    use crate::services::skill_service;

    let config = state.config.lock().map_err(|e| e.to_string())?;
    let active_id = config.active_agent_id.clone();

    if agent_id == active_id {
        return Err("Cannot migrate from the active agent to itself".to_string());
    }

    // Resolve source dir
    let source_dir = if scope == "project" {
        let pp = project_path.ok_or("project_path required for project scope")?;
        let pattern = config
            .agent_project_patterns
            .get(&agent_id)
            .cloned()
            .unwrap_or_default();
        if pattern.is_empty() {
            return Err(format!("No project pattern configured for agent '{}'", agent_id));
        }
        pattern.replace("{project}", &pp)
    } else {
        config
            .agent_global_paths
            .get(&agent_id)
            .cloned()
            .unwrap_or_default()
    };

    if source_dir.is_empty() {
        return Err(format!("No path configured for agent '{}'", agent_id));
    }

    // Resolve target dir (active agent)
    let target_dir = if scope == "project" {
        let pp = project_path.unwrap();
        config.active_project_dir(&pp)
    } else {
        config.active_global_path()
    };

    let display_name = config.agent_display_name(&agent_id);
    drop(config);

    let mut result = skill_service::scan_agent_skills_dir(&source_dir, &target_dir)?;
    result.agent_id = agent_id;
    result.agent_display_name = display_name;
    Ok(result)
}

#[tauri::command]
pub fn migrate_skills(
    state: tauri::State<'_, crate::AppState>,
    source_agent_id: String,
    skill_names: Vec<String>,
    scope: String,
    project_path: Option<String>,
    conflict_map: std::collections::HashMap<String, crate::models::skill::ConflictResolution>,
) -> Result<crate::models::skill::MigrateResult, String> {
    use crate::services::{history_service, skill_service};
    use crate::models::history::OperationType;

    let config = state.config.lock().map_err(|e| e.to_string())?;

    // Resolve source dir
    let source_dir = if scope == "project" {
        let pp = project_path.clone().ok_or("project_path required for project scope")?;
        let pattern = config
            .agent_project_patterns
            .get(&source_agent_id)
            .cloned()
            .unwrap_or_default();
        pattern.replace("{project}", &pp)
    } else {
        config
            .agent_global_paths
            .get(&source_agent_id)
            .cloned()
            .unwrap_or_default()
    };

    // Resolve target dir
    let target_dir = if scope == "project" {
        let pp = project_path.unwrap();
        config.active_project_dir(&pp)
    } else {
        config.active_global_path()
    };

    drop(config);

    let result = skill_service::migrate_skills_to_dir(
        &source_dir,
        &target_dir,
        &skill_names,
        &conflict_map,
    )?;

    // Record history for each migrated skill
    for name in &result.migrated {
        let _ = history_service::record_operation(
            OperationType::Install,
            name,
            &target_dir,
            Some(&format!("migrate:{}", source_agent_id)),
            None,
            None,
            false,
        );
    }

    Ok(result)
}

#[tauri::command]
pub fn get_migrate_skill_diff(
    state: tauri::State<'_, crate::AppState>,
    source_agent_id: String,
    skill_name: String,
    scope: String,
    project_path: Option<String>,
) -> Result<crate::models::skill::SkillDiff, String> {
    use crate::services::skill_service;

    let config = state.config.lock().map_err(|e| e.to_string())?;

    // Resolve source skill dir
    let source_base = if scope == "project" {
        let pp = project_path.clone().ok_or("project_path required for project scope")?;
        let pattern = config
            .agent_project_patterns
            .get(&source_agent_id)
            .cloned()
            .unwrap_or_default();
        pattern.replace("{project}", &pp)
    } else {
        config
            .agent_global_paths
            .get(&source_agent_id)
            .cloned()
            .unwrap_or_default()
    };

    // Resolve target skill dir
    let target_base = if scope == "project" {
        let pp = project_path.unwrap();
        config.active_project_dir(&pp)
    } else {
        config.active_global_path()
    };

    drop(config);

    let source_dir = std::path::Path::new(&source_base).join(&skill_name);
    let target_dir = std::path::Path::new(&target_base).join(&skill_name);

    skill_service::build_skill_diff(&source_dir, &target_dir)
}
```

- [ ] **Step 2: Register commands in lib.rs**

In `src-tauri/src/lib.rs`, add the 3 new commands to the `invoke_handler` macro. Insert after the existing skill commands (after `skill::write_skillbase_json,`) and before the `// Version` comment:

```rust
skill::scan_agent_skills,
skill::migrate_skills,
skill::get_migrate_skill_diff,
```

- [ ] **Step 3: Verify Rust compilation**

Run: `cd d:/Project/field-skill-manage && cargo check --manifest-path src-tauri/Cargo.toml`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/skill.rs src-tauri/src/lib.rs
git commit -m "feat(migration): add 3 IPC commands for skill migration"
```

---

### Task 4: TypeScript Types

**Files:**
- Modify: `src/types/index.ts` (append after SkillbaseResolution / ProjectDetailData)

- [ ] **Step 1: Add migration types**

Append to `src/types/index.ts` after the `ProjectDetailData` interface:

```typescript
// Migration types
export type MigrateConflictStatus = 'NewTarget' | 'SameContent' | 'DifferentVersion' | 'ContentDiffers'

export interface MigrateSkillEntry {
  name: string
  version: string
  description: string
  path: string
  conflictStatus: MigrateConflictStatus
}

export interface ScanAgentSkillsResult {
  agentId: string
  agentDisplayName: string
  sourceDir: string
  skills: MigrateSkillEntry[]
}

export type ConflictResolution = 'Skip' | 'Overwrite'

export interface MigrateResult {
  migrated: string[]
  skipped: string[]
  failed: [string, string][]
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd d:/Project/field-skill-manage && npx vue-tsc --noEmit`
Expected: No type errors

- [ ] **Step 3: Commit**

```bash
git add src/types/index.ts
git commit -m "feat(migration): add TypeScript migration types"
```

---

### Task 5: i18n Translations

**Files:**
- Modify: `src/i18n/locales/zh-CN.json`
- Modify: `src/i18n/locales/en-US.json`

- [ ] **Step 1: Add Chinese translations**

Add a new `"migration"` section to `src/i18n/locales/zh-CN.json` after the `"skillbase"` section (after line 275, before `"guide"`):

```json
"migration": {
  "title": "迁移 Skill",
  "step1Title": "选择来源",
  "step2Title": "选择 Skills",
  "step3Title": "确认迁移",
  "sourceAgent": "源 Agent",
  "sourceAgentPlaceholder": "选择要迁移的来源 Agent",
  "targetPath": "目标路径",
  "sourcePath": "源路径",
  "selectAll": "全选",
  "deselectAll": "取消全选",
  "selected": "已选 {count} 个",
  "statusNew": "新增",
  "statusSame": "内容相同",
  "statusVersionDiff": "版本不同",
  "statusContentDiff": "内容不同",
  "conflictAction": "冲突处理",
  "skip": "跳过",
  "overwrite": "覆盖",
  "diff": "对比",
  "summary": "将迁移 {migrate} 个，跳过 {skip} 个，覆盖 {overwrite} 个",
  "confirm": "确认迁移",
  "next": "下一步",
  "prev": "上一步",
  "cancel": "取消",
  "scanning": "正在扫描...",
  "noSkillsFound": "未发现可迁移的 Skill",
  "noAgentsAvailable": "没有可用的源 Agent",
  "migrateSuccess": "迁移完成：成功 {ok} 个，跳过 {skip} 个",
  "migratePartial": "迁移完成：成功 {ok} 个，失败 {fail} 个",
  "migrateFailed": "迁移失败: {error}",
  "newGroup": "新增 Skill",
  "conflictGroup": "冲突 Skill",
  "historyLabel": "迁移"
},
```

- [ ] **Step 2: Add English translations**

Add the corresponding `"migration"` section to `src/i18n/locales/en-US.json` after the `"skillbase"` section:

```json
"migration": {
  "title": "Migrate Skills",
  "step1Title": "Select Source",
  "step2Title": "Select Skills",
  "step3Title": "Confirm Migration",
  "sourceAgent": "Source Agent",
  "sourceAgentPlaceholder": "Select source agent to migrate from",
  "targetPath": "Target Path",
  "sourcePath": "Source Path",
  "selectAll": "Select All",
  "deselectAll": "Deselect All",
  "selected": "{count} selected",
  "statusNew": "New",
  "statusSame": "Same Content",
  "statusVersionDiff": "Different Version",
  "statusContentDiff": "Content Differs",
  "conflictAction": "Conflict Resolution",
  "skip": "Skip",
  "overwrite": "Overwrite",
  "diff": "Diff",
  "summary": "Will migrate {migrate}, skip {skip}, overwrite {overwrite}",
  "confirm": "Confirm Migration",
  "next": "Next",
  "prev": "Previous",
  "cancel": "Cancel",
  "scanning": "Scanning...",
  "noSkillsFound": "No skills found to migrate",
  "noAgentsAvailable": "No source agents available",
  "migrateSuccess": "Migration complete: {ok} migrated, {skip} skipped",
  "migratePartial": "Migration complete: {ok} migrated, {fail} failed",
  "migrateFailed": "Migration failed: {error}",
  "newGroup": "New Skills",
  "conflictGroup": "Conflicting Skills",
  "historyLabel": "Migrate"
},
```

- [ ] **Step 3: Verify no JSON syntax errors**

Run: `cd d:/Project/field-skill-manage && node -e "JSON.parse(require('fs').readFileSync('src/i18n/locales/zh-CN.json','utf8')); JSON.parse(require('fs').readFileSync('src/i18n/locales/en-US.json','utf8')); console.log('OK')" `
Expected: `OK`

- [ ] **Step 4: Commit**

```bash
git add src/i18n/locales/zh-CN.json src/i18n/locales/en-US.json
git commit -m "feat(migration): add i18n translations for migration feature"
```

---

### Task 6: Skill Store — Migration State and Methods

**Files:**
- Modify: `src/stores/skill.ts`

- [ ] **Step 1: Add migration imports and state**

In `src/stores/skill.ts`, add the type import at the top (alongside existing type imports):

```typescript
import type { ScanAgentSkillsResult, MigrateResult, ConflictResolution } from '@/types'
```

Add new reactive state inside the `defineStore` function (after existing state declarations like `skillbaseResolution`, `skillbaseSyncing`):

```typescript
const migrateDialogVisible = ref(false)
const migrateScope = ref<'global' | 'project'>('global')
const migrateProjectPath = ref<string | null>(null)
const scanResult = ref<ScanAgentSkillsResult | null>(null)
const migrating = ref(false)
```

- [ ] **Step 2: Add migration methods**

Add these functions before the `return {` statement:

```typescript
function openMigrateDialog(scope: 'global' | 'project', projectPath?: string) {
  migrateScope.value = scope
  migrateProjectPath.value = projectPath ?? null
  scanResult.value = null
  migrateDialogVisible.value = true
}

function closeMigrateDialog() {
  migrateDialogVisible.value = false
  scanResult.value = null
}

async function scanAgentSkills(agentId: string) {
  const args: Record<string, unknown> = {
    agentId,
    scope: migrateScope.value,
  }
  if (migrateScope.value === 'project' && migrateProjectPath.value) {
    args.projectPath = migrateProjectPath.value
  }
  scanResult.value = await invoke<ScanAgentSkillsResult>('scan_agent_skills', args)
}

async function migrateSkills(
  sourceAgentId: string,
  skillNames: string[],
  conflictMap: Record<string, ConflictResolution>,
) {
  migrating.value = true
  try {
    const args: Record<string, unknown> = {
      sourceAgentId,
      skillNames,
      scope: migrateScope.value,
      conflictMap,
    }
    if (migrateScope.value === 'project' && migrateProjectPath.value) {
      args.projectPath = migrateProjectPath.value
    }
    return await invoke<MigrateResult>('migrate_skills', args)
  } finally {
    migrating.value = false
  }
}

async function loadMigrateSkillDiff(sourceAgentId: string, skillName: string) {
  const args: Record<string, unknown> = {
    sourceAgentId,
    skillName,
    scope: migrateScope.value,
  }
  if (migrateScope.value === 'project' && migrateProjectPath.value) {
    args.projectPath = migrateProjectPath.value
  }
  return await invoke<import('@/types').SkillDiff>('get_migrate_skill_diff', args)
}
```

Add these to the `return {` object:

```typescript
migrateDialogVisible,
migrateScope,
migrateProjectPath,
scanResult,
migrating,
openMigrateDialog,
closeMigrateDialog,
scanAgentSkills,
migrateSkills,
loadMigrateSkillDiff,
```

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd d:/Project/field-skill-manage && npx vue-tsc --noEmit`
Expected: No type errors

- [ ] **Step 4: Commit**

```bash
git add src/stores/skill.ts
git commit -m "feat(migration): add migration state and methods to skill store"
```

---

### Task 7: MigrateDialog Component

**Files:**
- Create: `src/components/common/MigrateDialog.vue`

- [ ] **Step 1: Create the wizard dialog component**

Create `src/components/common/MigrateDialog.vue` with the 3-step wizard:

```vue
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NModal, NButton, NSelect, NCheckbox, NSpace, NSpin, NRadioGroup, NRadio, NDivider, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import SkillDiffViewer from '@/components/common/SkillDiffViewer.vue'
import type { MigrateSkillEntry, ConflictResolution, SkillDiff } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const message = useMessage()

const currentStep = ref(1)
const selectedAgentId = ref<string | null>(null)
const selectedSkills = ref<Set<string>>(new Set())
const conflictResolutions = ref<Record<string, ConflictResolution>>({})
const diffSkillName = ref<string | null>(null)
const diffData = ref<SkillDiff | null>(null)
const diffLoading = ref(false)

const sourceAgents = computed(() => {
  const activeId = configStore.config.active_agent_id
  const agents = configStore.config.agent_global_paths
  const result: { label: string; value: string }[] = []

  for (const [id, globalPath] of Object.entries(agents)) {
    if (id === activeId) continue
    // Check if path exists (we can't do filesystem check in frontend,
    // so we include all and let backend report empty)
    const displayName = configStore.getAgentDisplayName(id)
    const pathPreview = skillStore.migrateScope === 'project'
      ? (configStore.config.agent_project_patterns[id] || '').replace('{project}', '...')
      : globalPath
    if (pathPreview) {
      result.push({
        label: `${displayName} → ${pathPreview}`,
        value: id,
      })
    }
  }

  return result
})

const agentOptions = computed(() =>
  sourceAgents.value.map(a => ({ label: a.label, value: a.value }))
)

const skills = computed(() => skillStore.scanResult?.skills ?? [])

const newSkills = computed(() =>
  skills.value.filter(s => s.conflictStatus === 'NewTarget' && selectedSkills.value.has(s.name))
)

const conflictSkills = computed(() =>
  skills.value.filter(
    s => (s.conflictStatus === 'DifferentVersion' || s.conflictStatus === 'ContentDiffers')
      && selectedSkills.value.has(s.name)
  )
)

const sameSkills = computed(() =>
  skills.value.filter(s => s.conflictStatus === 'SameContent')
)

const summaryCounts = computed(() => {
  const conflictSkipped = conflictSkills.value.filter(
    s => conflictResolutions.value[s.name] !== 'Overwrite'
  ).length
  const conflictOverwritten = conflictSkills.value.filter(
    s => conflictResolutions.value[s.name] === 'Overwrite'
  ).length
  return {
    migrate: newSkills.value.length + conflictOverwritten,
    skip: conflictSkipped,
    overwrite: conflictOverwritten,
  }
})

watch(selectedAgentId, async (newId) => {
  if (!newId) return
  currentStep.value = 1
  skillStore.scanResult = null
  try {
    await skillStore.scanAgentSkills(newId)
    // Default selections: select all except SameContent
    const selected = new Set<string>()
    for (const s of skills.value) {
      if (s.conflictStatus !== 'SameContent') {
        selected.add(s.name)
      }
    }
    selectedSkills.value = selected
    // Default conflict resolutions: Skip
    const resolutions: Record<string, ConflictResolution> = {}
    for (const s of skills.value) {
      if (s.conflictStatus === 'DifferentVersion' || s.conflictStatus === 'ContentDiffers') {
        resolutions[s.name] = 'Skip'
      }
    }
    conflictResolutions.value = resolutions
    currentStep.value = 2
  } catch (e) {
    message.error(String(e))
  }
})

function toggleAll(checked: boolean) {
  if (checked) {
    const all = new Set(skills.value.map(s => s.name))
    selectedSkills.value = all
  } else {
    selectedSkills.value = new Set()
  }
}

function toggleSkill(name: string) {
  const next = new Set(selectedSkills.value)
  if (next.has(name)) {
    next.delete(name)
  } else {
    next.add(name)
  }
  selectedSkills.value = next
}

async function handleConfirm() {
  if (!selectedAgentId.value) return
  try {
    const result = await skillStore.migrateSkills(
      selectedAgentId.value,
      Array.from(selectedSkills.value),
      conflictResolutions.value
    )
    if (result.failed.length === 0) {
      message.success(t('migration.migrateSuccess', {
        ok: result.migrated.length,
        skip: result.skipped.length,
      }))
    } else {
      message.warning(t('migration.migratePartial', {
        ok: result.migrated.length,
        fail: result.failed.length,
      }))
    }
    skillStore.closeMigrateDialog()
    // Refresh current view
    if (skillStore.migrateScope === 'global') {
      await skillStore.loadGlobalSkills()
    } else if (skillStore.migrateProjectPath) {
      await skillStore.loadProjectDetail(skillStore.migrateProjectPath)
    }
  } catch (e) {
    message.error(t('migration.migrateFailed', { error: String(e) }))
  }
}

async function showDiff(skillName: string) {
  if (!selectedAgentId.value) return
  diffSkillName.value = skillName
  diffLoading.value = true
  try {
    diffData.value = await skillStore.loadMigrateSkillDiff(selectedAgentId.value, skillName)
  } catch {
    diffData.value = null
  } finally {
    diffLoading.value = false
  }
}

function statusBadgeClass(status: string) {
  switch (status) {
    case 'NewTarget': return 'badge-new'
    case 'SameContent': return 'badge-same'
    case 'DifferentVersion': return 'badge-version'
    case 'ContentDiffers': return 'badge-content'
    default: return ''
  }
}

function statusBadgeText(status: string) {
  switch (status) {
    case 'NewTarget': return t('migration.statusNew')
    case 'SameContent': return t('migration.statusSame')
    case 'DifferentVersion': return t('migration.statusVersionDiff')
    case 'ContentDiffers': return t('migration.statusContentDiff')
    default: return status
  }
}
</script>

<template>
  <NModal
    :show="skillStore.migrateDialogVisible"
    :mask-closable="false"
    preset="card"
    :title="t('migration.title')"
    style="width: 640px; max-width: 90vw;"
    @update:show="(v) => !v && skillStore.closeMigrateDialog()"
  >
    <!-- Step indicator -->
    <div class="step-indicator">
      <div class="step" :class="{ active: currentStep >= 1, current: currentStep === 1 }">
        <span class="step-num">1</span>
        <span class="step-label">{{ t('migration.step1Title') }}</span>
      </div>
      <div class="step-line" :class="{ active: currentStep >= 2 }"></div>
      <div class="step" :class="{ active: currentStep >= 2, current: currentStep === 2 }">
        <span class="step-num">2</span>
        <span class="step-label">{{ t('migration.step2Title') }}</span>
      </div>
      <div class="step-line" :class="{ active: currentStep >= 3 }"></div>
      <div class="step" :class="{ active: currentStep >= 3, current: currentStep === 3 }">
        <span class="step-num">3</span>
        <span class="step-label">{{ t('migration.step3Title') }}</span>
      </div>
    </div>

    <!-- Step 1: Select Source Agent -->
    <div v-if="currentStep === 1" class="step-content">
      <div class="form-group">
        <label>{{ t('migration.sourceAgent') }}</label>
        <NSelect
          v-model:value="selectedAgentId"
          :options="agentOptions"
          :placeholder="t('migration.sourceAgentPlaceholder')"
          :loading="skillStore.scanResult === null && !!selectedAgentId"
        />
      </div>
      <div v-if="agentOptions.length === 0" class="empty-hint">
        {{ t('migration.noAgentsAvailable') }}
      </div>
    </div>

    <!-- Step 2: Select Skills -->
    <div v-if="currentStep === 2" class="step-content">
      <div v-if="skillStore.scanResult" class="scan-info">
        <span>{{ t('migration.sourcePath') }}: {{ skillStore.scanResult.sourceDir }}</span>
      </div>
      <NSpin :show="false">
        <div v-if="skills.length === 0" class="empty-hint">
          {{ t('migration.noSkillsFound') }}
        </div>
        <div v-else class="skill-list">
          <div class="skill-list-header">
            <NCheckbox
              :checked="selectedSkills.size === skills.length && skills.length > 0"
              :indeterminate="selectedSkills.size > 0 && selectedSkills.size < skills.length"
              @update:checked="toggleAll"
            >
              {{ t('migration.selectAll') }}
            </NCheckbox>
            <span class="selected-count">{{ t('migration.selected', { count: selectedSkills.size }) }}</span>
          </div>
          <div
            v-for="skill in skills"
            :key="skill.name"
            class="skill-row"
            :class="{ 'same-content': skill.conflictStatus === 'SameContent' }"
          >
            <NCheckbox
              :checked="selectedSkills.has(skill.name)"
              @update:checked="() => toggleSkill(skill.name)"
            />
            <div class="skill-info">
              <span class="skill-name">{{ skill.name }}</span>
              <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
              <span v-if="skill.description" class="skill-desc">{{ skill.description }}</span>
            </div>
            <span class="status-badge" :class="statusBadgeClass(skill.conflictStatus)">
              {{ statusBadgeText(skill.conflictStatus) }}
            </span>
          </div>
        </div>
      </NSpin>
      <div class="step-actions">
        <NButton @click="currentStep = 1">{{ t('migration.prev') }}</NButton>
        <NButton
          type="primary"
          :disabled="selectedSkills.size === 0"
          @click="currentStep = 3"
        >
          {{ t('migration.next') }} ({{ selectedSkills.size }})
        </NButton>
      </div>
    </div>

    <!-- Step 3: Confirm Migration -->
    <div v-if="currentStep === 3" class="step-content">
      <!-- New skills -->
      <div v-if="newSkills.length > 0" class="confirm-group">
        <h4>{{ t('migration.newGroup') }} ({{ newSkills.length }})</h4>
        <div v-for="skill in newSkills" :key="skill.name" class="confirm-row">
          <span class="skill-name">{{ skill.name }}</span>
          <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
          <span class="status-badge badge-new">{{ t('migration.statusNew') }}</span>
        </div>
      </div>

      <!-- Conflict skills -->
      <div v-if="conflictSkills.length > 0" class="confirm-group">
        <h4>{{ t('migration.conflictGroup') }} ({{ conflictSkills.length }})</h4>
        <div v-for="skill in conflictSkills" :key="skill.name" class="confirm-row conflict-row">
          <div class="conflict-info">
            <span class="skill-name">{{ skill.name }}</span>
            <span class="status-badge" :class="statusBadgeClass(skill.conflictStatus)">
              {{ statusBadgeText(skill.conflictStatus) }}
            </span>
            <NButton text size="tiny" type="info" @click="showDiff(skill.name)">
              {{ t('migration.diff') }}
            </NButton>
          </div>
          <NRadioGroup
            v-model:value="conflictResolutions[skill.name]"
            size="small"
          >
            <NRadio value="Skip">{{ t('migration.skip') }}</NRadio>
            <NRadio value="Overwrite">{{ t('migration.overwrite') }}</NRadio>
          </NRadioGroup>
        </div>
      </div>

      <!-- Diff viewer modal -->
      <NModal
        v-if="diffSkillName"
        :show="!!diffSkillName"
        preset="card"
        :title="`${t('diff.title')} - ${diffSkillName}`"
        style="width: 80vw; max-width: 900px;"
        @update:show="(v) => { if (!v) diffSkillName = null }"
      >
        <NSpin :show="diffLoading">
          <SkillDiffViewer v-if="diffData" :diff="diffData" />
          <div v-else-if="!diffLoading" class="empty-hint">{{ t('diff.noContent') }}</div>
        </NSpin>
      </NModal>

      <!-- Summary -->
      <NDivider />
      <div class="summary-bar">
        {{ t('migration.summary', summaryCounts) }}
      </div>

      <div class="step-actions">
        <NButton @click="currentStep = 2">{{ t('migration.prev') }}</NButton>
        <NButton
          type="primary"
          :loading="skillStore.migrating"
          :disabled="summaryCounts.migrate === 0"
          @click="handleConfirm"
        >
          {{ t('migration.confirm') }}
        </NButton>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin-bottom: 24px;
}
.step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 0.4;
}
.step.active { opacity: 0.7; }
.step.current { opacity: 1; }
.step-num {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  background: var(--n-color, #f0f0f0);
  color: var(--n-text-color, #333);
}
.step.current .step-num {
  background: var(--primary-color, #4a9eff);
  color: #fff;
}
.step-label { font-size: 12px; }
.step-line {
  flex: 1;
  height: 2px;
  background: var(--n-border-color, #e0e0e0);
  margin: 0 12px;
  margin-bottom: 20px;
}
.step-line.active {
  background: var(--primary-color, #4a9eff);
}
.step-content { min-height: 200px; }
.form-group { margin-bottom: 16px; }
.form-group label {
  display: block;
  font-size: 13px;
  margin-bottom: 6px;
  color: var(--n-text-color-2, #666);
}
.empty-hint {
  text-align: center;
  color: var(--n-text-color-3, #999);
  padding: 40px 0;
  font-size: 14px;
}
.scan-info {
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  margin-bottom: 12px;
  padding: 8px;
  background: var(--n-color-embedded, #f9f9f9);
  border-radius: 4px;
}
.skill-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--n-border-color, #eee);
  margin-bottom: 4px;
}
.selected-count { font-size: 12px; color: var(--n-text-color-3, #888); }
.skill-list {
  max-height: 320px;
  overflow-y: auto;
}
.skill-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 4px;
  border-bottom: 1px solid var(--n-border-color-2, #f5f5f5);
}
.skill-row.same-content { opacity: 0.5; }
.skill-info { flex: 1; min-width: 0; }
.skill-name { font-weight: 500; font-size: 14px; }
.skill-version { font-size: 12px; color: var(--n-text-color-3, #888); margin-left: 6px; }
.skill-desc {
  display: block;
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 3px;
  white-space: nowrap;
}
.badge-new { background: #e8f8e8; color: #2d8a2d; }
.badge-same { background: #f0f0f0; color: #888; }
.badge-version { background: #fff3cd; color: #856404; }
.badge-content { background: #ffe8cc; color: #c65600; }
.confirm-group { margin-bottom: 16px; }
.confirm-group h4 { font-size: 13px; margin-bottom: 8px; color: var(--n-text-color-2, #555); }
.confirm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}
.conflict-row {
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 8px 0;
  border-bottom: 1px solid var(--n-border-color-2, #f5f5f5);
}
.conflict-info { display: flex; align-items: center; gap: 8px; }
.summary-bar {
  text-align: center;
  font-size: 14px;
  font-weight: 500;
  padding: 8px;
  background: var(--n-color-embedded, #f9f9f9);
  border-radius: 4px;
}
.step-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd d:/Project/field-skill-manage && npx vue-tsc --noEmit`
Expected: No type errors

- [ ] **Step 3: Commit**

```bash
git add src/components/common/MigrateDialog.vue
git commit -m "feat(migration): add MigrateDialog 3-step wizard component"
```

---

### Task 8: Wire Up Views — Toolbar Buttons + Dialog

**Files:**
- Modify: `src/views/GlobalView.vue`
- Modify: `src/views/ProjectDetailView.vue`

- [ ] **Step 1: Add MigrateDialog to GlobalView**

In `src/views/GlobalView.vue`:

**Import** (add after existing imports, around line 8):
```typescript
import MigrateDialog from '@/components/common/MigrateDialog.vue'
```

**Template** — Add "迁移" button in the toolbar `<NSpace>` (around line 231-238, after the history button):

```html
<NButton @click="skillStore.openMigrateDialog('global')">{{ t('migration.title') }}</NButton>
```

**Template** — Add `<MigrateDialog />` at the end of the template, after all other modal components (after `<OperationHistoryPanel>`).

- [ ] **Step 2: Add MigrateDialog to ProjectDetailView**

In `src/views/ProjectDetailView.vue`:

**Import** (add after existing imports, around line 11):
```typescript
import MigrateDialog from '@/components/common/MigrateDialog.vue'
```

**Template** — Add "迁移" button in the toolbar-left `<div>` (around line 266-269, after the history button):

```html
<NButton @click="skillStore.openMigrateDialog('project', projectStore.projectPath ?? undefined)">{{ t('migration.title') }}</NButton>
```

**Template** — Add `<MigrateDialog />` at the end of the template, after all other modal components.

- [ ] **Step 3: Verify TypeScript compilation**

Run: `cd d:/Project/field-skill-manage && npx vue-tsc --noEmit`
Expected: No type errors

- [ ] **Step 4: Commit**

```bash
git add src/views/GlobalView.vue src/views/ProjectDetailView.vue
git commit -m "feat(migration): wire migrate button and dialog into views"
```

---

### Task 9: Smoke Test

- [ ] **Step 1: Start dev server**

Run: `cd d:/Project/field-skill-manage && npm run tauri dev`

- [ ] **Step 2: Verify GlobalView migration button**

1. Navigate to GlobalView
2. Verify "迁移 Skill" button appears in toolbar
3. Click it — dialog opens
4. Select a source agent from dropdown
5. Verify skills list appears in Step 2
6. Verify status badges (New/Same/VersionDiff/ContentDiff) display correctly
7. Select some skills, proceed to Step 3
8. Verify conflict resolution radios work
9. Click "确认迁移" — verify skills are copied
10. Verify global skill list refreshes

- [ ] **Step 3: Verify ProjectDetailView migration button**

1. Navigate to a project detail page
2. Verify "迁移 Skill" button appears in toolbar
3. Click it — dialog opens with project scope
4. Repeat verification steps as GlobalView

- [ ] **Step 4: Fix any issues found during smoke test**

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "feat(migration): complete local skill migration feature"
```
