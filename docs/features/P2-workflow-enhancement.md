# P2: Workflow Enhancement

---

## F-06: Skill Presets/Profiles

### 目标

保存和恢复命名 Skill 集合（预设方案），用于在不同工作场景间快速切换。

### Rust 端

**新增模型** — `src-tauri/src/models/preset.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillPreset {
    pub id: String,                      // UUID
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,             // Skill 名称列表
    pub target: String,                  // "global" 或项目路径
    pub agent_id: String,                // 关联的 Agent
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PresetStore {
    pub presets: Vec<SkillPreset>,
}
```

**新增服务** — `src-tauri/src/services/preset_service.rs`：

持久化到 `~/.spm/presets.json`

```rust
pub fn load_presets(home: &str) -> PresetStore
pub fn save_presets(home: &str, store: &PresetStore) -> Result<()>
pub fn create_preset(home: &str, name: &str, description: &str, skills: Vec<String>, target: &str, agent_id: &str) -> Result<SkillPreset>
pub fn delete_preset(home: &str, id: &str) -> Result<()>
pub fn update_preset(home: &str, preset: &SkillPreset) -> Result<()>
```

**新增命令** — `src-tauri/src/commands/preset.rs`：

```rust
#[tauri::command]
pub fn list_presets(state: &State<AppState>) -> Result<Vec<SkillPreset>, String>

#[tauri::command]
pub fn save_preset(state: &State<AppState>, preset: SkillPreset) -> Result<(), String>

#[tauri::command]
pub fn delete_preset(state: &State<AppState>, id: String) -> Result<(), String>

#[tauri::command]
pub fn apply_preset(state: &State<AppState>, id: String) -> Result<(), String>
// 安装预设中的所有 Skill

#[tauri::command]
pub fn create_preset_from_current(
    state: &State<AppState>,
    name: String,
    description: String,
    target: String,
) -> Result<SkillPreset, String>
// 快照当前已安装的 Skill 列表，生成预设
```

### 前端

**新增 Store** — `src/stores/preset.ts`：

```typescript
defineStore('preset', () => {
  const presets = ref<SkillPreset[]>([])
  const loading = ref(false)
  const applying = ref(false)

  async function loadPresets() { ... }
  async function savePreset(preset: SkillPreset) { ... }
  async function deletePreset(id: string) { ... }
  async function applyPreset(id: string) { ... }
  async function createFromCurrent(name: string, desc: string, target: string) { ... }

  return { presets, loading, applying, loadPresets, savePreset, deletePreset, applyPreset, createFromCurrent }
})
```

**新增组件** — `src/components/common/PresetManager.vue`：

