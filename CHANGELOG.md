# Changelog

All notable changes to this project will be documented in this file.

---

## [1.0.0] - 2026-04-16

### ✨ Highlights

**Skill Diff Viewer** — File-level and line-level diff comparison for skill version changes. Click any outdated skill to see exactly what changed before updating.

**Operation History & Rollback** — Every install, update, and uninstall is recorded with full metadata. Browse history and rollback to previous versions with one click.

**Skill Development Guide** — Built-in guide page covering SKILL.md specification, directory structure, frontmatter fields, trigger configuration, security permissions, and validation workflow.

### 🚀 Features

- **Multi-Agent Management** — Support for Claude Code, OpenCode, Codex, Cursor, and Windsurf agents
- **Global & Project-level Skills** — Install and manage skills at both global and per-project scopes
- **Multi-repository Support** — Configure multiple skill repositories with async parallel sync
- **Skill Version Comparison** — 4-level fallback strategy: SHA256 checksum → version string → aggregate hash → file mtime
- **Batch Operations** — Multi-select skills for bulk install, update, or uninstall with confirmation dialogs
- **Configuration Import/Export** — Portable configs with `${HOME}` path placeholder substitution
- **Self-update Detection** — Auto-check for new versions on startup with red dot badge notification
- **Skill Diff Viewer** — File-level and line-level diff comparison for outdated skills
- **Operation History** — Persistent operation log with rollback support (`~/.spm/history.json`)
- **Skill Development Guide** — Interactive guide page for SKILL.md authoring

### 🎨 UI / UX

- Dark/Light theme with customizable accent colors
- Skeleton loading states for data-fetching phases
- Page transition animations with cross-fade
- Sidebar menu icons and breadcrumb navigation
- Search with `Ctrl+K` shortcut
- Sortable table columns with multi-select checkboxes
- Empty states with call-to-action guidance
- Inline loading indicators on action buttons
- Agent switch feedback toast notifications
- Fixed table headers for long skill lists
- Markdown rendering for update release notes

### 🌐 Internationalization

- Chinese (`zh-CN`) and English (`en-US`) support
- Language preference persisted in `localStorage`

### ⚡ Performance

- Async parallel repository sync (`tokio::spawn_blocking`) to prevent UI freezing
- `git clone --depth 1` shallow clones for fast repository sync
- Corrupted repo auto-fallback to fresh clone

### 🏗️ CI / CD

- GitHub Actions multi-platform build (Windows, macOS Apple Silicon, Ubuntu)
- Automated release pipeline triggered by `v*` tags
- Rust build cache via `swatinem/rust-cache`

### 🐛 Bug Fixes

- Eliminated page switch blank flash and horizontal scrollbar
- Removed unused macOS Intel build (incompatible with free GitHub Actions)
- Fixed workflow permissions for Release asset uploads
- Cleaned up unused reactive imports

---

## [1.0.0] - 2026-04-16 (中文版)

### ✨ 亮点

**Skill 版本对比 (Diff Viewer)** — 文件级和行级差异对比，点击 Outdated 状态的 Skill 即可查看具体变更内容，升级前心中有数。

**操作历史与回滚** — 每次安装、更新、卸载操作均自动记录完整元数据。浏览历史记录，一键回滚到先前版本。

**Skill 开发指南** — 内置交互式指南页面，涵盖 SKILL.md 规范、目录结构、frontmatter 字段、触发器配置、安全权限及验证安装流程。

### 🚀 新功能

- **多 Agent 管理** — 支持 Claude Code、OpenCode、Codex、Cursor、Windsurf 五种 Agent
- **全局与项目级 Skill** — 支持全局范围和单个项目维度的 Skill 安装与管理
- **多仓库支持** — 配置多个 Skill 远端仓库，异步并行同步
- **Skill 版本对比策略** — 4 级回退：SHA256 校验和 → 版本号 → 目录聚合哈希 → 文件修改时间
- **批量操作** — 多选 Skill 批量安装、更新或卸载，带确认弹窗
- **配置导入/导出** — 便携配置，自动替换 `${HOME}` 路径占位符
- **自更新检测** — 启动时自动检查新版本，红点徽标提醒
- **Skill Diff Viewer** — Outdated Skill 的文件级与行级代码差异对比
- **操作历史** — 持久化操作记录，支持一键回滚（`~/.spm/history.json`）
- **Skill 开发指南** — 交互式 SKILL.md 编写指南页面

### 🎨 界面 / 交互

- 深色/浅色主题，支持自定义强调色
- 骨架屏加载状态
- 页面切换交叉淡入过渡动画
- 侧边栏菜单图标 + 面包屑导航
- `Ctrl+K` 快捷搜索
- 表格列排序 + 多选复选框
- 空状态引导与 CTA 按钮
- 操作按钮行内 loading 动画
- Agent 切换反馈 Toast
- Skill 列表固定表头
- 更新说明 Markdown 渲染

### 🌐 国际化

- 支持中文（`zh-CN`）和英文（`en-US`）
- 语言偏好存储在 `localStorage`

### ⚡ 性能

- 异步并行仓库同步（`tokio::spawn_blocking`），避免 UI 卡死
- `git clone --depth 1` 浅克隆加速仓库同步
- 仓库损坏时自动回退为全新克隆

### 🏗️ CI / CD

- GitHub Actions 多平台构建（Windows、macOS Apple Silicon、Ubuntu）
- `v*` 标签触发的自动化发版流水线
- Rust 构建缓存（`swatinem/rust-cache`）

### 🐛 问题修复

- 消除页面切换空白闪烁和横向滚动条
- 移除 macOS Intel 构建（免费账户不可用）
- 修复 Release 写入权限问题
- 清理未使用的 reactive 导入
