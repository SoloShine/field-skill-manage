# P3: Polish

---

## F-09: Keyboard Shortcuts

### 目标

为高频操作提供全局快捷键，提升效率用户操作速度。

### 复杂度

S — 纯前端，无需 Rust 改动

### 快捷键规划

| 快捷键 | 操作 |
|--------|------|
| `Ctrl+K` | 聚焦搜索框（已有，需统一管理） |
| `Ctrl+Shift+F` | 打开全局搜索弹窗（F-05） |
| `Ctrl+S` | 同步远端仓库 |
| `Ctrl+1` | 导航到全局 Skill |
| `Ctrl+2` | 导航到项目列表 |
| `Ctrl+3` | 导航到设置 |
| `Ctrl+4` | 导航到开发指南 |
| `Ctrl+Shift+A` | 批量更新所有 |
| `Escape` | 关闭当前弹窗/面板 |
| `Ctrl+/` | 显示快捷键帮助 |

### 新增文件

**`src/composables/useKeyboardShortcuts.ts`**：

```typescript
export function useKeyboardShortcuts() {
  const router = useRouter()
  const skillStore = useSkillStore()

  function register() {
    document.addEventListener('keydown', handleKeyDown)
  }
  function unregister() {
    document.removeEventListener('keydown', handleKeyDown)
  }
  function handleKeyDown(e: KeyboardEvent) {
    // Ctrl+K → 聚焦搜索（emit 自定义事件）
    // Ctrl+Shift+F → 打开全局搜索
    // Ctrl+S → e.preventDefault() + syncRemote
    // Ctrl+1/2/3/4 → router.push
    // Ctrl+Shift+A → batchUpdateAll
    // Ctrl+/ → toggle help
  }

  onMounted(register)
  onUnmounted(unregister)

  return { register, unregister }
}
```

**`src/components/common/ShortcutsHelp.vue`**：

- 浮层/模态框
- 快捷键列表，格式：`快捷键` → `操作描述`
- `Ctrl+/` 或 `?` 触发

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src/composables/useKeyboardShortcuts.ts` | 新文件 |
| `src/components/common/ShortcutsHelp.vue` | 新文件 |
| `src/components/layout/AppLayout.vue` | 调用 `useKeyboardShortcuts()`，集成 ShortcutsHelp |
| `src/views/GlobalView.vue` | 移除内联 `handleSearchShortcut`，委托给 composable |
| `src/views/ProjectDetailView.vue` | 同上 |

### i18n 翻译键

```json
{
  "shortcuts": {
    "title": "快捷键",
    "search": "搜索 Skill",
    "globalSearch": "全局搜索",
    "sync": "同步远端",
    "updateAll": "更新全部",
    "navigate": "导航",
    "closeModal": "关闭弹窗",
    "showHelp": "显示快捷键帮助"
  }
}
```

### 依赖

F-05（Global Search）— `Ctrl+Shift+F` 需要全局搜索弹窗已存在

### 实现步骤

1. 创建 `useKeyboardShortcuts.ts`
2. 创建 `ShortcutsHelp.vue`
3. 在 `AppLayout.vue` 集成
4. 移除 GlobalView/ProjectDetailView 中的内联快捷键处理
5. 添加 i18n 翻译

---

## F-10: Drag & Drop Management

### 目标

支持拖拽 Skill 在全局和项目之间移动。

### 复杂度

M — 纯前端交互，使用 HTML5 Drag and Drop API

### 新增文件

**`src/components/common/DraggableSkillCard.vue`**：

- 包装层，为 SkillCard 添加 `draggable="true"` 属性
- `@dragstart` 设置 `dataTransfer` 数据（Skill 名称、来源 target）
- 接收 drop 区域显示高亮边框

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src/components/common/DraggableSkillCard.vue` | 新文件 |
| `src/components/common/SkillCompareTable.vue` | 行添加拖拽手柄；表格外层添加 drop 区域；emit `move` 事件 |
| `src/views/GlobalView.vue` | 处理 `move` 事件：从源 uninstall → 安装到目标 |
| `src/views/ProjectDetailView.vue` | 同上 |