- 模态框或抽屉面板
- 预设列表：名称、描述、Skill 数量、创建时间
- 操作按钮：应用（安装全部 Skill）、编辑、删除
- "从当前创建" 按钮：捕获已安装 Skill 名称，输入名称和描述
- 应用时显示进度（N/M 已安装）

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/preset.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod preset;` |
| `src-tauri/src/services/preset_service.rs` | 新文件 |
| `src-tauri/src/services/mod.rs` | 追加 `pub mod preset_service;` |
| `src-tauri/src/commands/preset.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod preset;` |
| `src-tauri/src/lib.rs` | 注册 5 个命令 |
| `src/types/index.ts` | 追加 `SkillPreset` |
| `src/stores/preset.ts` | 新文件 |
| `src/components/common/PresetManager.vue` | 新文件 |
| `src/views/GlobalView.vue` | 工具栏添加"预设"按钮 |
| `src/views/ProjectDetailView.vue` | 同上 |
| `src/i18n/locales/zh-CN.json` | 添加 `preset.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "preset": {
    "title": "Skill 预设方案",
    "create": "新建预设",
    "createFromCurrent": "从当前安装创建",
    "apply": "应用预设",
    "delete": "删除",
    "confirmDelete": "确定删除预设「{name}」？",
    "applySuccess": "预设「{name}」应用成功",
    "applyFailed": "应用预设失败",
    "createSuccess": "预设创建成功",
    "name": "名称",
    "description": "描述",
    "skillCount": "包含 {count} 个 Skill",
    "empty": "暂无预设方案",
    "applying": "正在应用预设...",
    "noInstalledSkills": "当前没有已安装的 Skill"
  }
}
```

### 实现步骤

1. 创建 `models/preset.rs`，注册到 `mod.rs`
2. 创建 `services/preset_service.rs`，实现 CRUD + 持久化
3. 创建 `commands/preset.rs`，注册到 `mod.rs` 和 `lib.rs`
4. 在 `types/index.ts` 添加 `SkillPreset`
5. 创建 `stores/preset.ts`
6. 创建 `PresetManager.vue`
7. 在 GlobalView 和 ProjectDetailView 添加入口
8. 添加 i18n 翻译

---

## F-07: Skill Publishing

### 目标

将本地开发的 Skill 推送到配置的 Git 仓库，实现"开发 → 测试 → 发布"闭环。

### Rust 端

**新增服务** — `src-tauri/src/services/publish_service.rs`：

```rust
pub fn publish_skill_to_repo(
    skill_dir: &Path,           // 本地 Skill 目录
    repo: &RepoConfig,          // 目标仓库配置
    skill_name: &str,
    commit_message: &str,
) -> Result<(), String>
```

逻辑：
1. 确保仓库已克隆到 `cache_path`
2. 将 Skill 目录内容复制到仓库的 skills/ 子目录
3. 如果仓库有 `skills.json`，更新清单
4. 执行 `git add` → `git commit -m` → `git push`
5. 认证使用系统 SSH 密钥或 Git credential helper

**新增命令** — `src-tauri/src/commands/publish.rs`：

```rust
#[tauri::command]
pub fn publish_skill(
    state: &State<AppState>,
    skill_name: String,
    source_target: String,     // "global" 或项目路径
    repo_id: String,
) -> Result<(), String>

#[tauri::command]
pub fn list_publishable_repos(
    state: &State<AppState>,
) -> Result<Vec<RepoConfig>, String>
// 返回所有配置的仓库（用户自行判断写入权限）
```

### 前端

**新增组件** — `src/components/common/PublishDialog.vue`：

- 对话框，包含：
  - Skill 名称（只读）
  - 来源（全局/项目路径）
  - 目标仓库选择（下拉框）
  - 覆盖警告（如果远端已存在同名 Skill）
  - 确认/取消按钮
- 发布中显示 loading 状态

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/services/publish_service.rs` | 新文件 |
| `src-tauri/src/services/mod.rs` | 追加 `pub mod publish_service;` |
| `src-tauri/src/commands/publish.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod publish;` |
| `src-tauri/src/lib.rs` | 注册 2 个命令 |
| `src/components/common/PublishDialog.vue` | 新文件 |
| `src/components/common/SkillCompareTable.vue` | 为 `LocalOnly` 行添加"发布"按钮 |
| `src/views/GlobalView.vue` | 处理 publish 事件 |
| `src/views/ProjectDetailView.vue` | 同上 |
| `src/i18n/locales/zh-CN.json` | 添加 `publish.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "publish": {
    "title": "发布 Skill",
    "selectRepo": "选择目标仓库",
    "confirm": "确认发布",
    "success": "发布成功",
    "failed": "发布失败",
    "noRepos": "没有可用的仓库",
    "overwriteWarning": "远端已存在同名 Skill，发布将覆盖",
    "pushing": "正在推送..."
  }
}
```

### 实现步骤

1. 创建 `services/publish_service.rs`，实现 Git 推送逻辑
2. 创建 `commands/publish.rs`，注册
3. 创建 `PublishDialog.vue`
4. 在 `SkillCompareTable.vue` 添加发布按钮
5. 在视图中接入
6. 添加 i18n 翻译

---

## F-08: Skill Dependency Management

### 目标

