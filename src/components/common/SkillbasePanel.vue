<script setup lang="ts">
import { computed } from 'vue'
import { NButton, NTag, NText } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import type { SkillbaseResolution, DependencyStatus } from '@/types'

const { t } = useI18n()

const props = defineProps<{
  resolution: SkillbaseResolution
  syncing: boolean
}>()

const emit = defineEmits<{
  sync: []
}>()

const hasUnsatisfied = computed(() => {
  return props.resolution.missingCount + props.resolution.mismatchCount + props.resolution.outdatedCount > 0
})

function statusType(status: DependencyStatus): 'success' | 'warning' | 'error' | 'info' | 'default' {
  switch (status) {
    case 'Satisfied': return 'success'
    case 'Missing': return 'error'
    case 'VersionMismatch': return 'warning'
    case 'Outdated': return 'info'
  }
}

function statusLabel(status: DependencyStatus): string {
  switch (status) {
    case 'Satisfied': return t('skillbase.satisfied')
    case 'Missing': return t('skillbase.missing')
    case 'VersionMismatch': return t('skillbase.mismatch')
    case 'Outdated': return t('skillbase.outdated')
  }
}
</script>

<template>
  <div class="skillbase-panel">
    <div class="panel-header">
      <div class="panel-title">
        <NText strong>skillbase.json</NText>
        <NText depth="3" style="font-size: 12px; margin-left: 8px">
          {{ resolution.manifest.name }}
        </NText>
      </div>
      <div class="panel-stats">
        <NTag size="small" round type="success">
          {{ resolution.satisfiedCount }} {{ t('skillbase.satisfied') }}
        </NTag>
        <NTag v-if="resolution.missingCount > 0" size="small" round type="error">
          {{ resolution.missingCount }} {{ t('skillbase.missing') }}
        </NTag>
        <NTag v-if="resolution.mismatchCount > 0" size="small" round type="warning">
          {{ resolution.mismatchCount }} {{ t('skillbase.mismatch') }}
        </NTag>
        <NTag v-if="resolution.outdatedCount > 0" size="small" round type="warning">
          {{ resolution.outdatedCount }} {{ t('skillbase.outdated') }}
        </NTag>
      </div>
      <div class="panel-actions">
        <NButton
          v-if="hasUnsatisfied"
          size="small"
          type="primary"
          :loading="syncing"
          @click="emit('sync')"
        >
          {{ t('skillbase.syncDeps') }}
        </NButton>
      </div>
    </div>
    <div class="dep-list">
      <div
        v-for="dep in resolution.dependencies"
        :key="dep.reference"
        class="dep-item"
      >
        <span class="dep-dot" :class="dep.status.toLowerCase()"></span>
        <span class="dep-ref">{{ dep.reference }}</span>
        <NText depth="3" style="font-size: 12px">{{ dep.versionRange }}</NText>
        <NTag size="tiny" round :type="statusType(dep.status)">
          {{ statusLabel(dep.status) }}
        </NTag>
        <NText v-if="dep.installed" depth="3" style="font-size: 11px; margin-left: auto">
          v{{ dep.installed.version || '?' }}
        </NText>
      </div>
    </div>
  </div>
</template>

<style scoped>
.skillbase-panel {
  border: 1px solid var(--color-border);
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 12px;
}
.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-secondary);
  flex-wrap: wrap;
}
.panel-title {
  display: flex;
  align-items: baseline;
  gap: 4px;
}
.panel-stats {
  display: flex;
  gap: 4px;
  margin-left: auto;
}
.panel-actions {
  display: flex;
  gap: 4px;
}
.dep-list {
  padding: 4px 0;
  max-height: 200px;
  overflow-y: auto;
}
.dep-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px;
  font-size: 13px;
}
.dep-item:hover {
  background: var(--color-bg-hover);
}
.dep-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.dep-dot.satisfied { background: var(--color-status-same, #52c41a); }
.dep-dot.missing { background: var(--color-status-remote, #ff4d4f); }
.dep-dot.versionmismatch { background: var(--color-status-outdated, #faad14); }
.dep-dot.outdated { background: #1890ff; }
.dep-ref {
  font-family: var(--font-mono);
  font-weight: 500;
}
</style>
