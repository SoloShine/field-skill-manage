# P0: Core Experience

---

## F-01: Skill Diff Viewer

### 目标

对本地已安装的 Skill 与远端最新版本进行逐文件差异对比。展示每个文件的 Added / Removed / Modified / Unchanged 状态及哈希值，帮助用户判断是否需要更新。

### 新增 Rust 类型

**`src-tauri/src/models/skill.rs`** — 追加：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FileDiffStatus {
    Unchanged,
    Added,
    Removed,
    Modified,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    pub path: String,
    pub local_hash: Option<String>,
    pub remote_hash: Option<String>,
    pub local_size: Option<u64>,
    pub remote_size: Option<u64>,
    pub status: FileDiffStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillDiff {
    pub skill_name: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub files: Vec<FileDiff>,
    pub added_count: u32,
    pub removed_count: u32,
    pub modified_count: u32,
    pub unchanged_count: u32,
}
```

### 新增 Rust 服务函数

**`src-tauri/src/services/skill_service.rs`** — 追加：

```rust
pub fn build_skill_diff(
    local_dir: &Path,
    remote_dir: &Path,
) -> Result<SkillDiff, String>
```

逻辑：
1. 调用 `hash_service::list_file_hashes()` 分别获取本地和远端文件列表
2. 以 `HashMap<path, FileEntry>` 建立两边索引
3. 合并所有路径，逐文件比较：
   - 两边都有且 hash 相同 → `Unchanged`
   - 两边都有且 hash 不同 → `Modified`
   - 仅本地有 → `Removed`
   - 仅远端有 → `Added`
4. 统计各类型数量，组装 `SkillDiff`

### 新增 Rust 命令

**`src-tauri/src/commands/version.rs`** — 追加：

```rust
#[tauri::command]
pub fn get_skill_diff(
    state: &State<AppState>,
    skill_name: String,
    target: String,
) -> Result<SkillDiff, String>
```

逻辑：
1. `state.config.lock()` 获取配置
2. 解析 target 为本地目录（global 或 project）
3. 遍历 repos 找到远端 skill 路径
4. 调用 `skill_service::build_skill_diff()`

**`src-tauri/src/lib.rs`** — 注册 `version::get_skill_diff`

### 新增前端类型

**`src/types/index.ts`** — 追加：

```typescript
export type FileDiffStatus = 'Unchanged' | 'Added' | 'Removed' | 'Modified'

export interface FileDiff {
  path: string
  localHash?: string
  remoteHash?: string
  localSize?: number
  remoteSize?: number
  status: FileDiffStatus
}

export interface SkillDiff {
  skillName: string
  localVersion?: string
  remoteVersion?: string
  files: FileDiff[]
  addedCount: number
  removedCount: number
  modifiedCount: number
  unchangedCount: number
}
```

### 新增 Store 状态

**`src/stores/skill.ts`** — 追加：

```typescript
const skillDiff = ref<SkillDiff | null>(null)

async function loadSkillDiff(name: string, target: string) {
  skillDiff.value = await invoke<SkillDiff>('get_skill_diff', {
    skillName: name, target,
  })
}
```

### 新增组件

**`src/components/common/SkillDiffViewer.vue`**

- Props: `{ diff: SkillDiff }`
- Emits: `{ close: [], openFile: [filePath: string] }`
- UI:
  - 顶部汇总条：Added (绿) / Removed (红) / Modified (橙) / Unchanged (灰) 数量徽章
  - 数据表格：文件路径、状态图标（彩色圆点）、本地哈希（截断）、远端哈希（截断）、大小变化
  - 点击文件行可调用已有 `read_skill_file` 查看内容

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/skill.rs` | 追加 `FileDiffStatus`、`FileDiff`、`SkillDiff` |
| `src-tauri/src/services/skill_service.rs` | 追加 `build_skill_diff()` |
| `src-tauri/src/commands/version.rs` | 追加 `get_skill_diff` 命令 |
| `src-tauri/src/lib.rs` | 注册 `version::get_skill_diff` |
| `src/types/index.ts` | 追加对应 TS 类型 |
| `src/stores/skill.ts` | 追加 `skillDiff` 状态和 `loadSkillDiff` action |
| `src/components/common/SkillCompareTable.vue` | 为 `Outdated` 行添加 "Diff" 按钮，emit `diff` 事件 |
| `src/views/GlobalView.vue` | 处理 `diff` 事件，弹出 SkillDiffViewer 模态框 |
| `src/views/ProjectDetailView.vue` | 同上 |
| `src/i18n/locales/zh-CN.json` | 添加 `diff.*` 翻译键 |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "diff": {
    "title": "版本差异对比",
    "file": "文件",
    "status": "状态",
    "localHash": "本地哈希",
    "remoteHash": "远端哈希",
    "size": "大小",
    "added": "新增",
    "removed": "删除",
    "modified": "修改",
    "unchanged": "未变",
    "noDiff": "无差异",
    "summary": "汇总",
    "close": "关闭"
  }
}
```

### 实现步骤

1. 在 `models/skill.rs` 添加 `FileDiffStatus`、`FileDiff`、`SkillDiff`
2. 在 `types/index.ts` 添加对应 TS 类型
3. 在 `services/skill_service.rs` 实现 `build_skill_diff()`
4. 在 `commands/version.rs` 添加 `get_skill_diff` 命令
5. 在 `lib.rs` 注册命令
6. 在 `stores/skill.ts` 添加状态和 action
7. 创建 `SkillDiffViewer.vue` 组件
8. 在 `SkillCompareTable.vue` 添加 Diff 按钮
9. 在 `GlobalView.vue` 和 `ProjectDetailView.vue` 接入
10. 添加 i18n 翻译

---

## F-02: Operation History & Rollback

### 目标

记录每次安装/更新/卸载操作，包含足够元数据支持撤销。历史记录持久化到 `~/.spm/history.json`。

### 新增 Rust 模块

**`src-tauri/src/models/history.rs`** — 新文件：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Install,
    Update,
    Uninstall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub id: String,                           // UUID
    pub operation: OperationType,
    pub skill_name: String,
    pub target: String,                       // "global" 或项目路径
    pub timestamp: String,                    // ISO 8601
    pub repo_id: Option<String>,              // 来源仓库
    pub version_before: Option<String>,       // 操作前版本
    pub version_after: Option<String>,        // 操作后版本
    pub rollback_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct OperationHistory {
    pub records: Vec<OperationRecord>,
}
```