解析 SKILL.md 中的 `dependencies` 字段，解析传递依赖链，自动安装所有依赖。

### Rust 端

**新增模型** — `src-tauri/src/models/dependency.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDependency {
    pub name: String,
    pub version: Option<String>,        // 版本要求（可选）
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyNode {
    pub skill_name: String,
    pub dependencies: Vec<SkillDependency>,
    pub available: bool,                 // 远端是否存在
    pub installed: bool,                 // 本地是否已安装
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyGraph {
    pub root: String,                    // 根 Skill 名称
    pub nodes: Vec<DependencyNode>,
    pub install_order: Vec<String>,      // 拓扑排序后的安装顺序
    pub conflicts: Vec<String>,          // 冲突列表
    pub missing: Vec<String>,            // 找不到的依赖
}
```

**修改 SkillFrontmatter** — `src-tauri/src/models/skill.rs`：

追加字段：
```rust
#[serde(default)]
pub dependencies: Option<Vec<SkillDependency>>,
```

**修改 frontmatter 解析** — `src-tauri/src/services/skill_service.rs`：

在 `parse_simple_yaml_frontmatter` 中增加对 `dependencies:` 块的解析。

**新增命令** — `src-tauri/src/commands/dependency.rs`：

```rust
#[tauri::command]
pub fn get_skill_dependencies(
    state: &State<AppState>,
    skill_name: String,
    repo_id: Option<String>,
) -> Result<DependencyGraph, String>
// 解析依赖树，返回拓扑排序的安装顺序

#[tauri::command]
pub fn install_with_dependencies(
    state: &State<AppState>,
    skill_name: String,
    target: String,
    repo_id: Option<String>,
) -> Result<Vec<String>, String>
// 按拓扑序安装所有依赖，返回实际安装的 Skill 列表
```

### 前端

**新增组件** — `src/components/common/DependencyView.vue`：

- 树形展示依赖关系
- 每个节点显示：Skill 名称、版本要求、状态（已安装/待安装/缺失）
- 底部显示安装顺序和冲突提示

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/dependency.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod dependency;` |
| `src-tauri/src/models/skill.rs` | `SkillFrontmatter` 追加 `dependencies` 字段 |
| `src-tauri/src/services/skill_service.rs` | frontmatter 解析增加 dependencies |
| `src-tauri/src/commands/dependency.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod dependency;` |
| `src-tauri/src/lib.rs` | 注册 2 个命令 |
| `src/types/index.ts` | 追加 `SkillDependency`、`DependencyNode`、`DependencyGraph`；`SkillMeta` 追加 `dependencies` |
| `src/components/common/DependencyView.vue` | 新文件 |
| `src/components/common/SkillPreviewModal.vue` | 添加"依赖"标签页 |
| `src/components/common/SkillCompareTable.vue` | 有依赖的 Skill 添加"含依赖安装"选项 |
| `src/i18n/locales/zh-CN.json` | 添加 `dependency.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "dependency": {
    "title": "依赖关系",
    "none": "无依赖",
    "installing": "正在安装依赖...",
    "installOrder": "安装顺序",
    "conflict": "依赖冲突",
    "circularError": "检测到循环依赖",
    "notFound": "依赖未找到",
    "installWithDeps": "含依赖安装",
    "installedCount": "已安装 {count} 个依赖"
  }
}
```

### 实现步骤

1. 创建 `models/dependency.rs`，注册到 `mod.rs`
2. 在 `models/skill.rs` 的 `SkillFrontmatter` 添加 `dependencies` 字段
3. 修改 frontmatter 解析器支持 dependencies
4. 创建 `commands/dependency.rs`，实现依赖解析和拓扑排序
5. 注册命令
6. 在 `types/index.ts` 添加类型
7. 创建 `DependencyView.vue`
8. 在 `SkillPreviewModal.vue` 添加依赖标签页
9. 在 `SkillCompareTable.vue` 添加含依赖安装选项
10. 添加 i18n 翻译
