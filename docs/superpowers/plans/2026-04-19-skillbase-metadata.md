# Skillbase 元数据解析与展示 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 扩展 SPM Manager 完整解析和展示 Skillbase 标准的 SKILL.md frontmatter 字段。

**Architecture:** 引入 `serde_yaml` 替代手写 YAML 解析器，统一使用 `SkillMeta` 作为唯一数据模型。前端扩展现有 SkillCard 和 SkillPreviewModal 组件展示新字段。

**Tech Stack:** Rust + serde_yaml, Vue 3 + TypeScript + Naive UI

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src-tauri/Cargo.toml` | 添加 serde_yaml 依赖 |
| Modify | `src-tauri/src/models/skill.rs` | 扩展 SkillMeta，新增 TriggerInfo/SecurityInfo/CompatibilityInfo，删除 SkillFrontmatter |
| Modify | `src-tauri/src/services/skill_service.rs` | 重写 parse_skill_frontmatter 用 serde_yaml，简化 build_remote/local_skill_meta，删除旧解析函数 |
| Modify | `src/types/index.ts` | 新增接口，扩展 SkillMeta |
| Modify | `src/components/common/SkillCard.vue` | 展示 author、language |
| Modify | `src/components/common/SkillPreviewModal.vue` | 展示 trigger、security、compatibility、dependencies 等完整元数据 |
| Modify | `src/i18n/locales/zh-CN.json` | 新增翻译键 |
| Modify | `src/i18n/locales/en-US.json` | 新增翻译键 |

---

### Task 1: 添加 serde_yaml 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 serde_yaml 到 Cargo.toml**

在 `[dependencies]` 末尾、`reqwest` 行之后添加：

```toml
serde_yaml = "0.9"
```

- [ ] **Step 2: 验证依赖编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add serde_yaml dependency for SKILL.md frontmatter parsing"
```

---

### Task 2: 扩展 Rust 数据模型

**Files:**
- Modify: `src-tauri/src/models/skill.rs`

- [ ] **Step 1: 添加 use 声明**

在文件顶部 `use serde::{Deserialize, Serialize};` 之后添加：

```rust
use std::collections::HashMap;
```

- [ ] **Step 2: 添加 TriggerInfo 结构体**

