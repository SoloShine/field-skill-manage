<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { NCard, NCollapse, NCollapseItem, NCode, NTag, NTable, NAlert, NH2, NP, NText, NButton } from 'naive-ui'
import { open } from '@tauri-apps/plugin-shell'

const { t } = useI18n()

const requiredFields = [
  { field: 'name', format: '^[a-z0-9][a-z0-9-]*$', descKey: 'guide.frontmatter.rows.name' },
  { field: 'version', format: 'x.y.z (semver)', descKey: 'guide.frontmatter.rows.version' },
  { field: 'author', format: 'string', descKey: 'guide.frontmatter.rows.author' },
  { field: 'license', format: 'SPDX', descKey: 'guide.frontmatter.rows.license' },
  { field: 'description', format: 'string', descKey: 'guide.frontmatter.rows.description' },
]

const optionalFields = [
  { field: 'language', format: 'string', descKey: 'guide.frontmatter.rows.language' },
  { field: 'repository', format: 'URL', descKey: 'guide.frontmatter.rows.repository' },
]

const xmlTags = [
  { tag: '<context>', descKey: 'guide.body.tags.context' },
  { tag: '<instructions>', descKey: 'guide.body.tags.instructions' },
  { tag: '<examples>', descKey: 'guide.body.tags.examples' },
  { tag: '<guidelines>', descKey: 'guide.body.tags.guidelines' },
  { tag: '<verification>', descKey: 'guide.body.tags.verification' },
  { tag: t('guide.body.tags.custom'), descKey: 'guide.body.tags.custom' },
]

const permissions = [
  { key: 'file:read', labelKey: 'guide.security.permissions.fileRead' },
  { key: 'file:write', labelKey: 'guide.security.permissions.fileWrite' },
  { key: 'file:delete', labelKey: 'guide.security.permissions.fileDelete' },
  { key: 'bash:execute', labelKey: 'guide.security.permissions.bashExecute' },
  { key: 'network:none', labelKey: 'guide.security.permissions.networkNone' },
  { key: 'network:allowlist', labelKey: 'guide.security.permissions.networkAllowlist' },
  { key: 'tool:web_search', labelKey: 'guide.security.permissions.toolWebSearch' },
  { key: 'tool:*', labelKey: 'guide.security.permissions.toolAll' },
]

const tips = [
  { key: 'specific', icon: '1' },
  { key: 'positive', icon: '2' },
  { key: 'motivation', icon: '3' },
  { key: 'examples', icon: '4' },
  { key: 'concise', icon: '5' },
]
</script>

