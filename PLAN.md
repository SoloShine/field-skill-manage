# SPM Manager - Skill Package Manager GUI 工具实现方案

## Context

用户已有一个 Monorepo Skill 仓库 (`xip.rmip.skills`)，包含 9 个 skill，使用 `skills.json` 作为索引，各 skill 目录含 `SKILL.md`（YAML frontmatter 元数据）。当前缺少：版本校验码、更新时间追踪、自动化更新管理。

**目标**：构建一个 **Tauri 2 + Vue 3 + TypeScript** 桌面应用，提供全局和项目级别的 skill 版本管理、校验、更新安装功能。

**远端仓库**：`https://g.mtpmp.cn/tiany/xip.rmip.skills`
**本地项目路径**：`d:\Project\my-skill-manage`（空目录）

---

## 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| 桌面框架 | Tauri 2 | ^2.x |
| 前端 | Vue 3 + TypeScript | ^3.5 |
| 构建工具 | Vite | ^6.x |
| UI 组件库 | Naive UI | ^2.x |
| 状态管理 | Pinia | ^2.x |
| 路由 | Vue Router | ^4.x |
| 后端 (Rust) | sha2, serde, chrono, git2/CLI | - |
| 图标 | @vicons/ionicons5 | ^0.x |

---

## 项目结构

```
d:\Project\my-skill-manage/
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json          # Tauri 2 权限配置
│   └── src/
│       ├── main.rs               # 入口
│       ├── lib.rs                # Tauri setup + command 注册
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── skill.rs          # Skill 查询/安装/更新命令
│       │   ├── version.rs        # 版本校验/哈希计算命令
│       │   ├── git_sync.rs       # 远端仓库同步命令
│       │   └── config.rs         # 配置管理命令
│       ├── models/
│       │   ├── mod.rs
│       │   ├── skill.rs          # SkillMeta / FileEntry / Checksum
│       │   └── config.rs         # AppConfig / InstallTarget
│       └── services/
│           ├── mod.rs
│           ├── skill_service.rs  # Skill 读取/解析/安装逻辑
│           ├── hash_service.rs   # SHA256/MD5 文件哈希
│           └── git_service.rs    # Git clone/pull/fetch
├── src/                          # Vue 3 前端
│   ├── App.vue
│   ├── main.ts
│   ├── router/
│   │   └── index.ts              # 路由: /global, /project, /settings
│   ├── stores/
│   │   ├── skill.ts              # Skill 状态管理
│   │   ├── project.ts            # 项目路径管理
│   │   └── config.ts             # 全局配置
│   ├── types/
│   │   └── index.ts              # TypeScript 类型定义
│   ├── views/
│   │   ├── GlobalView.vue        # 全局 Skill 管理
│   │   ├── ProjectView.vue       # 项目级 Skill 管理
│   │   └── SettingsView.vue      # 设置页
│   ├── components/
│   │   ├── layout/
│   │   │   ├── AppLayout.vue     # 主布局 (侧边栏 + 内容区)
│   │   │   └── Sidebar.vue       # 导航侧边栏
│   │   ├── common/
│   │   │   ├── SkillCard.vue     # Skill 卡片 (名称/版本/标签/状态)
│   │   │   ├── VersionBadge.vue  # 版本状态徽章 (最新/可更新/未知)
│   │   │   ├── HashInfo.vue      # 哈希信息展示
│   │   │   └── EmptyState.vue    # 空状态提示
│   │   ├── global/
│   │   │   └── GlobalSkillList.vue
│   │   ├── project/
│   │   │   ├── ProjectSelector.vue    # 项目路径选择器
│   │   │   └── ProjectSkillList.vue
│   │   └── settings/
│   │       ├── RemoteConfig.vue       # 远端仓库配置
│   │       └── GlobalPathConfig.vue   # 全局 Skill 安装路径配置
│   └── styles/
│       └── global.css
├── index.html
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
└── .gitignore
```

---

## 核心数据模型