在 `SkillMeta` 结构体之前插入：

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TriggerInfo {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecurityInfo {
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CompatibilityInfo {
    #[serde(default)]
    pub min_context_tokens: Option<u32>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub models: Vec<String>,
}
```

- [ ] **Step 3: 重写 SkillMeta 结构体**

将整个 `SkillMeta` 结构体替换为：

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SkillMeta {
    // 通用元数据字段（来自 SKILL.md frontmatter 或 skills.json）
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

- [ ] **Step 4: 为 InstallStatus 添加 Default**

将 `InstallStatus` 枚举的 derive 改为：

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub enum InstallStatus {
    #[default]
    Unknown,
    Installed,
    Outdated,
    NotInstalled,
}
```

- [ ] **Step 5: 删除 SkillFrontmatter 结构体**

删除整个 `SkillFrontmatter` 结构体及其 derive 宏（约第 67-77 行）：

```rust
// 删除这段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillFrontmatter {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}
```

- [ ] **Step 5: 删除 SkillFrontmatter 结构体**

删除整个 `SkillFrontmatter` 结构体及其 derive 宏（约第 67-77 行）：

```rust
// 删除这段
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillFrontmatter {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}
```

- [ ] **Step 6: 扩展 SkillManifestEntry**

在 `SkillManifestEntry` 结构体末尾、`checksum` 字段之后添加新字段：

```rust
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub trigger: Option<TriggerInfo>,
    #[serde(default)]
    pub security: Option<SecurityInfo>,
```

完整的 `SkillManifestEntry` 应为：

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillManifestEntry {
    pub name: String,
    pub path: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub checksum: Option<Checksum>,
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

- [ ] **Step 7: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译失败——`skill_service.rs` 中引用了已删除的 `SkillFrontmatter`。这是预期的，Task 3 会修复。

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/models/skill.rs
git commit -m "feat: extend SkillMeta with Skillbase fields, add TriggerInfo/SecurityInfo/CompatibilityInfo"
```

---

### Task 3: 重写解析逻辑

**Files:**
- Modify: `src-tauri/src/services/skill_service.rs`

- [ ] **Step 1: 更新 use 声明**

将顶部的 `use crate::models::skill::{...};` 替换为（删除 `SkillFrontmatter`，添加 `TriggerInfo`、`SecurityInfo`）：

```rust
use crate::models::skill::{
    FileDiff, FileDiffStatus, InstallStatus, SkillDiff, SkillManifestEntry,
    SkillMeta, SkillsManifest, TriggerInfo, SecurityInfo,
};
```

- [ ] **Step 2: 重写 parse_skill_frontmatter 函数**

将整个 `parse_skill_frontmatter` 函数替换为：

```rust
/// Parse SKILL.md YAML frontmatter from a skill directory using serde_yaml
pub fn parse_skill_frontmatter(skill_dir: &str) -> Result<SkillMeta, String> {
    let skill_md_path = Path::new(skill_dir).join("SKILL.md");
    let content =
        std::fs::read_to_string(&skill_md_path).map_err(|e| format!("Read SKILL.md: {}", e))?;

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("No YAML frontmatter found".to_string());
    }

    let rest = &trimmed[3..];
    let end = rest
        .find("---")
        .ok_or("Unterminated YAML frontmatter")?;
    let yaml_str = &rest[..end];

    serde_yaml::from_str(yaml_str).map_err(|e| format!("Parse frontmatter: {}", e))
}
```

- [ ] **Step 3: 删除旧解析函数**

删除以下两个函数（整个函数体）：
- `parse_frontmatter_from_string`
- `parse_simple_yaml_frontmatter`

- [ ] **Step 4: 简化 scan_skills_from_dirs**

将 `scan_skills_from_dirs` 函数中 `let fm = ...` 之后的字段提取逻辑简化。将整个 `entries.push(SkillManifestEntry { ... })` 块替换为：

```rust
        entries.push(SkillManifestEntry {
            name,
            path,
            version: fm.as_ref().map(|f| f.version.clone()).unwrap_or_default(),
            description: fm.as_ref().map(|f| f.description.clone()).unwrap_or_default(),
            tags: fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default(),
            updated_at: fm.as_ref().and_then(|f| f.updated_at.clone()),
            checksum: None,
            author: fm.as_ref().and_then(|f| f.author.clone()),
            language: fm.as_ref().and_then(|f| f.language.clone()),
            trigger: fm.as_ref().and_then(|f| f.trigger.clone()),
            security: fm.as_ref().and_then(|f| f.security.clone()),
            license: fm.as_ref().and_then(|f| f.license.clone()),
        });
```

注意：原代码中 `license` 没有从 frontmatter 提取，此处补充。

- [ ] **Step 5: 简化 build_remote_skill_meta**

将 `build_remote_skill_meta` 函数中 `let fm = ...` 之后的字段提取逻辑和 `Ok(SkillMeta { ... })` 替换为：

```rust
    let mut meta = fm.unwrap_or_else(|| SkillMeta {
        name: entry.name.clone(),
        version: entry.version.clone(),
        description: entry.description.clone(),
        tags: entry.tags.clone(),
        path: entry.path.clone(),
        ..Default::default()
    });

    meta.path = entry.path.clone();
    meta.checksum = crate::services::hash_service::aggregate_sha256(&skill_dir).ok().or_else(|| entry.checksum.clone());
    meta.install_status = Some(InstallStatus::NotInstalled);
    meta.source_repo_id = repo_id.map(|s| s.to_string());

    // Fill from manifest entry if frontmatter didn't provide these
    if meta.license.is_none() {
        // no license from frontmatter — leave as None
    }
    if meta.updated_at.is_none() {
        meta.updated_at = entry.updated_at.clone();
    }

    Ok(meta)
```

- [ ] **Step 6: 简化 build_local_skill_meta**

将 `build_local_skill_meta` 函数中从 `let fm = ...` 到函数结尾的整个逻辑替换为：

```rust
    if !skill_dir.exists() {
        return Ok(SkillMeta {
            name: skill_name.to_string(),
            path: skill_name.to_string(),
            install_status: Some(InstallStatus::NotInstalled),
            ..Default::default()
        });
    }

    let fm = parse_skill_frontmatter(skill_dir.to_string_lossy().as_ref()).ok();
    let checksum = crate::services::hash_service::aggregate_sha256(&skill_dir).ok();

    let mut meta = fm.unwrap_or_else(|| SkillMeta {
        name: skill_name.to_string(),
        path: skill_name.to_string(),
        ..Default::default()
    });

    meta.path = skill_name.to_string();
    meta.checksum = checksum;

    // Use mtime as fallback for updated_at
    if meta.updated_at.is_none() {
        meta.updated_at = skill_dir
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Local> = t.into();
                dt.to_rfc3339()
            });
    }

    Ok(meta)