**`src-tauri/src/models/mod.rs`** — 追加 `pub mod history;`

### 新增 Rust 服务

**`src-tauri/src/services/history_service.rs`** — 新文件：

```rust
const HISTORY_FILE: &str = ".spm/history.json";

pub fn load_history(home: &str) -> OperationHistory           // 读取，不存在则返回默认
pub fn save_history(home: &str, history: &OperationHistory) -> Result<()>  // 持久化
pub fn record_operation(
    home: &str,
    operation: OperationType,
    skill_name: &str,
    target: &str,
    repo_id: Option<&str>,
    version_before: Option<&str>,
    version_after: Option<&str>,
    rollback_available: bool,
) -> Result<OperationRecord>                                  // 记录并保存
pub fn rollback_operation(
    home: &str,
    record: &OperationRecord,
    repos: &[RepoConfig],
    global_path: &str,
) -> Result<()>                                               // 回滚操作
```

回滚逻辑：
- **Install** → 卸载该 Skill
- **Uninstall** → 从源仓库重新安装
- **Update** → 需要备份旧版本（安装前将 Skill 目录复制到 `~/.spm/backups/{id}/`），回滚时从备份恢复

**`src-tauri/src/services/mod.rs`** — 追加 `pub mod history_service;`

### 新增 Rust 命令

**`src-tauri/src/commands/history.rs`** — 新文件：

```rust
#[tauri::command]
pub fn get_operation_history(
    state: &State<AppState>,
    limit: Option<u32>,
) -> Result<Vec<OperationRecord>, String>

#[tauri::command]
pub fn rollback_operation(
    state: &State<AppState>,
    operation_id: String,
) -> Result<(), String>

#[tauri::command]
pub fn clear_history(state: &State<AppState>) -> Result<(), String>
```

**`src-tauri/src/commands/mod.rs`** — 追加 `pub mod history;`