### Rust (models/skill.rs)

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct SkillMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub path: String,
    pub license: Option<String>,
    pub updated_at: Option<String>,      // ISO 8601
    pub checksum: Option<Checksum>,
    pub files: Option<Vec<FileEntry>>,
    pub install_status: Option<InstallStatus>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Checksum {
    pub algorithm: String,  // "sha256" | "md5"
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,        // "sha256:abc..." 或 "md5:def..."
    pub size: u64,
    pub mtime: String,       // ISO 8601
}

#[derive(Serialize, Deserialize, Clone)]
pub enum InstallStatus {
    Installed,               // 已安装，版本匹配
    Outdated,                // 已安装，但有新版本
    NotInstalled,            // 未安装
    Unknown,                 // 无法判断 (缺省元数据)
}
```

### TypeScript (types/index.ts)

```typescript
interface SkillMeta {
  name: string
  version: string
  description: string
  tags: string[]
  path: string
  license?: string
  updated_at?: string
  checksum?: Checksum
  files?: FileEntry[]
  install_status?: 'installed' | 'outdated' | 'not_installed' | 'unknown'
}
```

---

## Rust 后端 Commands (Tauri IPC)

### 1. 远端同步 (commands/git_sync.rs)

| Command | 功能 |
|---------|------|
| `sync_remote_repo` | 从远端 clone/pull 到本地缓存目录 |
| `get_remote_skills` | 读取远端缓存中的 skills.json |
| `get_remote_skill_detail` | 获取远端某个 skill 的完整信息+文件哈希 |

### 2. Skill 操作 (commands/skill.rs)

| Command | 功能 |
|---------|------|
| `get_global_skills` | 获取全局已安装的 skill 列表 |
| `get_project_skills` | 获取指定项目路径下的 skill 列表 |
| `install_skill(name, target)` | 安装 skill 到全局/项目 |
| `update_skill(name, target)` | 更新 skill 到最新版本 |
| `batch_update(names, target)` | 批量更新 |
| `uninstall_skill(name, target)` | 卸载 skill |

### 3. 版本校验 (commands/version.rs)

| Command | 功能 |
|---------|------|
| `calculate_skill_hash(path)` | 计算目录下所有文件的 SHA256 聚合哈希 |
| `compare_skill_versions(local, remote)` | 对比本地与远端版本差异 |
| `verify_skill_integrity(path, expected_hash)` | 校验 skill 文件完整性 |

### 4. 配置 (commands/config.rs)

| Command | 功能 |
|---------|------|
| `get_config` | 读取配置 |
| `set_config` | 保存配置 |

### 配置模型

```rust
pub struct AppConfig {
    pub remote_url: String,           // 远端仓库地址
    pub global_skill_path: String,    // 全局 skill 安装路径
    pub cache_path: String,           // 远端仓库本地缓存路径
    pub auto_sync: bool,              // 是否自动同步
}
```

---

## 前端页面设计

### 1. 全局 Skill 管理页 (GlobalView)

```
┌──────────────────────────────────────────────────┐
│ [同步远端]  全局路径: ~/.claude/skills            │
├──────────────────────────────────────────────────┤
│ 🔍 搜索...  [标签过滤 ▼]  [状态过滤 ▼]          │
├──────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────┐ │
│ │ db-best-practice  v1.0.0  [已安装 ✓]        │ │
│ │ 数据库使用规范...                             │ │
│ │ database dotnet orm  [更新] [卸载]           │ │
│ ├──────────────────────────────────────────────┤ │
│ │ entity-creator  v1.0.0  [可更新 ↑ v1.1.0]   │ │
│ │ 实体类创建规范...                             │ │
│ │ dotnet entity model  [更新] [卸载]           │ │
│ ├──────────────────────────────────────────────┤ │
│ │ api-definition  v1.0.0  [未安装]             │ │
│ │ API 接口定义规范...                           │ │
│ │ api frontend http  [安装]                    │ │
│ └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