```

- [ ] **Step 7: 验证编译**

Run: `cd src-tauri && cargo check`
Expected: 编译成功，无错误

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/skill_service.rs src-tauri/src/models/skill.rs
git commit -m "feat: rewrite SKILL.md parsing with serde_yaml, unify SkillMeta model"
```

---

### Task 4: 扩展 TypeScript 类型

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: 添加新接口**

在 `Checksum` 接口之后、`SkillMeta` 接口之前插入：

```typescript
export interface TriggerInfo {
  description: string
  tags: string[]
  file_patterns: string[]
  priority?: number
}

export interface SecurityInfo {
  permissions: string[]
}

export interface CompatibilityInfo {
  min_context_tokens?: number
  requires: string[]
  models: string[]
}
```

- [ ] **Step 2: 扩展 SkillMeta 接口**

在 `SkillMeta` 接口中，在 `source_repo_id` 之前添加新字段：

```typescript
export interface SkillMeta {
  name: string
  version: string
  description: string
  tags: string[]
  path: string
  license?: string
  updated_at?: string
  checksum?: Checksum
  files?: FileEntry[]
  install_status?: InstallStatus
  source_repo_id?: string
  author?: string
  language?: string
  trigger?: TriggerInfo
  security?: SecurityInfo
  compatibility?: CompatibilityInfo
  dependencies?: Record<string, string>
  repository?: string
}
```

- [ ] **Step 3: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: extend TypeScript SkillMeta type with Skillbase fields"
```

---

### Task 5: 更新 SkillCard.vue

**Files:**
- Modify: `src/components/common/SkillCard.vue`

- [ ] **Step 1: 添加 author 和 language 展示**

在 `<template>` 中的 `skill-desc` div 之前（即 `</div>` 关闭 `skill-title-row` 之后）添加：

```html
      <div v-if="skill.author || skill.language" class="skill-meta-line">
        <NText v-if="skill.author" depth="3" style="font-size: 12px">
          by {{ skill.author }}
        </NText>
        <NTag v-if="skill.language" size="tiny" round :bordered="false" type="info" style="margin-left: 6px">
          {{ skill.language }}
        </NTag>
      </div>
```

- [ ] **Step 2: 合并 trigger.tags 到 tags 展示**

将 `skill-tags` 区域的 `v-for` 替换为合并逻辑。在 `<script setup>` 中添加计算属性：

```typescript
const displayTags = computed(() => {
  const allTags = [...skill.tags]
  if (skill.trigger?.tags) {
    for (const t of skill.trigger.tags) {
      if (!allTags.includes(t)) allTags.push(t)
    }
  }
  return allTags.slice(0, 6)
})
```

然后在 `<script setup>` 顶部添加 `import { computed } from 'vue'`（如果还没有的话）。

在 `<template>` 中将：

```html
<NTag v-for="tag in skill.tags.slice(0, 4)" :key="tag" size="tiny" round type="info">
```

替换为：

```html
<NTag v-for="tag in displayTags" :key="tag" size="tiny" round type="info">
```

- [ ] **Step 3: 添加样式**

在 `<style scoped>` 中 `.skill-desc` 之后添加：

```css
.skill-meta-line {
  margin-top: 4px;
  display: flex;
  align-items: center;
}
```

- [ ] **Step 4: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src/components/common/SkillCard.vue
git commit -m "feat: display author, language and trigger.tags in SkillCard"
```

---

### Task 6: 更新 SkillPreviewModal.vue

**Files:**
- Modify: `src/components/common/SkillPreviewModal.vue`

- [ ] **Step 1: 扩展 FrontmatterData 接口**

将 `FrontmatterData` 接口替换为：

```typescript
interface FrontmatterData {
  name: string
  version: string
  description: string
  tags: string[]
  license: string
  updated_at: string
  author: string
  language: string
  repository: string
  trigger: {
    description: string
    tags: string[]
    file_patterns: string[]
    priority?: number
  } | null
  security: {
    permissions: string[]
  } | null
  compatibility: {
    min_context_tokens?: number
    requires: string[]
    models: string[]
  } | null
  dependencies: Record<string, string> | null
  extra: Record<string, string>
}
```

