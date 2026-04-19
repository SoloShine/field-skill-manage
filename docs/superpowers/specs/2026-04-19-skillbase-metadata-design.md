# Phase 1: Skillbase SKILL.md 元数据解析与展示

**日期**: 2026-04-19
**状态**: 已批准
**范围**: 完整支持 Skillbase 标准的 SKILL.md frontmatter 字段解析和前端展示

## 背景

SPM Manager 是一个管理 AI Agent Skill 包的桌面应用。当前仅解析 SKILL.md 中的 6 个基础字段（name、version、description、tags、license、updated_at），使用手写 strip_prefix 逐行解析器，无法处理 Skillbase 标准中的嵌套对象（trigger、security、compatibility）。

目标：完整兼容 Skillbase/SPM 的 SKILL.md frontmatter 规范，为后续 Phase 2-4（skillbase.json、Registry、MCP 集成）奠定基础。

## 设计决策

### 1. 统一单一数据模型

**决策**：使用 `SkillMeta` 作为唯一数据模型，替代现有的 `SkillFrontmatter`。

**理由**：避免两个模型同步维护的负担。新增字段只需改 `SkillMeta` + 前端 `types/index.ts` 一处。

**删除**：
- `SkillFrontmatter` 结构体
- `parse_simple_yaml_frontmatter()` 函数
- `parse_frontmatter_from_string()` 函数
- `frontmatter_to_meta()` 映射函数（不再需要）

### 2. 引入 serde_yaml 替代手写解析

**决策**：添加 `serde_yaml` crate，SKILL.md frontmatter 直接反序列化为 `SkillMeta`。

**理由**：手写解析器无法处理嵌套对象，YAML 解析是已解决的问题，不值得自己实现。

### 3. 前端扩展现有组件

**决策**：在 `SkillCard.vue` 和 `SkillPreviewModal.vue` 中直接添加新字段展示区域，不新建独立页面。

**理由**：新字段是补充信息，不适合单独成页；保持用户现有操作习惯。

## 数据模型变更

### Rust — SkillMeta 扩展（models/skill.rs）

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillMeta {
    // 通用字段
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub trigger: Option<TriggerInfo>,
    #[serde(default)]
    pub security: Option<SecurityInfo>,
    #[serde(default)]
    pub compatibility: Option<CompatibilityInfo>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,

    // 运行时填充字段（service 层填入）
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub checksum: Option<Checksum>,
    #[serde(default)]
    pub files: Option<Vec<FileEntry>>,
    #[serde(default)]
    pub install_status: Option<InstallStatus>,
    #[serde(default)]
    pub source_repo_id: Option<String>,
}
```

### 新增子结构体

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriggerInfo {
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub file_patterns: Vec<String>,
    pub priority: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityInfo {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompatibilityInfo {
    pub min_context_tokens: Option<u32>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub models: Vec<String>,
}
```

### SkillManifestEntry 扩展（skills.json 条目）

```rust
pub struct SkillManifestEntry {
    // 现有字段不变
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub license: Option<String>,
    pub updated_at: Option<String>,
    pub checksum: Option<Checksum>,
    // 新增对齐 Skillbase 字段
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub trigger: Option<TriggerInfo>,
    #[serde(default)]
    pub security: Option<SecurityInfo>,
}
```

## 前端变更

### TypeScript 类型（types/index.ts）

新增 `TriggerInfo`、`SecurityInfo`、`CompatibilityInfo` 接口，`SkillMeta` 扩展对应的可选字段。

### SkillCard.vue

- `author` → name 下方显示 `by xxx` 小标签
- `language` → 语言徽章（如 `en`、`zh`）
- `trigger.tags` → 与现有 tags 合并展示（去重）

### SkillPreviewModal.vue

frontmatter 详情区新增分区：
- **Trigger**: description + tags + file_patterns
- **Security**: permissions 列表（危险权限如 `bash:execute` 用警告色）
- **Compatibility**: min_context_tokens + requires + models
- **Dependencies**: 列出依赖 skill 及版本范围
- **Author / Language / Repository**: 简洁一行

### 不改动的组件

- `SkillCompareTable.vue` — 信息密度已高，不加新字段
- `SettingsView.vue` — 与元数据无关
- `GlobalView.vue` / `ProjectDetailView.vue` — 只消费子组件

## 解析流程变更

### parse_skill_frontmatter 重写

```rust
pub fn parse_skill_frontmatter(skill_dir: &str) -> Result<SkillMeta, String> {
    let skill_md_path = Path::new(skill_dir).join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Read SKILL.md: {}", e))?;

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("No YAML frontmatter found".to_string());
    }
    let rest = &trimmed[3..];
    let end = rest.find("---").ok_or("Unterminated YAML frontmatter")?;
    let yaml_str = &rest[..end];

    serde_yaml::from_str(yaml_str)
        .map_err(|e| format!("Parse frontmatter: {}", e))
}
```

### build_remote_skill_meta / build_local_skill_meta 简化

解析结果即为 `SkillMeta`，只需补充运行时字段：
- `path` — 技能在仓库中的相对路径
- `checksum` — 目录聚合 SHA256
- `install_status` — 安装状态
- `source_repo_id` — 来源仓库

## 依赖变更

`Cargo.toml` 新增：
```toml
serde_yaml = "0.9"
```

## 向后兼容

- 旧 SKILL.md（无 author/trigger 等字段）→ `#[serde(default)]` 自动填充 None/空值
- 旧 skills.json 格式 → `SkillManifestEntry` 新字段都是 `Option` + `#[serde(default)]`
- 前端新字段都是 optional → 旧数据无这些字段，UI 不渲染
