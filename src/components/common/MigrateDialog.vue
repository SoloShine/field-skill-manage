<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { NModal, NButton, NSelect, NCheckbox, NRadioGroup, NRadio, NDivider, useMessage } from 'naive-ui'
import { useSkillStore } from '@/stores/skill'
import { useConfigStore } from '@/stores/config'
import SkillDiffViewer from '@/components/common/SkillDiffViewer.vue'
import type { ConflictResolution, SkillDiff } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const skillStore = useSkillStore()
const configStore = useConfigStore()
const message = useMessage()

const currentStep = ref(1)
const selectedAgentId = ref<string | null>(null)
const selectedSkills = ref<Set<string>>(new Set())
const conflictResolutions = ref<Record<string, ConflictResolution>>({})
const diffSkillName = ref<string | null>(null)
const diffData = ref<SkillDiff | null>(null)
const diffLoading = ref(false)

const sourceAgents = computed(() => {
  const activeId = configStore.config.active_agent_id
  const agents = configStore.config.agent_global_paths
  const result: { label: string; value: string }[] = []

  for (const [id, globalPath] of Object.entries(agents)) {
    if (id === activeId) continue
    const displayName = configStore.config.agent_display_names[id]
      || configStore.allAgents.find(a => a.id === id)?.display_name
      || id
    const pathPreview = skillStore.migrateScope === 'project'
      ? (configStore.config.agent_project_patterns[id] || '').replace('{project}', '...')
      : globalPath
    if (pathPreview) {
      result.push({
        label: `${displayName} → ${pathPreview}`,
        value: id,
      })
    }
  }

  return result
})

const agentOptions = computed(() =>
  sourceAgents.value.map(a => ({ label: a.label, value: a.value }))
)

const skills = computed(() => skillStore.scanResult?.skills ?? [])

const newSkills = computed(() =>
  skills.value.filter(s => s.conflict_status === 'NewTarget' && selectedSkills.value.has(s.name))
)

const conflictSkills = computed(() =>
  skills.value.filter(
    s => (s.conflict_status === 'DifferentVersion' || s.conflict_status === 'ContentDiffers')
      && selectedSkills.value.has(s.name)
  )
)

const summaryCounts = computed(() => {
  const conflictSkipped = conflictSkills.value.filter(
    s => conflictResolutions.value[s.name] !== 'Overwrite'
  ).length
  const conflictOverwritten = conflictSkills.value.filter(
    s => conflictResolutions.value[s.name] === 'Overwrite'
  ).length
  return {
    migrate: newSkills.value.length + conflictOverwritten,
    skip: conflictSkipped,
    overwrite: conflictOverwritten,
  }
})

// Reset all local state when dialog opens
watch(() => skillStore.migrateDialogVisible, (visible) => {
  if (visible) {
    currentStep.value = 1
    selectedAgentId.value = null
    selectedSkills.value = new Set()
    conflictResolutions.value = {}
    diffSkillName.value = null
    diffData.value = null
  }
})

watch(selectedAgentId, async (newId) => {
  if (!newId) return
  currentStep.value = 1
  skillStore.scanResult = null
  try {
    await skillStore.scanAgentSkills(newId)
    const selected = new Set<string>()
    for (const s of skills.value) {
      if (s.conflict_status !== 'SameContent') {
        selected.add(s.name)
      }
    }
    selectedSkills.value = selected
    const resolutions: Record<string, ConflictResolution> = {}
    for (const s of skills.value) {
      if (s.conflict_status === 'DifferentVersion' || s.conflict_status === 'ContentDiffers') {
        resolutions[s.name] = 'Skip'
      }
    }
    conflictResolutions.value = resolutions
    currentStep.value = 2
  } catch (e) {
    message.error(String(e))
  }
})

function toggleAll(checked: boolean) {
  if (checked) {
    selectedSkills.value = new Set(skills.value.map(s => s.name))
  } else {
    selectedSkills.value = new Set()
  }
}

function toggleSkill(name: string) {
  const next = new Set(selectedSkills.value)
  if (next.has(name)) {
    next.delete(name)
  } else {
    next.add(name)
  }
  selectedSkills.value = next
}

async function handleConfirm() {
  if (!selectedAgentId.value) return
  try {
    const result = await skillStore.migrateSkillsAction(
      selectedAgentId.value,
      Array.from(selectedSkills.value),
      conflictResolutions.value
    )
    if (result.failed.length === 0) {
      message.success(t('migration.migrateSuccess', {
        ok: result.migrated.length,
        skip: result.skipped.length,
      }))
    } else {
      message.warning(t('migration.migratePartial', {
        ok: result.migrated.length,
        fail: result.failed.length,
      }))
    }
    skillStore.closeMigrateDialog()
    if (skillStore.migrateScope === 'global') {
      await skillStore.loadGlobalSkills()
    } else if (skillStore.migrateProjectPath) {
      await skillStore.loadProjectDetail(skillStore.migrateProjectPath)
    }
  } catch (e) {
    message.error(t('migration.migrateFailed', { error: String(e) }))
  }
}