### 2. 项目级 Skill 管理 (ProjectView)

```
┌──────────────────────────────────────────────────┐
│ 项目路径: [D:\Project\xxx]  [浏览...]            │
├──────────────────────────────────────────────────┤
│ 该项目已安装 3 个 skill:                          │
│ ┌──────────────────────────────────────────────┐ │
│ │ db-best-practice  v1.0.0  [最新 ✓]          │ │
│ │ SHA256: a1b2c3...  更新: 2026-04-13         │ │
│ ├──────────────────────────────────────────────┤ │
│ │ entity-creator  v0.9.0  [可更新 → v1.0.0]   │ │
│ │ [校验] [更新]                                │ │
│ └──────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────┤
│ 可安装的远端 Skill:                               │
│ ┌──────────────────────────────────────────────┐ │
│ │ api-definition  v1.0.0  [安装 → 项目]        │ │
│ └──────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────┘
```

### 3. 设置页 (SettingsView)

- 远端仓库地址配置
- 全局 Skill 安装路径
- 缓存目录配置
- 自动同步开关

---

## 哈希与版本策略

### 计算策略

1. **有元数据**（skills.json 中有 checksum）→ 直接使用 checksum 比对
2. **无 checksum 但有 version** → 先比 version 字符串，不同则更新
3. **完全没有元数据** → 计算整个 skill 目录的 SHA256 聚合哈希：
   - 遍历目录所有文件，每个文件计算 SHA256
   - 将所有 `filename:hash` 排序后拼接，再计算一次 SHA256 作为目录级 checksum
4. **极简回退** → 使用文件修改时间 (mtime) 比对

### 聚合哈希算法 (Rust)

```
for file in sorted(walk_dir(skill_path)):
    hash_map[file.relative_path] = sha256(file.content)
aggregate_hash = sha256(sorted_join(hash_map.entries, ":"))
```

---

## 关键 Rust crates

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sha2 = "0.10"
md-5 = "0.10"
hex = "0.4"
chrono = { version = "0.4", features = ["serde"] }
walkdir = "2"
git2 = "0.20"          # 或使用 Command 调用 git CLI
tokio = { version = "1", features = ["full"] }
thiserror = "2"
```

---

## 实现步骤

### Phase 1: 项目初始化
1. 使用 `npm create tauri-app@latest` 创建 Tauri 2 + Vue 3 + TypeScript 项目
2. 安装前端依赖: `naive-ui`, `vue-router`, `pinia`, `@vicons/ionicons5`
3. 配置 Rust 依赖
4. 配置 Tauri 权限 (capabilities/default.json)

### Phase 2: Rust 后端核心
5. 实现 `models/` - 数据模型定义
6. 实现 `services/hash_service.rs` - SHA256/MD5 文件哈希计算
7. 实现 `services/git_service.rs` - Git 远端 clone/pull
8. 实现 `services/skill_service.rs` - Skill 解析/安装/更新
9. 实现 `commands/` - 所有 Tauri IPC 命令
10. 注册命令到 `lib.rs`

### Phase 3: Vue 前端
11. 实现路由和布局组件 (AppLayout, Sidebar)
12. 实现全局 Skill 管理页
13. 实现项目级 Skill 管理页
14. 实现设置页
15. 实现 Pinia stores

### Phase 4: 集成与打磨
16. 前后端联调
17. 版本对比和更新流程测试
18. 错误处理和加载状态
19. 应用打包和分发配置

---

## 验证方式

1. **启动**：`npm run tauri dev` 能正常启动桌面窗口
2. **远端同步**：点击「同步远端」能从 `g.mtpmp.cn` 拉取最新 skill 列表
3. **全局管理**：能看到已安装/可更新的 skill 状态，能一键更新
4. **项目管理**：选择项目路径后能看到该项目下的 skill，能对比远端版本
5. **哈希校验**：对无元数据的 skill 能自动计算 SHA256 并显示
6. **版本对比**：本地版本与远端版本不同时显示「可更新」标记