<template>
  <div class="guide-view">
    <div class="page-header">
      <h1>{{ t('guide.title') }}</h1>
      <p class="page-subtitle">{{ t('guide.subtitle') }} <NButton text type="primary" size="small" @click="open('https://skillbase.space/')">skillbase.space</NButton></p>
    </div>

    <div class="guide-content">
      <!-- Directory Structure -->
      <NCard :title="t('guide.structure.title')" size="small" class="guide-card">
        <NP>{{ t('guide.structure.desc') }}</NP>
        <pre class="code-block"><code>{{ t('guide.structure.tree') }}</code></pre>
      </NCard>

      <!-- Frontmatter Fields -->
      <NCard :title="t('guide.frontmatter.title')" size="small" class="guide-card">
        <NP>{{ t('guide.frontmatter.desc') }}</NP>

        <NH2 prefix="bar" style="margin: 16px 0 8px; font-size: 16px;">
          {{ t('guide.frontmatter.required') }}
        </NH2>
        <NTable :bordered="false" :single-line="false" size="small" striped>
          <thead>
            <tr>
              <th>{{ t('guide.frontmatter.field') }}</th>
              <th>{{ t('guide.frontmatter.format') }}</th>
              <th>{{ t('guide.frontmatter.description') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in requiredFields" :key="row.field">
              <td><NCode code:inline>{{ row.field }}</NCode></td>
              <td><NText depth="3">{{ row.format }}</NText></td>
              <td>{{ t(row.descKey) }}</td>
            </tr>
          </tbody>
        </NTable>

        <NH2 prefix="bar" style="margin: 16px 0 8px; font-size: 16px;">
          {{ t('guide.frontmatter.optional') }}
        </NH2>
        <NTable :bordered="false" :single-line="false" size="small" striped>
          <thead>
            <tr>
              <th>{{ t('guide.frontmatter.field') }}</th>
              <th>{{ t('guide.frontmatter.format') }}</th>
              <th>{{ t('guide.frontmatter.description') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in optionalFields" :key="row.field">
              <td><NCode code:inline>{{ row.field }}</NCode></td>
              <td><NText depth="3">{{ row.format }}</NText></td>
              <td>{{ t(row.descKey) }}</td>
            </tr>
          </tbody>
        </NTable>
      </NCard>

      <!-- Trigger Configuration -->
      <NCard :title="t('guide.trigger.title')" size="small" class="guide-card">
        <NP>{{ t('guide.trigger.desc') }}</NP>
        <NCollapse>
          <NCollapseItem title="trigger.description" name="trigger-desc">
            <NTag type="info" size="small" style="margin-bottom: 8px;">required</NTag>
            <NP>{{ t('guide.trigger.triggerDesc') }}</NP>
          </NCollapseItem>
          <NCollapseItem title="trigger.tags" name="trigger-tags">
            <NTag type="info" size="small" style="margin-bottom: 8px;">required</NTag>
            <NP>{{ t('guide.trigger.tags') }}</NP>
          </NCollapseItem>
          <NCollapseItem title="trigger.file_patterns" name="trigger-patterns">
            <NTag size="small" style="margin-bottom: 8px;">optional</NTag>
            <NP>{{ t('guide.trigger.filePatterns') }}</NP>
          </NCollapseItem>
          <NCollapseItem title="trigger.priority" name="trigger-priority">
            <NTag size="small" style="margin-bottom: 8px;">optional</NTag>
            <NP>{{ t('guide.trigger.priority') }}</NP>
          </NCollapseItem>
        </NCollapse>
      </NCard>

      <!-- Security Permissions -->
      <NCard :title="t('guide.security.title')" size="small" class="guide-card">
        <NP>{{ t('guide.security.desc') }}</NP>
        <div class="permissions-grid">
          <div v-for="p in permissions" :key="p.key" class="permission-item">
            <NCode code:inline>{{ p.key }}</NCode>
            <NText depth="3">{{ t(p.labelKey) }}</NText>
          </div>
        </div>
      </NCard>

      <!-- Dependencies -->
      <NCard :title="t('guide.dependencies.title')" size="small" class="guide-card">
        <NP>{{ t('guide.dependencies.desc') }}</NP>
        <ul class="info-list">
          <li><NCode code:inline>dependencies</NCode> — {{ t('guide.dependencies.deps') }}</li>
          <li><NCode code:inline>compatibility</NCode> — {{ t('guide.dependencies.compatibility') }}</li>
        </ul>
      </NCard>

      <!-- Body Writing Guide -->
      <NCard :title="t('guide.body.title')" size="small" class="guide-card">
        <NP>{{ t('guide.body.desc') }}</NP>
        <NTable :bordered="false" :single-line="false" size="small" striped>
          <thead>
            <tr>
              <th>{{ t('guide.body.tag') }}</th>
              <th>{{ t('guide.body.purpose') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in xmlTags" :key="row.tag">
              <td><NCode code:inline>{{ row.tag }}</NCode></td>
              <td>{{ t(row.descKey) }}</td>
            </tr>
          </tbody>
        </NTable>
      </NCard>

      <!-- Writing Tips -->
      <NCard :title="t('guide.tips.title')" size="small" class="guide-card">
        <NP>{{ t('guide.tips.desc') }}</NP>
        <div class="tips-grid">
          <div v-for="tip in tips" :key="tip.key" class="tip-item">
            <span class="tip-number">{{ tip.icon }}</span>
            <span>{{ t(`guide.tips.${tip.key}`) }}</span>
          </div>
        </div>
      </NCard>

      <!-- Complete Example -->
      <NCard :title="t('guide.example.title')" size="small" class="guide-card">
        <NP>{{ t('guide.example.desc') }}</NP>
        <pre class="code-block code-block-lg"><code>{{ t('guide.example.code') }}</code></pre>
      </NCard>

      <!-- Validation & Publishing -->
      <NCard :title="t('guide.validate.title')" size="small" class="guide-card">
        <NP>{{ t('guide.validate.desc') }}</NP>
        <NCollapse>
          <NCollapseItem :title="t('guide.validate.validateCmd')" name="validate">
            <NAlert type="info" :bordered="false">
              {{ t('guide.validate.validateDesc') }}
            </NAlert>
          </NCollapseItem>
          <NCollapseItem :title="t('guide.validate.linkCmd')" name="link">
            <NP>{{ t('guide.validate.linkDesc') }}</NP>
            <NH2 prefix="bar" style="margin: 12px 0 8px; font-size: 15px;">
              {{ t('guide.validate.repoStructure') }}
            </NH2>
            <NP><strong>{{ t('guide.validate.manifestFirst') }}</strong></NP>
            <NP>{{ t('guide.validate.manifestDesc') }}</NP>
            <pre class="code-block"><code>{{ t('guide.validate.manifestTree') }}</code></pre>
            <NP style="margin-top: 12px;"><strong>{{ t('guide.validate.scanFallback') }}</strong></NP>
            <NP>{{ t('guide.validate.scanDesc') }}</NP>
            <pre class="code-block"><code>{{ t('guide.validate.scanTree') }}</code></pre>
          </NCollapseItem>
          <NCollapseItem :title="t('guide.validate.publishCmd')" name="publish">
            <NAlert type="info" :bordered="false">
              {{ t('guide.validate.publishDesc') }}
            </NAlert>
          </NCollapseItem>
        </NCollapse>
      </NCard>
    </div>
  </div>
</template>

<style scoped>
.guide-view {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.page-header {
  flex-shrink: 0;
  padding: 0 0 12px;
  border-bottom: 1px solid var(--color-border);
}
.page-header h1 {
  font-size: 22px;
  font-weight: 600;
  margin: 0 0 2px;
}
.page-subtitle {
  color: var(--color-text-muted);
  font-size: 13px;
  margin: 0;
}
.guide-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.guide-card :deep(.n-card-header) {
  font-size: 16px;
  font-weight: 600;
}
.code-block {
  background: var(--color-bg-tertiary);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-sm);
  padding: 12px 16px;
  font-family: var(--font-mono, 'JetBrains Mono', monospace);
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  margin: 8px 0;
}
.code-block-lg {
  max-height: 500px;
  overflow-y: auto;
}
.permissions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 8px;
  margin-top: 8px;
}
.permission-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}
.info-list {
  margin: 8px 0 0;
  padding-left: 20px;
}
.info-list li {
  margin-bottom: 6px;
  line-height: 1.5;
}
.tips-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 8px;
}
.tip-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  line-height: 1.5;
}
.tip-number {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--color-accent, #3b82f6);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}
</style>
