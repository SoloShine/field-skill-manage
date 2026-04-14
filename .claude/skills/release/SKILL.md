---
name: release
description: Automate the full release workflow for this Tauri project. Use this skill whenever the user wants to publish a new version, create a release, bump version and deploy, or says things like "发版", "发布", "release", "publish a new version", "bump version and release". Also trigger when the user mentions version bumping combined with git push/tag operations.
---

# Release Workflow

This skill automates the complete release pipeline: version bump → build verification → commit → push → tag → trigger CI.

## Input

The user provides a version argument, which can be:
- `patch` — bump patch version (0.2.0 → 0.2.1)
- `minor` — bump minor version (0.2.0 → 0.3.0)
- `major` — bump major version (0.2.0 → 1.0.0)
- A specific version like `0.3.0`

If no argument is provided, ask the user which type of bump they want.

## Steps

Execute these steps in order. If any step fails, stop and report the error to the user.

### 1. Pre-checks

- Run `git status --porcelain` to verify the working tree is clean. If there are uncommitted changes, stop and ask the user to commit or stash them first.
- Run `git tag list` to check existing tags. We'll validate against this after the version bump.

### 2. Bump version

```bash
npm run version:bump <arg>
```

After bumping, read the new version from `package.json` to confirm what version was set. Let's call this `<version>`.

Verify the tag `v<version>` does NOT already exist in the tag list from step 1.

### 3. Build verification

```bash
npm run tauri build
```

This ensures the project compiles successfully before committing. If it fails, the version bump should be reverted (restore the 3 version files) and the user should fix the build issue.

### 4. Commit version change

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: bump version to <version>"
```

### 5. Push to main

```bash
git push origin main
```

### 6. Create and push tag

```bash
git tag v<version>
git push origin v<version>
```

### 7. Summary

Report to the user:
- The new version number
- The GitHub Actions build URL: `https://github.com/SoloShine/field-skill-manage/actions`
- The expected Release page: `https://github.com/SoloShine/field-skill-manage/releases/tag/v<version>`

## Rollback on failure

If the build (step 3) fails:
1. Restore the three version files: `git checkout package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json`
2. Inform the user of the build failure and suggest they fix the issue before retrying.

If push (step 5 or 6) fails:
1. The commit/tag is local only, no remote state was changed.
2. Report the error — the user can retry the push or investigate the network/auth issue.
