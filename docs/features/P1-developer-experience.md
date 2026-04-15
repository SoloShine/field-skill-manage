# P1: Developer Experience

---

## F-03: Skill Creation Wizard

### 目标

通过分步交互向导引导用户创建符合规范的 SKILL.md 文件，降低 Skill 开发门槛。

### Rust 端

**新增命令** — `src-tauri/src/commands/skill.rs` 追加：

```rust
#[tauri::command]
pub fn create_skill_directory(
    state: &State<AppState>,
    skill_name: String,
    target: String,       // "global" 或项目路径
    content: String,      // SKILL.md 内容
) -> Result<(), String>
```

逻辑：解析 target → 拼接路径 → 创建目录 → 写入 SKILL.md

**可选辅助命令** — `src-tauri/src/commands/version.rs` 追加：

```rust
#[tauri::command]
pub fn validate_skill_name(name: String) -> Result<bool, String>  // ^[a-z0-9][a-z0-9-]*$

#[tauri::command]
pub fn validate_semver(version: String) -> Result<bool, String>
```

**`src-tauri/src/lib.rs`** — 注册新命令

### 前端

**新增 Composable** — `src/composables/useSkillValidator.ts`：

```typescript
export function useSkillValidator() {
  const errors = ref<string[]>([])

  function validateName(name: string): boolean  // 调用 Rust 命令或前端正则
  function validateVersion(version: string): boolean
  function validateFrontmatter(data: FrontmatterData): string[]

  return { errors, validateName, validateVersion, validateFrontmatter }
}
```

**新增组件** — `src/components/common/SkillCreationWizard.vue`：

使用 Naive UI 的 `NSteps` 组件，6 个步骤：

| 步骤 | 内容 | 关键字段 |
|------|------|----------|
| 1. 基本信息 | 名称、版本、作者、许可证、描述 | name, version, description |
| 2. 触发配置 | 触发描述、标签输入、文件模式、优先级 | trigger description, tags, file patterns |
| 3. 安全权限 | 权限复选框列表（从预定义集合选取） | permissions |
| 4. 正文内容 | 文本编辑器 + XML 标签模板插入 | body (context, instructions, examples) |
| 5. 预览校验 | 渲染生成的 SKILL.md，显示校验结果 | 预览 + 错误提示 |
| 6. 导出 | 选择目标（全局/项目），创建 Skill | target 选择 |

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/commands/skill.rs` | 追加 `create_skill_directory` |
| `src-tauri/src/commands/version.rs` | 追加 `validate_skill_name`、`validate_semver` |
| `src-tauri/src/lib.rs` | 注册新命令 |
| `src/composables/useSkillValidator.ts` | 新文件 |
| `src/components/common/SkillCreationWizard.vue` | 新文件 |
| `src/views/GlobalView.vue` | 工具栏添加"创建 Skill"按钮 |
| `src/views/ProjectDetailView.vue` | 同上 |
| `src/i18n/locales/zh-CN.json` | 添加 `wizard.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "wizard": {
    "title": "创建 Skill",
    "stepBasic": "基本信息",
    "stepTrigger": "触发配置",
    "stepSecurity": "安全权限",
    "stepBody": "正文内容",
    "stepPreview": "预览校验",
    "stepExport": "导出",
    "name": "Skill 名称",
    "version": "版本号",
    "author": "作者",
    "license": "许可证",
    "description": "描述",
    "triggerDesc": "触发描述",
    "tags": "标签",
    "filePatterns": "文件模式",
    "priority": "优先级",
    "permissions": "权限",
    "body": "正文",
    "preview": "预览",
    "create": "创建",
    "next": "下一步",
    "prev": "上一步",
    "nameRequired": "名称为必填项",
    "nameInvalid": "名称只能包含小写字母、数字和连字符",
    "versionInvalid": "版本号格式无效（需 semver 格式）",
    "createSuccess": "Skill 创建成功",
    "createFailed": "创建失败"
  }
}
```

### 实现步骤

1. 添加 `create_skill_directory`、`validate_skill_name`、`validate_semver` 命令
2. 在 `lib.rs` 注册
3. 创建 `useSkillValidator.ts`
4. 创建 `SkillCreationWizard.vue`（6 步向导）
5. 在 GlobalView 和 ProjectDetailView 工具栏添加入口
6. 添加 i18n 翻译

---

## F-04: Auto Sync & Scheduled Updates

### 目标

支持启动时自动同步和定时周期同步，减少手动操作。

### Rust 端

**修改模型** — `src-tauri/src/models/config.rs` 中 `AppConfig` 追加：

```rust
#[serde(default)]
pub sync_interval_minutes: u32,          // 0 = 禁用定时同步
#[serde(default)]
pub last_sync_time: Option<String>,      // ISO 8601，上次同步时间
```

**新增模型** — `src-tauri/src/models/config.rs` 追加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub auto_sync_enabled: bool,
    pub sync_interval_minutes: u32,
    pub last_sync_time: Option<String>,
    pub is_syncing: bool,
}
```

