# Phase 2-4 规划路线图

**创建日期**: 2026-04-20
**前置条件**: Phase 1（Skillbase 元数据解析与展示）已完成，分支 `feat/skillbase-metadata`
**目的**: 为后续三个阶段提供范围界定、技术方案和依赖关系，待有带宽时逐一启动

---

## 阶段关系

```
Phase 1: 元数据解析 ✅ 已完成
    ↓
Phase 2: skillbase.json 依赖管理
    ↓
Phase 3: Registry 集成
    ↓
Phase 4: MCP 集成
```

Phase 2 是 Phase 3 的基础（Registry 需要 skillbase.json 声明依赖），Phase 4 可以与 Phase 3 并行。

---

## Phase 2: skillbase.json 依赖管理

### 目标

支持读取 `skillbase.json` 作为项目级 Skill 依赖声明，实现一键同步项目所需的所有 Skill。

### 背景

Skillbase 标准中 `skillbase.json` 等同于 `package.json`：

```json
{
  "schema_version": 1,
  "name": "my-project",
  "version": "1.0.0",
  "skills": {
    "@core/docx": "^1.2.0",
    "@core/xlsx": "^1.0.0"
  },
  "personas": { ... },
  "registry": "https://registry.skillbase.space"
}
```

与现有 `skills.json` 的本质区别：`skills.json` 是"仓库里有什么"，`skillbase.json` 是"项目需要什么"。

### 功能范围

| 功能 | 说明 |
|------|------|
| 解析 skillbase.json | 读取项目根目录的 skillbase.json，提取 skills 依赖列表 |
| 依赖解析 | 将 `@author/name: semver` 格式匹配到已配置仓库中的 skill |
| 一键同步 | 对比项目已安装 skill 与 skillbase.json 声明，批量安装/更新缺失的 |
| 可视化依赖状态 | 在项目详情页展示哪些依赖已满足、哪些缺失、哪些版本不匹配 |
| 生成 skillbase.json | 支持从当前已安装的 skill 列表反向生成 skillbase.json |

### 关键设计决策（待 brainstorming 时确定）

1. **semver 范围匹配**：是否引入 `semver` crate 进行版本范围解析（`^1.2.0`、`~2.1.0`），还是先做精确匹配简化实现
2. **多来源解析**：依赖声明用 `@author/name` 格式，但当前仓库用目录名匹配，需要设计 author → repo 的映射策略
3. **personas 支持**：Phase 2 是否同时支持 personas（Skill 组合），还是推到更后阶段
4. **生成 skillbase.json**：是否在安装/卸载 skill 时自动更新 skillbase.json

### 涉及文件预估

| 文件 | 变更类型 |
|------|---------|
| `src-tauri/src/models/skill.rs` | 新增 SkillbaseManifest 结构体 |
| `src-tauri/src/services/skill_service.rs` | 新增 parse_skillbase_manifest、resolve_dependencies 函数 |
| `src-tauri/src/commands/skill.rs` | 新增 IPC 命令 |
| `src/types/index.ts` | 新增 SkillbaseManifest 类型 |
| `src/views/ProjectDetailView.vue` | 新增依赖状态展示 |
| `src/stores/skill.ts` | 新增依赖解析逻辑 |

### 预估复杂度

**中等**。核心难点在 semver 匹配和多仓库来源解析。基础版（精确匹配 + 单仓库）可在 1-2 天内完成。

---

## Phase 3: Registry 集成

### 目标

对接 SPM Registry API，支持搜索和从远程 registry 安装 Skill，类似 `npm add`。

### 背景

Skillbase 提供公共 Registry（`https://registry.skillbase.space`）和自托管 Registry 能力。Registry API 支持：
- 搜索 Skill（按关键词、标签、文件模式）
- 下载 Skill 包（.tar.gz，SHA-256 校验）
- 发布 Skill（需认证）
- 依赖自动解析

### 功能范围

| 功能 | 说明 |
|------|------|
| Registry 配置 | 在设置页添加 Registry URL 和认证 Token 配置 |
| 搜索 Skill | 按 keyword/tag 搜索远程 Registry，展示结果列表 |
| 从 Registry 安装 | 下载 Skill 包，验证 SHA-256，安装到目标目录 |
| 发布 Skill | 将本地 Skill 发布到 Registry（需认证） |
| 依赖自动解析 | 安装 Skill 时自动解析并安装其声明的依赖 |