**`src-tauri/src/lib.rs`** — 注册 3 个命令

### 修改现有命令

**`src-tauri/src/commands/skill.rs`** — 在 `install_skill`、`update_skill`、`uninstall_skill` 中：

1. 操作前记录当前版本/状态
2. 执行操作
3. 操作后调用 `history_service::record_operation()`
4. 对 `update_skill`：操作前备份到 `~/.spm/backups/{uuid}/`

### 新增前端类型

**`src/types/index.ts`** — 追加：

```typescript
export type OperationType = 'Install' | 'Update' | 'Uninstall'

export interface OperationRecord {
  id: string
  operation: OperationType
  skillName: string
  target: string
  timestamp: string
  repoId?: string
  versionBefore?: string
  versionAfter?: string
  rollbackAvailable: boolean
}
```

### 新增 Store

**`src/stores/history.ts`** — 新文件：

```typescript
defineStore('history', () => {
  const records = ref<OperationRecord[]>([])
  const loading = ref(false)
  const rollbackingId = ref<string | null>(null)

  async function loadHistory() { ... }
  async function rollbackOperation(id: string) { ... }
  async function clearHistory() { ... }

  return { records, loading, rollbackingId, loadHistory, rollbackOperation, clearHistory }
})
```

### 新增组件

**`src/components/common/OperationHistoryPanel.vue`**

- 抽屉面板（NDrawer），从侧边栏按钮触发
- 操作列表（倒序），每项显示：时间戳、Skill 名称、操作类型徽章、目标、版本变化
- 每项有"回滚"按钮（`rollbackAvailable` 为 false 时禁用）
- 底部"清空历史"按钮（带确认对话框）

### 修改现有文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/models/history.rs` | 新文件 |
| `src-tauri/src/models/mod.rs` | 追加 `pub mod history;` |
| `src-tauri/src/services/history_service.rs` | 新文件 |
| `src-tauri/src/services/mod.rs` | 追加 `pub mod history_service;` |
| `src-tauri/src/commands/history.rs` | 新文件 |
| `src-tauri/src/commands/mod.rs` | 追加 `pub mod history;` |
| `src-tauri/src/commands/skill.rs` | install/update/uninstall 中记录历史 |
| `src-tauri/src/lib.rs` | 注册 3 个命令 |
| `src/types/index.ts` | 追加 `OperationType`、`OperationRecord` |
| `src/stores/history.ts` | 新文件 |
| `src/components/common/OperationHistoryPanel.vue` | 新文件 |
| `src/components/layout/Sidebar.vue` | 添加历史记录入口图标 |
| `src/components/layout/AppLayout.vue` | 集成 OperationHistoryPanel |
| `src/i18n/locales/zh-CN.json` | 添加 `history.*` 翻译键 |
| `src/i18n/locales/en-US.json` | 同上 |

### i18n 翻译键

```json
{
  "history": {
    "title": "操作历史",
    "empty": "暂无操作记录",
    "install": "安装",
    "update": "更新",
    "uninstall": "卸载",
    "rollback": "回滚",
    "rollbackSuccess": "回滚成功",
    "rollbackFailed": "回滚失败",
    "clear": "清空历史",
    "clearConfirm": "确定清空所有操作记录？此操作不可撤销。",
    "clearSuccess": "历史已清空",
    "target": "目标",
    "global": "全局",
    "project": "项目",
    "noRollback": "不可回滚",
    "loading": "加载中..."
  }
}
```

### 实现步骤

1. 创建 `models/history.rs`，注册到 `mod.rs`
2. 创建 `services/history_service.rs`，注册到 `mod.rs`
3. 实现 load / save / record / rollback 逻辑
4. 创建 `commands/history.rs`，注册到 `mod.rs` 和 `lib.rs`
5. 修改 `commands/skill.rs` 中 install/update/uninstall 集成历史记录
6. 在 `types/index.ts` 添加 TS 类型
7. 创建 `stores/history.ts`
8. 创建 `OperationHistoryPanel.vue`
9. 在 `Sidebar.vue` 添加入口，在 `AppLayout.vue` 集成面板
10. 添加 i18n 翻译
