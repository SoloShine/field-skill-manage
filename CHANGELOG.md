# Changelog

All notable changes to this project will be documented in this file.

---

## [1.1.0] - 2026-04-20

### ✨ Highlights

**Skillbase/SPM Standard Compliance** — Full support for parsing and displaying Skillbase-standard SKILL.md frontmatter fields, including nested objects (trigger, security, compatibility, dependencies).

### 🚀 Features

- **serde_yaml Parsing** — Replaced hand-written YAML parser with `serde_yaml` crate, supporting nested objects and complex frontmatter
- **Unified Data Model** — Consolidated `SkillFrontmatter` into `SkillMeta` as the single source of truth for all skill metadata
- **Skillbase Metadata Fields** — Now parses and displays: `author`, `language`, `trigger` (description/tags/file_patterns), `security` (permissions), `compatibility` (min_context_tokens/requires/models), `dependencies`, `repository`
- **SkillCard Enhancement** — Author name and language badge displayed inline; trigger.tags merged into tag display
- **SkillPreviewModal Enhancement** — Full metadata panel showing trigger conditions, security permissions (dangerous ones highlighted), compatibility requirements, and dependency list
- **skills.json Extension** — Manifest entries now support author, language, trigger, and security fields

### 🌐 Internationalization

- Added 7 new i18n keys for Skillbase metadata fields (author, language, repository, trigger, security, compatibility, dependencies) in both zh-CN and en-US

### 📐 Architecture

- Introduced `TriggerInfo`, `SecurityInfo`, `CompatibilityInfo` structs with `Default` derive and `#[serde(default)]` for backward compatibility
- Added `Default` derive to `SkillMeta` and `InstallStatus`
- Simplified `build_remote_skill_meta` and `build_local_skill_meta` using `Default` trait pattern

---

## [1.1.0] - 2026-04-20 (中文版)

### ✨ 亮点

**兼容 Skillbase/SPM 标准** — 完整支持解析和展示符合 Skillbase 标准的 SKILL.md frontmatter 字段，包括嵌套对象（trigger、security、compatibility、dependencies）。

### 🚀 新功能

- **serde_yaml 解析** — 用 `serde_yaml` crate 替代手写 YAML 解析器，支持嵌套对象和复杂 frontmatter
- **统一数据模型** — 将 `SkillFrontmatter` 合并到 `SkillMeta`，作为唯一元数据模型
- **Skillbase 元数据字段** — 解析并展示：`author`、`language`、`trigger`（description/tags/file_patterns）、`security`（permissions）、`compatibility`（min_context_tokens/requires/models）、`dependencies`、`repository`
- **SkillCard 增强** — 内联显示作者名称和语言徽章；trigger.tags 合并到标签展示
- **SkillPreviewModal 增强** — 完整元数据面板，展示触发条件、安全权限（危险权限高亮）、兼容性要求、依赖列表
- **skills.json 扩展** — 清单条目支持 author、language、trigger、security 字段

### 🌐 国际化

- 新增 7 个 Skillbase 元数据相关的中英文翻译键（author、language、repository、trigger、security、compatibility、dependencies）

### 📐 架构

- 引入 `TriggerInfo`、`SecurityInfo`、`CompatibilityInfo` 结构体，带 `Default` derive 和 `#[serde(default)]` 保证向后兼容
- `SkillMeta` 和 `InstallStatus` 添加 `Default` derive
- 简化 `build_remote_skill_meta` 和 `build_local_skill_meta`，使用 `Default` trait 模式

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
