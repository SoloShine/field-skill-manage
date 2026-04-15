# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

SPM Manager — 基于 Tauri 2 的桌面应用，用于管理 AI Agent 的 Skill 包。支持对多种 Agent（Claude Code、OpenCode、Codex、Cursor、Windsurf）进行全局和项目级别的 Skill 安装、更新、校验和卸载。前端使用 Vue 3 + TypeScript，后端使用 Rust。

## 常用命令

```bash
npm install                # 安装前端依赖
npm run tauri dev          # 启动开发服务器（Vite 端口 :1420 + Rust 后端热重载）
npm run tauri build        # 生产构建 → src-tauri/target/release/bundle/
npm run build              # 仅前端构建（vue-tsc 类型检查 + vite build）
npm run version:bump patch # 版本号提升（patch|minor|major|x.y.z），同步更新 3 个文件
```

项目未配置测试框架，没有测试命令。

## 架构

### 前后端通信

所有数据通过 Tauri IPC（`invoke()`）传递。Rust 后端暴露约 20 个 `#[tauri::command]` 函数，在 [lib.rs](src-tauri/src/lib.rs) 中注册。前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用。没有 REST API 或 HTTP 服务器，纯桌面 IPC。

### Rust 后端（src-tauri/src/）

三层结构：`commands/` → `services/` → `models/`

- **commands/** — 薄层 `#[tauri::command]` 处理器，锁定 `AppState`、提取配置，然后委托给 services。所有状态访问都通过 `Mutex<AppConfig>`。
- **services/** — 业务逻辑：
  - `skill_service.rs` — Skill 发现、对比、安装/卸载。两种发现策略：优先解析 `skills.json` 清单文件，回退为扫描含 `SKILL.md` 的目录并解析 YAML frontmatter。
  - `git_service.rs` — `git clone --depth 1` / `git pull --ff-only` 同步远端仓库。损坏时回退为全新克隆。
  - `hash_service.rs` — SHA256/MD5 文件哈希和目录聚合哈希（按路径排序后拼接每文件哈希，再整体哈希）。
  - `update_service.rs` — 通过 GitHub Releases API 检查应用自更新。
- **models/** — 共享类型：`AppConfig`、`SkillMeta`、`SkillComparison`、`RepoConfig`、`AgentType` 枚举。

### 前端（src/）

- **views/** — 四个页面：`GlobalView`（全局 Skill 管理）、`ProjectListView` + `ProjectDetailView`（项目 Skill）、`SettingsView`（Agent/仓库配置）。
- **stores/** — Pinia 状态管理：`config`（Agent/仓库设置）、`skill`（对比 + 同步）、`project`（项目路径，存 localStorage）、`update`（自更新检查）。
- **components/** — `layout/`（侧边栏 + 应用外壳）、`common/`（Skill 卡片、对比表格、预览弹窗、版本徽章）。
- **composables/useTheme.ts** — 深色/浅色主题 + 强调色管理，通过 CSS 变量 + Tauri 窗口主题同步实现。
- **i18n/** — vue-i18n，支持 `zh-CN`（默认）和 `en-US`，语言设置存 localStorage。
- **types/index.ts** — TypeScript 接口定义，与 Rust models 一一对应。修改 Rust 类型时必须同步更新此文件。

### 配置系统

- 配置文件：`~/.spm/config.json`（首次运行自动创建）。
- `AppConfig` 包含：当前活跃 Agent、Agent 路径映射、仓库列表、自定义 Agent。
- 多仓库支持：`repos: Vec<RepoConfig>`，并支持从旧版单 `remote_url` 自动迁移。
- 导出/导入功能会将主目录替换为 `${HOME}` 占位符以实现可移植性。

### Skill 版本对比策略（4 级回退）

1. SHA256 校验和匹配 → `Same`
2. 版本号不匹配 → `Outdated`
3. 无元数据 → 目录聚合 SHA256 对比
4. 最终回退 → 文件修改时间（mtime）对比

## 版本管理

版本号在 3 个文件中定义，必须保持同步：

| 文件 | 字段 |
|------|------|
| `package.json` | `"version": "x.y.z"` |
| `src-tauri/Cargo.toml` | `version = "x.y.z"` |
| `src-tauri/tauri.conf.json` | `"version": "x.y.z"` |

始终使用 `npm run version:bump <type>` 来同步更新三处。脚本位于 [scripts/version-bump.mjs](scripts/version-bump.mjs)。

## 发版流程

完整的发版规范和自动化步骤已记录在 release skill 中（`.claude/skills/release/SKILL.md`），通过 `/release` 命令触发。CI 由推送 `v*` 标签触发（见 [.github/workflows/release.yml](.github/workflows/release.yml)），构建 Windows、macOS（ARM）和 Ubuntu 三平台安装包。

## 关键约定

- 路径别名：`@` → `src/`（配置在 [vite.config.ts](vite.config.ts)）
- Rust 端跨平台主目录：优先读 `USERPROFILE`（Windows），回退 `HOME`
- Skill 元数据：通过 `SKILL.md` 中的 YAML frontmatter 定义（Rust 端使用极简 YAML 解析器，无 serde_yaml 依赖）
- Agent 项目路径使用 `{project}` 占位符模式（如 `{project}/.claude/skills`）
