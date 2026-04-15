---
name: release
description: Automate the full release workflow for this Tauri project. Use this skill whenever the user wants to publish a new version, create a release, bump version and deploy, or says things like "发版", "发布", "release", "publish a new version", "bump version and release". Also trigger when the user mentions version bumping combined with git push/tag operations.
---

# Release 发版流程

本 skill 自动化完整的发版流水线：版本号更新 → 构建验证 → 提交 → 推送 → 打标签 → 触发 CI。

## 版本号规范

### 版本号位置

项目版本号在以下三个文件中定义，发版时必须同步更新：

1. `package.json` → `"version": "x.y.z"`
2. `src-tauri/Cargo.toml` → `version = "x.y.z"`
3. `src-tauri/tauri.conf.json` → `"version": "x.y.z"`

### 版本号更新方式

使用脚本自动同步三处版本号：

```bash
# 语义化版本提升
npm run version:bump patch   # 0.1.0 → 0.1.1
npm run version:bump minor   # 0.1.0 → 0.2.0
npm run version:bump major   # 0.1.0 → 1.0.0

# 直接指定版本号
npm run version:bump 1.2.3
```

### 版本检测机制

应用内通过 GitHub Releases API 检测更新：
- Rust 端读取 `env!("CARGO_PKG_VERSION")` 作为当前版本（编译时从 Cargo.toml 注入）
- 优先查正式版 `/releases/latest`，404 时回退查 `/releases` 列表（含 prerelease）
- 仓库地址：`SoloShine/field-skill-manage`

## 输入参数

用户提供的版本参数，可以是：
- `patch` — 补丁版本（0.2.0 → 0.2.1）
- `minor` — 次版本（0.2.0 → 0.3.0）
- `major` — 主版本（0.2.0 → 1.0.0）
- 具体版本号，如 `0.3.0`

如果用户未提供参数，询问用户想要哪种类型的版本提升。

## 发版步骤

按顺序执行以下步骤。如果任何步骤失败，停止并向用户报告错误。

### 1. 前置检查

- 运行 `git status --porcelain` 确认工作区干净。如果有未提交的变更，停止并要求用户先提交或暂存。
- 运行 `git tag list` 检查已有标签，用于后续验证。

### 2. 更新版本号

```bash
npm run version:bump <arg>
```

更新后，从 `package.json` 读取确认新版本号，记为 `<version>`。

验证标签 `v<version>` 在已有标签列表中不存在。

### 3. 构建验证

```bash
npm run tauri build
```

确保项目能编译成功后再提交。如果失败，回退版本号变更并提示用户修复构建问题。

### 4. 提交版本变更

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to <version>"
```

### 5. 推送到 main

```bash
git push origin main
```

### 6. 创建并推送标签

```bash
git tag v<version>
git push origin v<version>
```

### 7. 汇总报告

向用户报告：
- 新版本号
- GitHub Actions 构建地址：`https://github.com/SoloShine/field-skill-manage/actions`
- 预期的 Release 页面：`https://github.com/SoloShine/field-skill-manage/releases/tag/v<version>`

## 失败回滚

**构建失败（步骤 3）：**
1. 恢复三个版本文件：`git checkout package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json`
2. 告知用户构建失败，建议修复后重试。

**推送失败（步骤 5 或 6）：**
1. 提交/标签仅在本地，远程状态未改变。
2. 报告错误，用户可重试推送或排查网络/认证问题。