- [ ] **Step 2: 重写 parseFrontmatter 函数**

将整个 `parseFrontmatter` 函数替换为（使用简单的 YAML 行解析，但扩展字段提取）：

```typescript
function parseFrontmatter(content: string): { fm: FrontmatterData | null; body: string } {
  const trimmed = content.trimStart()
  if (!trimmed.startsWith('---')) return { fm: null, body: content }

  const rest = trimmed.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return { fm: null, body: content }

  const yaml = rest.slice(0, end)
  const body = rest.slice(end + 4).trimStart()

  const fm: FrontmatterData = {
    name: '',
    version: '',
    description: '',
    tags: [],
    license: '',
    updated_at: '',
    author: '',
    language: '',
    repository: '',
    trigger: null,
    security: null,
    compatibility: null,
    dependencies: null,
    extra: {},
  }

  let currentSection = ''
  let currentObj: Record<string, any> | null = null

  for (const line of yaml.split('\n')) {
    const trimmedLine = line.trim()

    if (!trimmedLine || trimmedLine.startsWith('#')) continue

    // Detect nested section headers (indented objects)
    if (!line.startsWith(' ') && !line.startsWith('\t')) {
      currentSection = ''
      currentObj = null
    }

    const colon = trimmedLine.indexOf(':')
    if (colon === -1) continue
    const key = trimmedLine.slice(0, colon).trim()
    let val = trimmedLine.slice(colon + 1).trim()

    // Top-level keys
    if (!line.startsWith(' ') && !line.startsWith('\t')) {
      switch (key) {
        case 'name': fm.name = val; break
        case 'version': fm.version = val; break
        case 'description': fm.description = val; break
        case 'license': fm.license = val; break
        case 'updated_at': fm.updated_at = val; break
        case 'author': fm.author = val; break
        case 'language': fm.language = val; break
        case 'repository': fm.repository = val; break
        case 'tags':
          fm.tags = parseYamlArray(val)
          break
        case 'trigger':
          currentSection = 'trigger'
          currentObj = { description: '', tags: [], file_patterns: [] }
          fm.trigger = currentObj as any
          break
        case 'security':
          currentSection = 'security'
          currentObj = { permissions: [] }
          fm.security = currentObj as any
          break
        case 'compatibility':
          currentSection = 'compatibility'
          currentObj = { requires: [], models: [] }
          fm.compatibility = currentObj as any
          break
        case 'dependencies':
          currentSection = 'dependencies'
          fm.dependencies = {}
          break
        default: fm.extra[key] = val
      }
      continue
    }

    // Nested keys (indented under trigger/security/compatibility/dependencies)
    if (currentSection === 'trigger' && currentObj) {
      if (key === 'description') currentObj.description = val
      else if (key === 'tags') currentObj.tags = parseYamlArray(val)
      else if (key === 'file_patterns') currentObj.file_patterns = parseYamlArray(val)
      else if (key === 'priority') currentObj.priority = parseInt(val, 10) || undefined
    } else if (currentSection === 'security' && currentObj) {
      if (key === 'permissions') currentObj.permissions = parseYamlArray(val)
    } else if (currentSection === 'compatibility' && currentObj) {
      if (key === 'min_context_tokens') currentObj.min_context_tokens = parseInt(val, 10) || undefined
      else if (key === 'requires') currentObj.requires = parseYamlArray(val)
      else if (key === 'models') currentObj.models = parseYamlArray(val)
    } else if (currentSection === 'dependencies' && fm.dependencies) {
      // dependency entries: "skill-name: ^1.0.0"
      fm.dependencies[key] = val
    }
  }

  return { fm, body }
}

function parseYamlArray(val: string): string[] {
  if (val.startsWith('[') && val.endsWith(']')) {
    return val.slice(1, -1).split(',').map(s => s.trim()).filter(Boolean)
  }
  return []
}
```

- [ ] **Step 3: 扩展 frontmatter 展示模板**

在 `<template>` 的 `fm-card` 区域中，在 `extra` 的 `NDescriptionsItem` 之前（即 `updated_at` 之后、`extra` 之前）添加新的展示区块：