### 关键设计决策

1. **Registry 存储模型**：是在现有 `RepoConfig` 中添加 registry 类型，还是新建独立的 Registry 配置模型
2. **缓存策略**：Registry 搜索结果是否本地缓存，缓存时长
3. **认证方式**：API Token 直接存储 vs GitHub OAuth 设备流
4. **发布流程**：是否集成 `spm publish` 等价功能到 GUI
5. **安全扫描**：Registry 扫描的 prompt injection 检测是否在前端展示

### 涉及文件预估

| 文件 | 变更类型 |
|------|---------|
| `src-tauri/src/models/config.rs` | 扩展 RepoConfig 支持Registry 类型 |
| `src-tauri/src/services/` | 新增 registry_service.rs |
| `src-tauri/src/commands/` | 新增 registry.rs |
| `src/views/` | 可能新增 RegistrySearchView |
| `src/components/` | 搜索结果列表、安装进度组件 |
| `src/stores/` | 新增 registry.ts |

### 预估复杂度

**高**。涉及 HTTP 客户端、认证、包下载/解压/校验、依赖图解析。建议分两步：先做搜索+安装，再做发布。

---

## Phase 4: MCP 集成

### 目标

让 SPM Manager 自身作为 MCP Server 运行，或配置其他 AI 客户端的 MCP 连接，实现 Skill 自动发现和加载。

### 背景

Skillbase/SPM 通过 MCP（Model Context Protocol）协议暴露 Skill 给 AI 客户端。6 个 MCP 工具：

| 工具 | 功能 |
|------|------|
| `skill_list` | 返回已安装 Skill 索引 |
| `skill_load` | 加载完整 Skill 指令到上下文 |
| `skill_context` | 当前会话状态 |
| `skill_search` | 搜索本地/远程 Skill |
| `skill_feedback` | 记录使用反馈 |
| `skill_install` | 从 Registry 安装 |

### 功能范围

| 功能 | 说明 |
|------|------|
| MCP Server 模式 | SPM Manager 启动时可选运行 `spm serve --stdio` 等价的 MCP 服务 |
| 客户端配置 | 一键配置 Claude/Cursor/VS Code 等 AI 客户端的 MCP 连接 |
| Skill 索引嵌入 | 在 MCP 指令中嵌入紧凑的 Skill 索引，AI 自动匹配 |
| Trigger 匹配 | 利用 Phase 1 解析的 trigger 信息，辅助 AI 选择正确的 Skill |
| 使用反馈 | Skill 使用后收集反馈，影响置信度分数 |

### 关键设计决策

1. **内嵌 MCP vs 独立进程**：是在 Tauri 应用内运行 MCP Server，还是作为独立进程
2. **Trigger 展示**：trigger.description 如何嵌入 MCP 指令，格式是紧凑管道分隔还是 JSON
3. **客户端配置**：支持哪些客户端（Claude Desktop、Claude Code、Cursor、VS Code 等14种），配置文件路径差异
4. **权限模型**：Phase 1 解析的 security.permissions 是否在 MCP 层面强制执行

### 涉及文件预估

| 文件 | 变更类型 |
|------|---------|
| `src-tauri/src/services/` | 新增 mcp_service.rs |
| `src-tauri/src/commands/` | 新增 mcp.rs |
| `src/views/SettingsView.vue` | 添加 MCP 配置区域 |
| `src-tauri/Cargo.toml` | 可能需要 MCP 协议库 |

### 预估复杂度

**高**。MCP 协议实现、多客户端配置文件路径差异、运行时 Skill 选择逻辑。建议在 Phase 2/3 完成后再启动。

---

## 实施建议

1. **先合并 Phase 1**：将 `feat/skillbase-metadata` 合并到 main，确保基础稳固
2. **Phase 2 是关键路径**：skillbase.json 支持是 Phase 3（Registry 需要 resolve `@author/name`）和 Phase 4（MCP skill_list 需要知道项目依赖了什么）的基础
3. **Phase 3 和 Phase 4 可部分并行**：Registry 搜索/安装和 MCP Server 实现相对独立
4. **每个 Phase 都应先做 brainstorming + design spec**：按 Phase 1 的流程，先写设计文档再写实施计划