### i18n 翻译键

```json
{
  "dragDrop": {
    "moveConfirm": "确定将「{name}」从 {source} 移动到 {target}？",
    "moving": "正在移动...",
    "moveSuccess": "移动成功",
    "moveFailed": "移动失败"
  }
}
```

### 实现步骤

1. 创建 `DraggableSkillCard.vue`
2. 在 `SkillCompareTable.vue` 添加拖拽支持
3. 在视图中处理移动逻辑
4. 添加 i18n 翻译

---

## F-11: Skill Integrity Monitor

### 目标

扫描已安装 Skill 的文件完整性，对比安装时的预期哈希，检测意外修改或损坏。

### Rust 端

**新增模型** — `src-tauri/src/models/skill.rs` 追加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum IntegrityStatus {
    Intact,
    Modified,
    MissingFiles,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityResult {
    pub skill_name: String,
    pub status: IntegrityStatus,
    pub expected_hash: Option<String>,
    pub current_hash: Option<String>,
    pub mismatched_files: Vec<FileEntry>,
}
```

**新增命令** — `src-tauri/src/commands/version.rs` 追加：

```rust
#[tauri::command]
pub fn scan_skill_integrity(
    state: &State<AppState>,
    target: String,
) -> Result<Vec<IntegrityResult>, String>
```

**修改安装流程** — `src-tauri/src/commands/skill.rs`：

安装完成后在 Skill 目录写入 `.spm-meta.json`：
```json
{ "installed_hash": "sha256:...", "installed_at": "...", "source_repo_id": "..." }
```

### 前端

**Store 扩展** — `src/stores/skill.ts` 追加：

```typescript
const integrityResults = ref<IntegrityResult[]>([])

async function scanIntegrity(target: string) {
  integrityResults.value = await invoke<IntegrityResult[]>('scan_skill_integrity', { target })
}
```

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/skill.rs` | 追加 `IntegrityStatus`、`IntegrityResult` |
| `src-tauri/src/commands/version.rs` | 追加 `scan_skill_integrity` |
| `src-tauri/src/commands/skill.rs` | 安装时写入 `.spm-meta.json` |
| `src-tauri/src/lib.rs` | 注册新命令 |
| `src/types/index.ts` | 追加 `IntegrityStatus`、`IntegrityResult` |
| `src/stores/skill.ts` | 追加 `integrityResults` 和 `scanIntegrity` |
| `src/components/common/SkillCompareTable.vue` | 可选：添加完整性状态列 |
| `src/views/GlobalView.vue` | 添加"扫描完整性"按钮 |
| `src/i18n/locales/zh-CN.json` | 添加 `integrity.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "integrity": {
    "title": "完整性扫描",
    "scan": "扫描完整性",
    "scanning": "正在扫描...",
    "intact": "完好",
    "modified": "已修改",
    "missingFiles": "文件缺失",
    "scanComplete": "扫描完成，发现 {count} 个异常",
    "issues": "异常"
  }
}
```

### 实现步骤

1. 在 `models/skill.rs` 添加 `IntegrityStatus`、`IntegrityResult`
2. 在 `types/index.ts` 添加对应类型
3. 修改安装命令写入元数据文件
4. 实现 `scan_skill_integrity` 命令
5. 在 Store 中添加扫描 action
6. 在 GlobalView 添加扫描按钮和结果展示
7. 添加 i18n 翻译

---

## F-12: Skill Statistics Dashboard

### 目标

统计仪表盘，展示安装趋势、最常用 Skill、分类分布等。

### 复杂度

L — 需要新页面 + 图表 + 依赖 F-02 历史数据

### 依赖

F-02（Operation History）— 提供历史操作数据

### Rust 端

**新增模型** — `src-tauri/src/models/stats.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimePoint {
    pub date: String,       // YYYY-MM-DD
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillRank {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepoStats {
    pub repo_id: String,
    pub repo_name: String,
    pub skill_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub total_installed: u32,
    pub total_remote: u32,
    pub total_operations: u32,
    pub installs_over_time: Vec<TimePoint>,  // 最近 30 天
    pub top_skills: Vec<SkillRank>,          // Top 10
    pub by_tag: HashMap<String, u32>,
    pub by_repo: Vec<RepoStats>,
}
```

**新增命令** — `src-tauri/src/commands/stats.rs`：

```rust
#[tauri::command]
pub fn get_skill_stats(state: &State<AppState>) -> Result<SkillStats, String>
```

从 `history.json` 和当前 Skill 状态聚合统计数据。

### 前端

**新增视图/组件** — `src/components/common/StatsDashboard.vue`：

使用 CSS-only 柱状图（避免引入 chart.js 等重量级依赖）：
- 安装趋势图（30 天折线）
- Top 10 Skill 横向柱状图
- 标签云（按数量缩放）
- 仓库分布饼图

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/stats.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod stats;` |
| `src-tauri/src/commands/stats.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod stats;` |
| `src-tauri/src/lib.rs` | 注册命令 |
| `src/types/index.ts` | 追加统计类型 |
| `src/components/common/StatsDashboard.vue` | 新文件 |
| `src/views/SettingsView.vue` | 在设置中嵌入统计面板（或独立路由） |
| `src/router/index.ts` | 可选：添加 `/dashboard` 路由 |
| `src/components/layout/Sidebar.vue` | 可选：添加"统计"菜单项 |
| `src/i18n/locales/zh-CN.json` | 添加 `dashboard.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "dashboard": {
    "title": "统计面板",
    "installed": "已安装 Skill",
    "remote": "远端 Skill",
    "totalOps": "总操作次数",
    "trends": "安装趋势（30天）",
    "topSkills": "最常用 Skill",
    "byTag": "标签分布",
    "byRepo": "仓库分布",
    "noData": "暂无数据"
  }
}
```

### 实现步骤

1. 创建 `models/stats.rs`，注册
2. 创建 `commands/stats.rs`，实现统计聚合
3. 注册命令
4. 添加 TS 类型
5. 创建 `StatsDashboard.vue`
6. 在设置页或独立路由中嵌入
7. 添加 i18n 翻译

---

## F-13: Skill Tags & Favorites

### 目标

自定义标签和收藏系统，快速定位常用 Skill。

### 复杂度

M — 需要持久化存储和 UI 交互

### Rust 端

**新增模型** — `src-tauri/src/models/favorites.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserSkillTags {
    pub skill_name: String,
    pub custom_tags: Vec<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserTagsStore {
    pub tags: Vec<UserSkillTags>,
}
```

**新增服务** — `src-tauri/src/services/tags_service.rs`：

持久化到 `~/.spm/user_tags.json`

```rust
pub fn load_tags(home: &str) -> UserTagsStore
pub fn save_tags(home: &str, store: &UserTagsStore) -> Result<()>
pub fn set_skill_tags(home: &str, skill_name: &str, tags: Vec<String>, favorite: bool) -> Result<()>
pub fn get_skill_tags(home: &str, skill_name: &str) -> Option<UserSkillTags>
pub fn get_favorites(home: &str) -> Vec<String>
```

**新增命令** — `src-tauri/src/commands/tags.rs`：

```rust
#[tauri::command]
pub fn get_all_user_tags(state: &State<AppState>) -> Result<Vec<UserSkillTags>, String>

#[tauri::command]
pub fn set_skill_tags(state: &State<AppState>, skillName: String, tags: Vec<String>, favorite: bool) -> Result<(), String>

#[tauri::command]
pub fn get_favorite_skills(state: &State<AppState>) -> Result<Vec<String>, String>
```

### 前端

**修改现有组件**：

- `SkillCompareTable.vue`：添加收藏星标列（点击切换）
- `SkillPreviewModal.vue`：元数据区域添加标签编辑（标签输入框 + 已有标签展示）
- `GlobalView.vue` / `ProjectDetailView.vue`：筛选器添加"仅收藏"选项

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/favorites.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod favorites;` |
| `src-tauri/src/services/tags_service.rs` | 新文件 |
| `src-tauri/src/services/mod.rs` | 追加 `pub mod tags_service;` |
| `src-tauri/src/commands/tags.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod tags;` |
| `src-tauri/src/lib.rs` | 注册 3 个命令 |
| `src/types/index.ts` | 追加 `UserSkillTags` |
| `src/components/common/SkillCompareTable.vue` | 添加收藏列 |
| `src/components/common/SkillPreviewModal.vue` | 添加标签编辑 |
| `src/views/GlobalView.vue` | 添加"仅收藏"筛选 |
| `src/views/ProjectDetailView.vue` | 同上 |
| `src/i18n/locales/zh-CN.json` | 添加 `tags.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "tags": {
    "favorite": "收藏",
    "unfavorite": "取消收藏",
    "addTag": "添加标签",
    "removeTag": "移除标签",
    "customTags": "自定义标签",
    "favorites": "收藏",
    "showFavorites": "仅显示收藏"
  }
}
```

### 实现步骤

1. 创建 `models/favorites.rs`，注册
2. 创建 `services/tags_service.rs`
3. 创建 `commands/tags.rs`，注册
4. 添加 TS 类型
5. 在 `SkillCompareTable.vue` 添加收藏列
6. 在 `SkillPreviewModal.vue` 添加标签编辑
7. 在视图中添加收藏筛选
8. 添加 i18n 翻译

---

## F-14: Changelog Display

### 目标

展示远端 Skill 目录的 Git 提交历史，让用户了解更新了什么。

### 复杂度

S — 利用已有 Git 基础设施

### Rust 端

**新增模型** — `src-tauri/src/models/changelog.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogEntry {
    pub hash: String,          // 提交哈希（短）
    pub message: String,       // 提交消息
    pub author: String,        // 作者
    pub date: String,          // ISO 8601
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillChangelog {
    pub skill_name: String,
    pub entries: Vec<ChangelogEntry>,
}
```

**新增服务函数** — `src-tauri/src/services/git_service.rs` 追加：

```rust
pub fn get_dir_log(
    repo_path: &Path,
    dir_path: &str,          // Skill 相对路径
    limit: u32,              // 最多返回条数（默认 20）
) -> Result<Vec<ChangelogEntry>, String>
```

执行 `git log --oneline -n {limit} -- {dir_path}` 并解析输出。

**新增命令** — `src-tauri/src/commands/changelog.rs`：

```rust
#[tauri::command]
pub fn get_skill_changelog(
    state: &State<AppState>,
    skill_name: String,
    repo_id: Option<String>,
) -> Result<SkillChangelog, String>
```

### 前端

**新增组件** — `src/components/common/ChangelogPanel.vue`：

- 时间线视图
- 每项：日期、作者、提交消息、短哈希
- 可从 SkillPreviewModal 或 CompareTable 触发

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/changelog.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod changelog;` |
| `src-tauri/src/services/git_service.rs` | 追加 `get_dir_log()` |
| `src-tauri/src/commands/changelog.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod changelog;` |
| `src-tauri/src/lib.rs` | 注册命令 |
| `src/types/index.ts` | 追加 `ChangelogEntry`、`SkillChangelog` |
| `src/components/common/ChangelogPanel.vue` | 新文件 |
| `src/components/common/SkillCompareTable.vue` | Outdated 行添加"变更日志"按钮 |
| `src/i18n/locales/zh-CN.json` | 添加 `changelog.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "changelog": {
    "title": "变更日志",
    "noChanges": "暂无变更记录",
    "loadFailed": "加载变更日志失败",
    "commits": "提交记录",
    "viewDiff": "查看差异"
  }
}
```

### 实现步骤

1. 创建 `models/changelog.rs`，注册
2. 在 `services/git_service.rs` 添加 `get_dir_log()`
3. 创建 `commands/changelog.rs`，注册
4. 添加 TS 类型
5. 创建 `ChangelogPanel.vue`
6. 在 CompareTable 添加按钮触发
7. 添加 i18n 翻译
