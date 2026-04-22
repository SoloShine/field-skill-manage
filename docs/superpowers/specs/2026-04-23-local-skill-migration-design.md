# Local Skill Migration Feature Design

**Date**: 2026-04-23
**Status**: Draft

## Goal

Support migrating skills between different AI agents' local skill directories, without requiring a remote repository. Users switching agents (e.g., from Cursor to Claude Code) can transfer their accumulated skills directly.

## Scope

- **Global migration**: Agent A's global dir (`~/.cursor/skills`) → Agent B's global dir (`~/.claude/skills`)
- **Project migration**: Agent A's project dir (`{project}/.cursor/skills`) → Agent B's project dir (`{project}/.claude/skills`)

## UI Design

### Entry Point

"迁移" button in the toolbar of both `GlobalView` and `ProjectDetailView`. Opens a 3-step wizard dialog (`MigrateDialog.vue`).

### Step 1 — Select Source Agent

- Dropdown listing all configured agents, excluding:
  - The currently active agent (can't migrate to self)
  - Agents whose skill directory path doesn't exist on disk
- Each option shows: Agent display name + path preview (e.g., `Cursor → ~/.cursor/skills`)
- On selection, calls `scan_agent_skills` and proceeds to Step 2

### Step 2 — Select Skills

- Table/list of scanned skills from source directory
- Only directories containing `SKILL.md` are recognized as valid skills (consistent with existing remote discovery)
- Columns: checkbox, name, version, description, status badge
- Status badges derived from `MigrateConflictStatus`:
  - **NewTarget** (green) — doesn't exist in target, safe to migrate
  - **SameContent** (gray) — source and target content identical (SHA256 match), skipped by default
  - **DifferentVersion** (yellow) — version numbers differ
  - **ContentDiffers** (orange) — content differs (aggregate SHA256 mismatch)
- Select all / deselect all toggle at top
- "SameContent" items default unchecked (no migration needed)
- "Next" button shows selected count

### Step 3 — Confirm Migration

- Two groups:
  - **New skills** list (direct migration)
  - **Conflicting skills** list (DifferentVersion / ContentDiffers), each with Skip/Override radio buttons and a "对比" link
- "对比" link opens existing `SkillDiffViewer` showing file-level diff between source and target skill (reuses `build_skill_diff` and diff viewer component)
- Summary bar: "Will migrate X, skip Y, override Z"
- "Confirm Migration" button → calls `migrate_skills` → shows result (success/failure per skill)
- On completion: auto-close dialog, refresh current view

## Data Models

### Rust (models/skill.rs)

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
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

### TypeScript (types/index.ts)

```typescript
type MigrateConflictStatus = 'NewTarget' | 'SameContent' | 'DifferentVersion' | 'ContentDiffers'

interface MigrateSkillEntry {
  name: string
  version: string
  description: string
  path: string
  conflictStatus: MigrateConflictStatus
}

interface ScanAgentSkillsResult {
  agentId: string
  agentDisplayName: string
  sourceDir: string
  skills: MigrateSkillEntry[]
}

type ConflictResolution = 'Skip' | 'Overwrite'

interface MigrateResult {
  migrated: string[]
  skipped: string[]
  failed: [string, string][]
}
```

## Backend Design

### New IPC Commands (commands/skill.rs)

**`scan_agent_skills(agent_id: String, scope: String, project_path: Option<String>)`**

- Resolves source dir from `AppConfig.agent_global_paths[agent_id]` (global) or `agent_project_patterns[agent_id]` with `{project}` substitution (project)
- Resolves target dir from active agent's corresponding path
- Scans source dir for subdirectories containing `SKILL.md`
- For each skill, compares against target using same strategy as existing remote/local comparison:
  - Aggregate SHA256 match → `SameContent`
  - Version mismatch (from frontmatter) → `DifferentVersion`
  - Aggregate SHA256 mismatch → `ContentDiffers`
  - Target doesn't exist → `NewTarget`
- Returns `ScanAgentSkillsResult`

**`migrate_skills(source_agent_id: String, skill_names: Vec<String>, scope: String, project_path: Option<String>, conflict_map: HashMap<String, ConflictResolution>)`**

- Resolves source and target dirs (same logic as scan)
- For each selected skill:
  - If target exists and not in conflict_map → skip
  - If target exists and `ConflictResolution::Skip` → skip
  - If target exists and `ConflictResolution::Overwrite` → `remove_dir_all` then copy
  - If target doesn't exist → copy directly
- Copies via existing `copy_dir_recursive`
- Records operation history via existing `history_service`
- Returns `MigrateResult`

**`get_migrate_skill_diff(source_agent_id: String, skill_name: String, scope: String, project_path: Option<String>)`**

- Resolves source and target skill directories
- Calls existing `build_skill_diff(source_dir, target_dir)` — zero modification to diff logic
- Returns `SkillDiff`

### Service Layer (services/skill_service.rs)

No new service file. Migration functions live in `skill_service.rs`, reusing:
- `list_installed_skills` — directory scanning
- `parse_skill_frontmatter` — SKILL.md parsing
- `build_local_skill_meta` — metadata building
- `hash_service::aggregate_sha256` — content comparison
- `build_skill_diff` — file-level diff
- `copy_dir_recursive` — directory copying
- `history_service` — operation recording

## Conflict Detection Strategy

Uses the same 4-level fallback as existing remote/local comparison, comparing source agent's skill against target agent's skill:

1. Aggregate SHA256 checksum match → `SameContent`
2. Version number (from frontmatter) differs → `DifferentVersion`
3. No metadata → aggregate SHA256 comparison
4. Fallback → `ContentDiffers`

## Frontend Changes

### New Component

`src/components/common/MigrateDialog.vue` — 3-step wizard dialog with step indicator at top.

### Store Changes (stores/skill.ts)

New state:
- `scanResult: ScanAgentSkillsResult | null`
- `migrateDialogVisible: boolean`
- `migrateScope: 'global' | 'project'`
- `migrateProjectPath: string | null`

New methods:
- `scanAgentSkills(agentId, scope, projectPath?)` — invoke `scan_agent_skills`
- `migrateSkills(sourceAgentId, skillNames, conflictMap, scope, projectPath?)` — invoke `migrate_skills`
- `openMigrateDialog(scope, projectPath?)` / `closeMigrateDialog()`

### View Changes

- `GlobalView.vue` — add "迁移" button to toolbar, wire up dialog open
- `ProjectDetailView.vue` — add "迁移" button to toolbar, wire up dialog open with project scope

### i18n

Add `migration.*` keys for both `zh-CN` and `en-US` locales.

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/models/skill.rs` | Add MigrateConflictStatus, MigrateSkillEntry, ScanAgentSkillsResult, ConflictResolution, MigrateResult |
| `src-tauri/src/models/mod.rs` | No change (skill module already exported) |
| `src-tauri/src/services/skill_service.rs` | Add scan_agent_skills, migrate_skills_to functions |
| `src-tauri/src/commands/skill.rs` | Add scan_agent_skills, migrate_skills, get_migrate_skill_diff commands |
| `src-tauri/src/lib.rs` | Register 3 new commands |
| `src/types/index.ts` | Add migration types |
| `src/components/common/MigrateDialog.vue` | New file — wizard dialog |
| `src/stores/skill.ts` | Add migration state and methods |
| `src/views/GlobalView.vue` | Add migration button to toolbar |
| `src/views/ProjectDetailView.vue` | Add migration button to toolbar |
| `src/locales/zh-CN.ts` | Add migration translations |
| `src/locales/en-US.ts` | Add migration translations |