```html
                <NDescriptionsItem v-if="frontmatter.author" :label="t('preview.author')">
                  {{ frontmatter.author }}
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.language" :label="t('preview.language')">
                  {{ frontmatter.language }}
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.repository" :label="t('preview.repository')">
                  {{ frontmatter.repository }}
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.trigger" :label="t('preview.trigger')">
                  <div>
                    <NText>{{ frontmatter.trigger.description }}</NText>
                    <div v-if="frontmatter.trigger.tags.length > 0" style="margin-top: 4px">
                      <NTag v-for="tag in frontmatter.trigger.tags" :key="'t-'+tag" size="small" round type="info" style="margin-right: 4px">
                        {{ tag }}
                      </NTag>
                    </div>
                    <div v-if="frontmatter.trigger.file_patterns.length > 0" style="margin-top: 4px; font-size: 12px">
                      <NText depth="3">{{ frontmatter.trigger.file_patterns.join(', ') }}</NText>
                    </div>
                  </div>
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.security && frontmatter.security.permissions.length > 0" :label="t('preview.security')">
                  <NTag
                    v-for="perm in frontmatter.security.permissions"
                    :key="perm"
                    size="small"
                    round
                    :type="perm.includes('bash') || perm.includes('delete') ? 'warning' : 'default'"
                    style="margin-right: 4px"
                  >
                    {{ perm }}
                  </NTag>
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.compatibility" :label="t('preview.compatibility')">
                  <div style="font-size: 13px">
                    <div v-if="frontmatter.compatibility.min_context_tokens">
                      min_context_tokens: {{ frontmatter.compatibility.min_context_tokens }}
                    </div>
                    <div v-if="frontmatter.compatibility.requires.length > 0">
                      requires: {{ frontmatter.compatibility.requires.join(', ') }}
                    </div>
                    <div v-if="frontmatter.compatibility.models.length > 0">
                      models: {{ frontmatter.compatibility.models.join(', ') }}
                    </div>
                  </div>
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.dependencies && Object.keys(frontmatter.dependencies).length > 0" :label="t('preview.dependencies')">
                  <div style="font-size: 13px">
                    <div v-for="(ver, dep) in frontmatter.dependencies" :key="dep">
                      <NText code>{{ dep }}</NText> <NText depth="3">{{ ver }}</NText>
                    </div>
                  </div>
                </NDescriptionsItem>
```

- [ ] **Step 4: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 5: Commit**

```bash
git add src/components/common/SkillPreviewModal.vue
git commit -m "feat: display full Skillbase metadata in SkillPreviewModal"
```

---

### Task 7: 添加 i18n 翻译

**Files:**
- Modify: `src/i18n/locales/zh-CN.json`
- Modify: `src/i18n/locales/en-US.json`

- [ ] **Step 1: 添加中文翻译键**

在 `zh-CN.json` 的 `preview` 区域中添加新键。找到现有的 `preview.updatedAt` 行之后，添加：

```json
      "author": "作者",
      "language": "语言",
      "repository": "仓库",
      "trigger": "触发条件",
      "security": "安全权限",
      "compatibility": "兼容性",
      "dependencies": "依赖"
```

- [ ] **Step 2: 添加英文翻译键**

在 `en-US.json` 的 `preview` 区域中同样的位置添加：

```json
      "author": "Author",
      "language": "Language",
      "repository": "Repository",
      "trigger": "Trigger",
      "security": "Security",
      "compatibility": "Compatibility",
      "dependencies": "Dependencies"
```

- [ ] **Step 3: 验证前端编译**

Run: `npm run build`
Expected: 编译成功

- [ ] **Step 4: Commit**

```bash
git add src/i18n/locales/zh-CN.json src/i18n/locales/en-US.json
git commit -m "feat: add i18n keys for Skillbase metadata fields"
```

---

### Task 8: 集成构建验证

**Files:** 无变更

- [ ] **Step 1: 完整前端构建**

Run: `npm run build`
Expected: vue-tsc 类型检查 + vite build 成功

- [ ] **Step 2: Rust 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译成功

- [ ] **Step 3: 完整 Tauri 开发构建**

Run: `npm run tauri build`
Expected: 构建成功，生成安装包

- [ ] **Step 4: 手动功能验证**

启动 `npm run tauri dev`，验证：
1. 应用正常启动，无崩溃
2. 同步远程仓库，skill 列表正常加载
3. 点击 skill 预览，SKILL.md frontmatter 正确展示（包括新增的 author、trigger、security 等字段）
4. SkillCard 卡片正确显示 author 和 language
5. 旧的 SKILL.md（无新字段）不报错，新字段区域不显示