async function showDiff(skillName: string) {
  if (!selectedAgentId.value) return
  diffSkillName.value = skillName
  diffLoading.value = true
  try {
    diffData.value = await skillStore.loadMigrateSkillDiff(selectedAgentId.value, skillName)
  } catch {
    diffData.value = null
  } finally {
    diffLoading.value = false
  }
}

async function loadDiffFileContent(filePath: string) {
  if (!selectedAgentId.value || !diffSkillName.value) {
    return { local_content: undefined, remote_content: undefined } as import('@/types').DiffFileContent
  }
  return skillStore.loadMigrateDiffContent(selectedAgentId.value, diffSkillName.value, filePath)
}

function statusBadgeClass(status: string) {
  switch (status) {
    case 'NewTarget': return 'badge-new'
    case 'SameContent': return 'badge-same'
    case 'DifferentVersion': return 'badge-version'
    case 'ContentDiffers': return 'badge-content'
    default: return ''
  }
}

function statusBadgeText(status: string) {
  switch (status) {
    case 'NewTarget': return t('migration.statusNew')
    case 'SameContent': return t('migration.statusSame')
    case 'DifferentVersion': return t('migration.statusVersionDiff')
    case 'ContentDiffers': return t('migration.statusContentDiff')
    default: return status
  }
}
</script>

