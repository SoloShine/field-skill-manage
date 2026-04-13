# CLAUDE.md - SPM Manager 项目规范

## 版本更新规范

### 版本号位置

项目版本号在以下三个文件中定义，发版时必须同步更新：

1. `package.json` → `"version": "x.y.z"`
2. `src-tauri/Cargo.toml` → `version = "x.y.z"`
3. `src-tauri/tauri.conf.json` → `"version": "x.y.z"`

### 更新方式

使用脚本自动同步三处版本号：

```bash
# 语义化版本提升
npm run version:bump patch   # 0.1.0 → 0.1.1
npm run version:bump minor   # 0.1.0 → 0.2.0
npm run version:bump major   # 0.1.0 → 1.0.0

# 直接指定版本号
npm run version:bump 1.2.3
```

### 发版流程

1. 执行 `npm run version:bump <type>` 更新版本号
2. 提交变更：`git commit -am "chore: bump version to x.y.z"`
3. 推送到远程：`git push origin main`
4. 在 GitHub 上创建 Release，tag 名与版本号一致（如 `0.1.1`）
5. Release 不要勾选 "Set as a pre-release"，除非确实是预发布版本

### 版本检测机制

应用内通过 GitHub Releases API 检测更新：
- Rust 端读取 `env!("CARGO_PKG_VERSION")` 作为当前版本（编译时从 Cargo.toml 注入）
- 优先查正式版 `/releases/latest`，404 时回退查 `/releases` 列表（含 prerelease）
- 仓库地址：`SoloShine/field-skill-manage`
