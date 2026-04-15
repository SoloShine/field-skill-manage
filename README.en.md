# SPM Manager

简体中文 | **[English](README.md)**

A cross-platform desktop app for managing AI Agent Skill packages, built with Tauri 2 + Vue 3 + Rust.

Supports global and project-level skill installation, updates, diff comparison, and uninstallation.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-orange)
![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)

## Features

### Core Management

- **Multi-Agent Support** — Claude Code, OpenCode, Codex, Cursor, Windsurf, and custom agents
- **Global Skill Management** — View, install, update, and uninstall skills in the agent's global directory
- **Project Skill Management** — Add multiple projects, overview skill status per project, manage independently
- **Multi-Repository Support** — Configure multiple remote Git repositories as skill sources, enable/disable independently
- **Batch Operations** — One-click remote sync, batch update all outdated skills, batch install/uninstall

### Version Comparison & Diff

- **4-Tier Version Verification** — SHA256 hash → version string → directory aggregate hash → file mtime
- **File-Level Diff** — Shows per-file Added/Removed/Modified/Unchanged status for outdated skills
- **Line-Level Content Diff** — Click a changed file to view line-by-line code differences between local and remote
- **Skill Preview** — File tree browsing + Markdown rendering with structured YAML metadata display

### Operation History & Rollback

- **Operation History** — Automatically records every install, update, and uninstall operation with version snapshots
- **One-Click Rollback** — Undo supported operations and restore to the previous state
- **History Management** — View up to 200 recent operation records, with option to clear history

### Skill Development

- **Development Guide** — Built-in comprehensive skill development documentation covering frontmatter, triggers, permissions, and body writing
- **Repository Structure Support** — Prioritizes `skills.json` manifest parsing, falls back to scanning directories containing `SKILL.md`

### Personalization

- **Dark/Light Theme** — Follow system preference or switch manually
- **7 Preset Accent Colors** — Ocean Blue, Teal, Emerald, Amber, Rose, Violet, Slate
- **Custom Accent Color** — Supports any HEX color value
- **Bilingual UI** — Simplified Chinese (default) and English
- **Auto-Update** — Checks GitHub Releases for new versions, shows notification in sidebar

### Configuration Management

- **Custom Agents** — Configure global skill paths and project directory patterns
- **Config Import/Export** — Export to portable JSON (with `${HOME}` placeholders) for cross-machine migration

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2 |
| Frontend | Vue 3 + TypeScript + Composition API |
| Build | Vite |
| UI Library | Naive UI |
| State Management | Pinia |
| Internationalization | vue-i18n |
| Markdown | marked + highlight.js |
| Diff Algorithm | diff |
| Backend | Rust (sha2, serde, chrono, walkdir, tokio, reqwest) |

## Development

### Prerequisites

- Node.js >= 18
- Rust >= 1.70
- Git CLI installed on your system

### Start Dev Server

```bash
npm install
npm run tauri dev
```

Vite frontend runs on port `:1420`, Rust backend supports hot reload.

### Production Build

```bash
npm run tauri build
```

Output installers are in `src-tauri/target/release/bundle/`.

### Version Management

```bash
npm run version:bump patch   # 0.2.0 → 0.2.1
npm run version:bump minor   # 0.2.0 → 0.3.0
npm run version:bump major   # 0.2.0 → 1.0.0
npm run version:bump 1.5.0   # Specify exact version
```

Syncs version across `package.json`, `Cargo.toml`, and `tauri.conf.json`.

## Project Structure

```
├── src/                       # Vue 3 frontend
│   ├── components/
│   │   ├── layout/            # Sidebar, app shell
│   │   └── common/            # Shared components
│   │       ├── SkillCompareTable.vue    # Comparison table (batch ops)
│   │       ├── SkillPreviewModal.vue    # File preview modal
│   │       ├── SkillDiffViewer.vue      # Version diff + code diff
│   │       ├── OperationHistoryPanel.vue # Operation history drawer
│   │       ├── EmptyState.vue           # Empty state
│   │       └── VersionBadge.vue         # Version status badge
│   ├── views/                 # Pages
│   │   ├── GlobalView.vue     # Global skill management
│   │   ├── ProjectListView.vue # Project list
│   │   ├── ProjectDetailView.vue # Project skill details
│   │   ├── SettingsView.vue   # Settings (General/Repository/Agents)
│   │   └── GuideView.vue      # Skill development guide
│   ├── stores/                # Pinia state management
│   ├── composables/           # Composables (theme, i18n)
│   ├── i18n/                  # zh-CN and en-US translations
│   ├── types/                 # TypeScript type definitions
│   └── router/                # Route configuration
├── src-tauri/                 # Rust backend
│   └── src/
│       ├── commands/          # Tauri IPC commands (~25)
│       ├── services/          # Business logic
│       │   ├── skill_service.rs   # Skill discovery, comparison, installation
│       │   ├── git_service.rs     # Git clone/pull sync
│       │   ├── hash_service.rs    # SHA256/MD5 hashing
│       │   ├── history_service.rs # Operation history persistence
│       │   └── update_service.rs  # GitHub auto-update check
│       └── models/            # Data models
├── docs/                      # Documentation
│   ├── IMPLEMENTATION_CHECKLIST.md   # Feature roadmap index
│   └── features/                     # Detailed feature plans by priority
├── scripts/                   # Build scripts (version bump)
└── .github/workflows/         # CI/CD (3-platform build & release)
```

## Supported Agents & Default Paths

| Agent | Global Path | Project Pattern |
|-------|------------|-----------------|
| Claude Code | `~/.claude/skills` | `{project}/.claude/skills` |
| OpenCode | `~/.opencode/skills` | `{project}/.opencode/skills` |
| Codex (OpenAI) | `~/.codex/skills` | `{project}/.codex/skills` |
| Cursor | `~/.cursor/skills` | `{project}/.cursor/skills` |
| Windsurf / Cline | `~/.windsurf/skills` | `{project}/.windsurf/skills` |
| Custom | Configurable | Configurable |

All paths can be customized in the Settings page.

## Skill Version Verification Strategy

1. **Has checksum** → Direct SHA256 hash comparison
2. **No checksum, has version** → Version string comparison
3. **No metadata at all** → Compute directory-level SHA256 aggregate hash (walk all files, sort by path, concatenate and hash)
4. **Final fallback** → File modification time (mtime) comparison

## Remote Repositories

By default, skills are synced from `https://github.com/anthropics/skills.git`. Multiple repositories can be added in Settings.

## License

MIT