<template>
  <NModal
    :show="skillStore.migrateDialogVisible"
    :mask-closable="false"
    preset="card"
    :title="t('migration.title')"
    style="width: 640px; max-width: 90vw;"
    @update:show="(v: boolean) => { if (!v) skillStore.closeMigrateDialog() }"
    @close="skillStore.closeMigrateDialog()"
  >
    <div class="step-indicator">
      <div class="step" :class="{ active: currentStep >= 1, current: currentStep === 1 }">
        <span class="step-num">1</span>
        <span class="step-label">{{ t('migration.step1Title') }}</span>
      </div>
      <div class="step-line" :class="{ active: currentStep >= 2 }" />
      <div class="step" :class="{ active: currentStep >= 2, current: currentStep === 2 }">
        <span class="step-num">2</span>
        <span class="step-label">{{ t('migration.step2Title') }}</span>
      </div>
      <div class="step-line" :class="{ active: currentStep >= 3 }" />
      <div class="step" :class="{ active: currentStep >= 3, current: currentStep === 3 }">
        <span class="step-num">3</span>
        <span class="step-label">{{ t('migration.step3Title') }}</span>
      </div>
    </div>

    <div v-if="currentStep === 1" class="step-content">
      <div class="form-group">
        <label>{{ t('migration.sourceAgent') }}</label>
        <NSelect
          v-model:value="selectedAgentId"
          :options="agentOptions"
          :placeholder="t('migration.sourceAgentPlaceholder')"
        />
      </div>
      <div v-if="agentOptions.length === 0" class="empty-hint">
        {{ t('migration.noAgentsAvailable') }}
      </div>
      <div v-if="skills.length > 0" class="step-actions">
        <NButton type="primary" @click="currentStep = 2">
          {{ t('migration.next') }} ({{ skills.length }})
        </NButton>
      </div>
    </div>

    <div v-if="currentStep === 2" class="step-content">
      <div v-if="skillStore.scanResult" class="scan-info">
        <span>{{ t('migration.sourcePath') }}: {{ skillStore.scanResult.source_dir }}</span>
      </div>
      <div v-if="skills.length === 0" class="empty-hint">
        {{ t('migration.noSkillsFound') }}
      </div>
      <div v-else class="skill-list">
        <div class="skill-list-header">
          <NCheckbox
            :checked="selectedSkills.size === skills.length && skills.length > 0"
            :indeterminate="selectedSkills.size > 0 && selectedSkills.size < skills.length"
            @update:checked="toggleAll"
          >
            {{ t('migration.selectAll') }}
          </NCheckbox>
          <span class="selected-count">{{ t('migration.selected', { count: selectedSkills.size }) }}</span>
        </div>
        <div
          v-for="skill in skills"
          :key="skill.name"
          class="skill-row"
          :class="{ 'same-content': skill.conflict_status === 'SameContent' }"
        >
          <NCheckbox
            :checked="selectedSkills.has(skill.name)"
            @update:checked="() => toggleSkill(skill.name)"
          />
          <div class="skill-info">
            <span class="skill-name">{{ skill.name }}</span>
            <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
            <span v-if="skill.description" class="skill-desc">{{ skill.description }}</span>
          </div>
          <span class="status-badge" :class="statusBadgeClass(skill.conflict_status)">
            {{ statusBadgeText(skill.conflict_status) }}
          </span>
        </div>
      </div>
      <div class="step-actions">
        <NButton @click="currentStep = 1">{{ t('migration.prev') }}</NButton>
        <NButton
          type="primary"
          :disabled="selectedSkills.size === 0"
          @click="currentStep = 3"
        >
          {{ t('migration.next') }} ({{ selectedSkills.size }})
        </NButton>
      </div>
    </div>

    <div v-if="currentStep === 3" class="step-content">
      <div v-if="newSkills.length > 0" class="confirm-group">
        <h4>{{ t('migration.newGroup') }} ({{ newSkills.length }})</h4>
        <div v-for="skill in newSkills" :key="skill.name" class="confirm-row">
          <span class="skill-name">{{ skill.name }}</span>
          <span v-if="skill.version" class="skill-version">v{{ skill.version }}</span>
          <span class="status-badge badge-new">{{ t('migration.statusNew') }}</span>
        </div>
      </div>

      <div v-if="conflictSkills.length > 0" class="confirm-group">
        <h4>{{ t('migration.conflictGroup') }} ({{ conflictSkills.length }})</h4>
        <div v-for="skill in conflictSkills" :key="skill.name" class="confirm-row conflict-row">
          <div class="conflict-info">
            <span class="skill-name">{{ skill.name }}</span>
            <span class="status-badge" :class="statusBadgeClass(skill.conflict_status)">
              {{ statusBadgeText(skill.conflict_status) }}
            </span>
            <NButton text size="tiny" type="info" @click="showDiff(skill.name)">
              {{ t('migration.diff') }}
            </NButton>
          </div>
          <NRadioGroup
            v-model:value="conflictResolutions[skill.name]"
            size="small"
          >
            <NRadio value="Skip">{{ t('migration.skip') }}</NRadio>
            <NRadio value="Overwrite">{{ t('migration.overwrite') }}</NRadio>
          </NRadioGroup>
        </div>
      </div>

      <SkillDiffViewer
        v-if="diffSkillName && diffData"
        :diff="diffData"
        target="migration"
        :load-file-content="loadDiffFileContent"
        @close="diffSkillName = null; diffData = null"
      />

      <NDivider />
      <div class="summary-bar">
        {{ t('migration.summary', summaryCounts) }}
      </div>

      <div class="step-actions">
        <NButton @click="currentStep = 2">{{ t('migration.prev') }}</NButton>
        <NButton
          type="primary"
          :loading="skillStore.migrating"
          :disabled="summaryCounts.migrate === 0"
          @click="handleConfirm"
        >
          {{ t('migration.confirm') }}
        </NButton>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0;
  margin-bottom: 24px;
}
.step {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  opacity: 0.4;
}
.step.active { opacity: 0.7; }
.step.current { opacity: 1; }
.step-num {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 600;
  background: var(--n-color, #f0f0f0);
  color: var(--n-text-color, #333);
}
.step.current .step-num {
  background: var(--primary-color, #4a9eff);
  color: #fff;
}
.step-label { font-size: 12px; }
.step-line {
  flex: 1;
  height: 2px;
  background: var(--n-border-color, #e0e0e0);
  margin: 0 12px;
  margin-bottom: 20px;
}
.step-line.active {
  background: var(--primary-color, #4a9eff);
}
.step-content { min-height: 200px; }
.form-group { margin-bottom: 16px; }
.form-group label {
  display: block;
  font-size: 13px;
  margin-bottom: 6px;
  color: var(--n-text-color-2, #666);
}
.empty-hint {
  text-align: center;
  color: var(--n-text-color-3, #999);
  padding: 40px 0;
  font-size: 14px;
}
.scan-info {
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  margin-bottom: 12px;
  padding: 8px;
  background: var(--n-color-embedded, #f9f9f9);
  border-radius: 4px;
}
.skill-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--n-border-color, #eee);
  margin-bottom: 4px;
}
.selected-count { font-size: 12px; color: var(--n-text-color-3, #888); }
.skill-list {
  max-height: 320px;
  overflow-y: auto;
}
.skill-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 4px;
  border-bottom: 1px solid var(--n-border-color-2, #f5f5f5);
}
.skill-row.same-content { opacity: 0.5; }
.skill-info { flex: 1; min-width: 0; }
.skill-name { font-weight: 500; font-size: 14px; }
.skill-version { font-size: 12px; color: var(--n-text-color-3, #888); margin-left: 6px; }
.skill-desc {
  display: block;
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.status-badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 3px;
  white-space: nowrap;
}
.badge-new { background: #e8f8e8; color: #2d8a2d; }
.badge-same { background: #f0f0f0; color: #888; }
.badge-version { background: #fff3cd; color: #856404; }
.badge-content { background: #ffe8cc; color: #c65600; }
.confirm-group { margin-bottom: 16px; }
.confirm-group h4 { font-size: 13px; margin-bottom: 8px; color: var(--n-text-color-2, #555); }
.confirm-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
}
.conflict-row {
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: 8px 0;
  border-bottom: 1px solid var(--n-border-color-2, #f5f5f5);
}
.conflict-info { display: flex; align-items: center; gap: 8px; }
.summary-bar {
  text-align: center;
  font-size: 14px;
  font-weight: 500;
  padding: 8px;
  background: var(--n-color-embedded, #f9f9f9);
  border-radius: 4px;
}
.step-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}
</style>