**新增命令** — `src-tauri/src/commands/git_sync.rs` 追加：

```rust
#[tauri::command]
pub fn get_sync_status(state: &State<AppState>) -> Result<SyncStatus, String>
```

**修改命令** — `src-tauri/src/commands/git_sync.rs` 中 `sync_remote_repo`：

- 同步完成后更新 `config.last_sync_time = Some(chrono::Utc::now().to_rfc3339())`
- 持久化配置

### 前端

**新增 Store** — `src/stores/sync.ts`：

```typescript
defineStore('sync', () => {
  const syncStatus = ref<SyncStatus | null>(null)
  const autoSyncTimer = ref<ReturnType<typeof setInterval> | null>(null)

  async function loadSyncStatus() { ... }
  function startAutoSync(intervalMinutes: number, onSync: () => Promise<void>) {
    autoSyncTimer.value = setInterval(onSync, intervalMinutes * 60 * 1000)
  }
  function stopAutoSync() { ... }

  return { syncStatus, autoSyncTimer, loadSyncStatus, startAutoSync, stopAutoSync }
})
```

**新增 Composable** — `src/composables/useAutoSync.ts`：

```typescript
export function useAutoSync() {
  const skillStore = useSkillStore()
  const syncStore = useSyncStore()
  const configStore = useConfigStore()

  function initAutoSync() {
    // 检查 auto_sync 配置和 interval
    // 启动时执行一次同步
    // 设置定时器
  }
  function destroyAutoSync() {
    syncStore.stopAutoSync()
  }

  return { initAutoSync, destroyAutoSync }
}
```

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/config.rs` | AppConfig 追加 `sync_interval_minutes`、`last_sync_time`；新增 `SyncStatus` |
| `src-tauri/src/commands/git_sync.rs` | 追加 `get_sync_status`；修改 `sync_remote_repo` 记录同步时间 |
| `src-tauri/src/lib.rs` | 注册 `get_sync_status` |
| `src/types/index.ts` | 追加 `SyncStatus`；`AppConfig` 追加字段 |
| `src/stores/sync.ts` | 新文件 |
| `src/composables/useAutoSync.ts` | 新文件 |
| `src/components/layout/AppLayout.vue` | onMounted 调用 `initAutoSync()`，onUnmounted 调用 `destroyAutoSync()` |
| `src/views/SettingsView.vue` | 仓库 Tab 添加同步间隔输入框 |
| `src/i18n/locales/zh-CN.json` | 添加 `sync.*` 和 `settings.syncInterval*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "settings": {
    "syncInterval": "同步间隔",
    "syncIntervalDesc": "设置为 0 表示禁用定时同步",
    "syncIntervalPlaceholder": "分钟数",
    "minutes": "分钟"
  },
  "sync": {
    "autoSyncStarted": "自动同步已启动",
    "autoSyncStopped": "自动同步已停止",
    "syncing": "正在同步...",
    "lastSyncTime": "上次同步",
    "neverSynced": "从未同步"
  }
}
```

### 实现步骤

1. 修改 `AppConfig` 添加新字段（Rust + TS）
2. 新增 `SyncStatus` 模型
3. 实现 `get_sync_status` 命令，修改 `sync_remote_repo` 记录时间
4. 在 `lib.rs` 注册
5. 创建 `stores/sync.ts`
6. 创建 `composables/useAutoSync.ts`
7. 在 `AppLayout.vue` 初始化自动同步
8. 在 `SettingsView.vue` 添加同步间隔设置
9. 添加 i18n 翻译

---

## F-05: Global Search (Full-text)

### 目标

跨所有本地和远端 Skill 搜索关键词，匹配范围包括名称、描述、标签和 SKILL.md 正文内容。

### Rust 端

**新增模型** — `src-tauri/src/models/search.rs`：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SearchMatchField {
    Name,
    Description,
    Tags,
    Content,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub skill_name: String,
    pub source: String,                    // "local" 或 repo_id
    pub source_type: String,               // "local" | "remote"
    pub match_fields: Vec<SearchMatchField>,
    pub match_snippets: Vec<String>,       // 匹配上下文片段
    pub score: f64,
}
```

