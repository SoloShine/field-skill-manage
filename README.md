# SPM Manager

**[English](README.en.md)** | 简体中文

Skill Package Manager 桌面管理工具 — 基于 Tauri 2 + Vue 3 + Rust 构建的跨平台应用。

用于管理 AI Agent 的 Skill 包，支持全局和项目级别的安装、更新、差异对比和卸载。

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)
![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)

## 功能特性

### 核心管理

- **多 Agent 支持** — Claude Code、OpenCode、Codex、Cursor、Windsurf，以及自定义 Agent
- **全局 Skill 管理** — 查看、安装、更新、卸载 Agent 全局目录下的 Skill
- **项目 Skill 管理** — 添加多个项目，概览各项目 Skill 安装情况，进入详情独立管理
- **多仓库支持** — 配置多个远端 Git 仓库作为 Skill 来源，独立启用/禁用
- **批量操作** — 一键同步远端、批量更新所有过期 Skill、批量安装/卸载

### 版本对比与 Diff

- **4 级版本校验** — SHA256 哈希 → 版本号 → 目录聚合哈希 → 文件修改时间
- **文件级差异对比** — 对 Outdated Skill 展示逐文件的 Added/Removed/Modified/Unchanged 状态
- **行级内容 Diff** — 点击变更文件查看本地与远端的逐行代码差异，支持语法高亮
- **Skill 预览** — 文件树浏览 + Markdown 渲染，YAML 元数据结构化展示

### 操作历史与回滚

- **操作历史记录** — 自动记录每次安装、更新、卸载操作（含版本快照）
- **一键回滚** — 对支持回滚的操作可撤销，恢复到操作前状态
- **历史管理** — 查看最近 200 条操作记录，支持清空历史

### Skill 开发

- **开发指南** — 内置完整的 Skill 开发文档，涵盖 frontmatter、触发器、权限、正文编写
- **仓库结构支持** — 优先解析 `skills.json` 清单，回退为扫描含 `SKILL.md` 的目录

### 个性化

- **深色/浅色主题** — 跟随系统或手动切换
- **7 种预设强调色** — Ocean Blue、Teal、Emerald、Amber、Rose、Violet、Slate
- **自定义强调色** — 支持任意 HEX 色值
- **中英双语** — 简体中文（默认）和 English 切换
- **应用自更新** — 检查 GitHub Releases，有新版时侧边栏提示

### 配置管理

- **自定义 Agent** — 配置全局 Skill 路径和项目目录模式
- **配置导入/导出** — 导出为可移植 JSON（`${HOME}` 占位符），跨机器迁移

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Composition API |
| 构建 | Vite |
| UI 组件库 | Naive UI |
| 状态管理 | Pinia |
| 国际化 | vue-i18n |
| Markdown | marked + highlight.js |
| Diff 算法 | diff |
| 后端 | Rust (sha2, serde, chrono, walkdir, tokio, reqwest) |

## 开发

### 前置条件

- Node.js >= 18
- Rust >= 1.70
- 系统需安装 Git CLI

### 启动开发服务器

```bash
npm install
npm run tauri dev
```

Vite 前端端口 `:1420`，Rust 后端支持热重载。

### 构建发布

```bash
npm run tauri build
```

输出安装包在 `src-tauri/target/release/bundle/` 目录。

### 版本管理

```bash
npm run version:bump patch   # 0.2.0 → 0.2.1
npm run version:bump minor   # 0.2.0 → 0.3.0
npm run version:bump major   # 0.2.0 → 1.0.0
npm run version:bump 1.5.0   # 指定版本号
```

同步更新 `package.json`、`Cargo.toml`、`tauri.conf.json` 三处版本号。

## 项目结构

```
├── src/                       # Vue 3 前端
│   ├── components/
│   │   ├── layout/            # 侧边栏、应用外壳
│   │   └── common/            # 通用组件
│   │       ├── SkillCompareTable.vue    # 对比表格（批量操作）
│   │       ├── SkillPreviewModal.vue    # 文件预览弹窗
│   │       ├── SkillDiffViewer.vue      # 版本差异 + 代码 Diff
│   │       ├── OperationHistoryPanel.vue # 操作历史抽屉
│   │       ├── EmptyState.vue           # 空状态
│   │       └── VersionBadge.vue         # 版本状态徽章
│   ├── views/                 # 页面
│   │   ├── GlobalView.vue     # 全局 Skill 管理
│   │   ├── ProjectListView.vue # 项目列表
│   │   ├── ProjectDetailView.vue # 项目 Skill 详情
│   │   ├── SettingsView.vue   # 设置（常规/仓库/Agent）
│   │   └── GuideView.vue      # Skill 开发指南
│   ├── stores/                # Pinia 状态管理
│   ├── composables/           # 组合式函数（主题、国际化）
│   ├── i18n/                  # 中英翻译文件
│   ├── types/                 # TypeScript 类型定义
│   └── router/                # 路由配置
├── src-tauri/                 # Rust 后端
│   └── src/
│       ├── commands/          # Tauri IPC 命令（约 25 个）
│       ├── services/          # 业务逻辑
│       │   ├── skill_service.rs   # Skill 发现、对比、安装
│       │   ├── git_service.rs     # Git clone/pull 同步
│       │   ├── hash_service.rs    # SHA256/MD5 哈希
│       │   ├── history_service.rs # 操作历史持久化
│       │   └── update_service.rs  # GitHub 自更新检查
│       └── models/            # 数据模型
├── docs/                      # 文档
│   ├── IMPLEMENTATION_CHECKLIST.md   # 功能实现清单索引
│   └── features/                     # 各优先级功能详细规划
├── scripts/                   # 构建脚本（版本提升）
└── .github/workflows/         # CI/CD（三平台构建发布）
```

## 支持的 Agent 及默认路径

| Agent | 全局路径 | 项目路径模式 |
|-------|---------|-------------|
| Claude Code | `~/.claude/skills` | `{project}/.claude/skills` |
| OpenCode | `~/.opencode/skills` | `{project}/.opencode/skills` |
| Codex (OpenAI) | `~/.codex/skills` | `{project}/.codex/skills` |
| Cursor | `~/.cursor/skills` | `{project}/.cursor/skills` |
| Windsurf / Cline | `~/.windsurf/skills` | `{project}/.windsurf/skills` |
| 自定义 | 可配置 | 可配置 |

所有路径均可在设置页自定义。

## Skill 版本校验策略

1. **有 checksum** → 直接 SHA256 哈希比对
2. **无 checksum 有 version** → 版本号对比
3. **完全没有元数据** → 计算目录级 SHA256 聚合哈希（遍历所有文件，排序后拼接再哈希）
4. **极简回退** → 文件修改时间 (mtime) 比对

## 远端仓库

默认从 `https://github.com/anthropics/skills.git` 同步 Skill 列表，可在设置中添加多个仓库。

## License

MIT
