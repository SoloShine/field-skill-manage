# SPM Manager

Skill Package Manager 桌面管理工具 — 基于 Tauri 2 + Vue 3 + TypeScript 构建。

用于管理 AI Agent 的 Skill 包，支持全局和项目级别的安装、更新、校验和卸载。

## 功能

- **多 Agent 支持** — Claude Code、OpenCode、Codex、Cursor、Windsurf，以及自定义 Agent
- **全局管理** — 查看和管理 Agent 全局目录下的已安装 Skill
- **项目管理** — 添加多个项目，概览各项目 Skill 安装情况，点击进入详情对比
- **版本对比** — 本地 vs 远端 Skill 版本、SHA256 哈希、更新时间的逐行对比
- **一键操作** — 同步远端、批量更新、安装/卸载
- **Skill 预览** — 文件树浏览 + Markdown 渲染，YAML 元数据结构化展示
- **自定义 Agent** — 自定义全局目录和项目目录模式

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript |
| 构建 | Vite |
| UI | Naive UI |
| 状态管理 | Pinia |
| 后端 | Rust (sha2, serde, chrono, walkdir) |
| Markdown | marked |

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

### 构建发布

```bash
npm run tauri build
```

输出安装包在 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
├── src/                    # Vue 3 前端
│   ├── components/         # 组件
│   │   ├── layout/         # 布局（侧边栏、主布局）
│   │   └── common/         # 通用组件（对比表格、预览弹窗、版本徽章）
│   ├── views/              # 页面（全局管理、项目列表、项目详情、设置）
│   ├── stores/             # Pinia 状态管理
│   ├── types/              # TypeScript 类型定义
│   └── router/             # 路由配置
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands/       # Tauri IPC 命令
│       ├── models/         # 数据模型
│       └── services/       # 业务逻辑（哈希、Git、Skill 解析）
├── package.json
├── vite.config.ts
└── tsconfig.json
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

默认从 `https://g.mtpmp.cn/tiany/xip.rmip.skills` 同步 Skill 列表，可在设置中修改。

## License

MIT