**新增服务** — `src-tauri/src/services/search_service.rs`：

```rust
pub fn search_skills(
    query: &str,
    repos: &[RepoConfig],
    global_path: &str,
) -> Result<Vec<SearchResult>, String>
```

逻辑：
1. 小写化查询词
2. 遍历所有本地 Skill 目录：读取 SKILL.md 获取名称/描述/标签/正文
3. 遍历所有启用的远端仓库缓存：同上
4. 对每个 Skill 按字段匹配打分（名称权重最高，正文最低）
5. 按分数降序排列
6. 为每个匹配生成上下文片段（关键词前后各 50 字符）

**新增命令** — `src-tauri/src/commands/search.rs`：

```rust
#[tauri::command]
pub fn search_skills(
    state: &State<AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String>
```

### 前端

**新增 Store** — `src/stores/search.ts`：

```typescript
defineStore('search', () => {
  const results = ref<SearchResult[]>([])
  const query = ref('')
  const searching = ref(false)

  async function search(q: string) { ... }
  function clear() { results.value = []; query.value = '' }

  return { results, query, searching, search, clear }
})
```

**新增组件** — `src/components/common/GlobalSearchModal.vue`：

- 全屏模态框，`Ctrl+Shift+F` 触发
- 顶部搜索输入框（自动聚焦）
- 结果列表按匹配来源分组（本地 / 各远端仓库）
- 每条结果：Skill 名称、匹配字段标签、上下文片段（关键词高亮）
- 点击结果：关闭弹窗 → 导航到对应视图 → 打开预览

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/search.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod search;` |
| `src-tauri/src/services/search_service.rs` | 新文件 |
| `src-tauri/src/services/mod.rs` | 追加 `pub mod search_service;` |
| `src-tauri/src/commands/search.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod search;` |
| `src-tauri/src/lib.rs` | 注册 `search_skills` |
| `src/types/index.ts` | 追加 `SearchMatchField`、`SearchResult` |
| `src/stores/search.ts` | 新文件 |
| `src/components/common/GlobalSearchModal.vue` | 新文件 |
| `src/components/layout/AppLayout.vue` | 注册 `Ctrl+Shift+F` 快捷键，集成搜索弹窗 |
| `src/components/layout/Sidebar.vue` | 添加搜索图标按钮 |
| `src/i18n/locales/zh-CN.json` | 添加 `search.*` |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "search": {
    "placeholder": "搜索 Skill（名称、描述、标签、内容）...",
    "searching": "搜索中...",
    "noResults": "未找到匹配的 Skill",
    "resultCount": "找到 {count} 个结果",
    "matchName": "名称匹配",
    "matchDescription": "描述匹配",
    "matchTags": "标签匹配",
    "matchContent": "内容匹配",
    "globalShortcut": "Ctrl+Shift+F",
    "local": "本地",
    "remote": "远端"
  }
}
```

### 实现步骤

1. 创建 `models/search.rs`，注册到 `mod.rs`
2. 创建 `services/search_service.rs`，实现全文搜索逻辑
3. 创建 `commands/search.rs`，注册到 `mod.rs` 和 `lib.rs`
4. 在 `types/index.ts` 添加 TS 类型
5. 创建 `stores/search.ts`
6. 创建 `GlobalSearchModal.vue`
7. 在 `AppLayout.vue` 注册快捷键，在 `Sidebar.vue` 添加入口
8. 添加 i18n 翻译
