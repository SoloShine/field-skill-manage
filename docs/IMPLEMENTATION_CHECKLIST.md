# SPM Manager 功能实现清单

## 项目约定速查

### Rust 后端

```
src-tauri/src/
├── models/        # 数据结构，derive(Serialize, Deserialize, Clone)
├── commands/      # 薄层命令处理器，锁定 Mutex → 提取配置 → 委托 service
├── services/      # 业务逻辑，返回 Result<T, String>
└── lib.rs         # 命令注册入口（tauri::generate_handler![]）
```

- 新模块必须在对应 `mod.rs` 中声明 `pub mod xxx;`
- 新命令必须在 `lib.rs` 的 `generate_handler![]` 中注册
- 可选字段用 `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`
- 所有配置访问通过 `state.config.lock()` 获取 `MutexGuard`
- Git 操作使用 `tokio::spawn_blocking` 实现并行

### Vue 前端

```
src/
├── types/index.ts      # TS 接口，与 Rust models 一一对应
├── stores/             # Pinia，defineStore('name', () => {...})
├── components/common/  # 可复用组件，props-down / events-up
├── composables/        # 组合式函数 (use*.ts)
├── views/              # 页面组件
├── i18n/locales/       # zh-CN.json + en-US.json，同步更新
└── router/index.ts     # 路由定义
```

- 组件使用 `<script setup lang="ts">`
- 用户可见字符串必须通过 `useI18n()` 的 `t('key')`
- Store 中 `invoke()` 调用包裹在 `try/finally` 中管理 loading 状态
- 路径别名 `@` → `src/`

### 关键文件清单

| 用途 | 文件路径 |
|------|----------|
| 命令注册 | `src-tauri/src/lib.rs` |
| 前后端类型契约 | `src/types/index.ts` |
| 核心业务逻辑 | `src-tauri/src/services/skill_service.rs` |
| 主 UI 交互面 | `src/components/common/SkillCompareTable.vue` |
| 国际化 (中文) | `src/i18n/locales/zh-CN.json` |
| 国际化 (英文) | `src/i18n/locales/en-US.json` |
| 路由定义 | `src/router/index.ts` |
| 侧边栏 | `src/components/layout/Sidebar.vue` |
| 应用外壳 | `src/components/layout/AppLayout.vue` |

---

## 功能总览

| 编号 | 优先级 | 功能 | 复杂度 | 依赖 |
|------|--------|------|--------|------|
| F-01 | P0 | Skill Diff Viewer | M | - |
| F-02 | P0 | Operation History & Rollback | L | - |
| F-03 | P1 | Skill Creation Wizard | L | - |
| F-04 | P1 | Auto Sync & Scheduled Updates | M | - |
| F-05 | P1 | Global Search (Full-text) | M | - |
| F-06 | P2 | Skill Presets/Profiles | M | - |
| F-07 | P2 | Skill Publishing | L | - |
| F-08 | P2 | Skill Dependency Management | L | - |
| F-09 | P3 | Keyboard Shortcuts | S | F-05 |
| F-10 | P3 | Drag & Drop Management | M | - |
| F-11 | P3 | Skill Integrity Monitor | M | - |
| F-12 | P3 | Skill Statistics Dashboard | L | F-02 |
| F-13 | P3 | Skill Tags & Favorites | M | - |
| F-14 | P3 | Changelog Display | S | - |

## 依赖关系图

```
F-01 ── standalone
F-02 ── standalone ────────────────────── F-12 依赖此
F-03 ── standalone
F-04 ── standalone
F-05 ── standalone ────────────────────── F-09 依赖此
F-06 ── standalone
F-07 ── standalone
F-08 ── standalone
F-09 ── uses F-05 (Ctrl+Shift+F)
F-10 ── standalone
F-11 ── standalone
F-12 ── uses F-02 (历史数据)
F-13 ── standalone
F-14 ── standalone
```

## 推荐实现顺序

| 阶段 | 功能 | 说明 |
|------|------|------|
| Phase 1 | F-01, F-02 | 核心体验提升，无外部依赖 |
| Phase 2 | F-04, F-05 | 自动化与搜索，独立开发 |
| Phase 3 | F-03, F-06 | 开发者体验与工作流 |
| Phase 4 | F-07, F-08 | 发布与依赖管理 |
| Phase 5 | F-09, F-14, F-13 | 快捷键、变更日志、标签收藏 |
| Phase 6 | F-11, F-12, F-10 | 完整性、统计、拖拽 |

## 详细文档索引

- [P0: Diff Viewer + Operation History](features/P0-core-experience.md)
- [P1: Skill Wizard + Auto Sync + Global Search](features/P1-developer-experience.md)
- [P2: Presets + Publishing + Dependencies](features/P2-workflow-enhancement.md)
- [P3: Shortcuts + Drag & Drop + Integrity + Stats + Tags + Changelog](features/P3-polish.md)
