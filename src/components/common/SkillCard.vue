<script setup lang="ts">
import { NCard, NTag, NButton, NSpace, NText } from 'naive-ui'
import type { SkillMeta } from '@/types'
import VersionBadge from './VersionBadge.vue'
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'

const { t } = useI18n()

const props = defineProps<{
  skill: SkillMeta
  target: string
}>()

const emit = defineEmits<{
  install: [name: string, target: string]
  update: [name: string, target: string]
  uninstall: [name: string, target: string]
}>()

function truncateHash(hash?: string): string {
  if (!hash) return '-'
  return hash.length > 16 ? hash.slice(0, 16) + '...' : hash
}

const displayTags = computed(() => {
  const allTags = [...props.skill.tags]
  if (props.skill.trigger?.tags) {
    for (const t of props.skill.trigger.tags) {
      if (!allTags.includes(t)) allTags.push(t)
    }
  }
  return allTags.slice(0, 6)
})
</script>

<template>
  <NCard size="small" hoverable class="skill-card">
    <div class="skill-header">
      <div class="skill-title-row">
        <NText strong class="skill-name">{{ skill.name }}</NText>
        <NText depth="3" class="skill-version">v{{ skill.version || '?' }}</NText>
        <VersionBadge :status="skill.install_status" />
      </div>
      </div>
      <div v-if="skill.author || skill.language" class="skill-meta-line">
        <NText v-if="skill.author" depth="3" style="font-size: 12px">
          by {{ skill.author }}
        </NText>
        <NTag v-if="skill.language" size="tiny" round :bordered="false" type="info" style="margin-left: 6px">
          {{ skill.language }}
        </NTag>
      </div>
      <div class="skill-desc">
        <NText depth="2" style="font-size: 13px">
          {{ skill.description.length > 80 ? skill.description.slice(0, 80) + '...' : skill.description }}
        </NText>
      </div>
      <div class="skill-tags">
        <NTag v-for="tag in displayTags" :key="tag" size="tiny" round type="info">
          {{ tag }}
        </NTag>
      </div>
      <div v-if="skill.checksum" class="skill-hash">
        <NText depth="3" style="font-size: 11px">
          {{ skill.checksum.algorithm.toUpperCase() }}: {{ truncateHash(skill.checksum.value) }}
        </NText>
      </div>
      <div v-if="skill.updated_at" class="skill-time">
        <NText depth="3" style="font-size: 11px">
          {{ t('skillCard.updated', { time: skill.updated_at }) }}
        </NText>
      </div>
    </div>
    <template #action>
      <NSpace>
        <NButton
          v-if="skill.install_status === 'not_installed'"
          type="primary"
          size="small"
          @click="emit('install', skill.name, target)"
        >
          {{ t('common.install') }}
        </NButton>
        <NButton
          v-if="skill.install_status === 'outdated'"
          type="warning"
          size="small"
          @click="emit('update', skill.name, target)"
        >
          {{ t('common.update') }}
        </NButton>
        <NButton
          v-if="skill.install_status === 'installed' || skill.install_status === 'outdated'"
          size="small"
          @click="emit('uninstall', skill.name, target)"
        >
          {{ t('common.uninstall') }}
        </NButton>
        <NButton
          v-if="skill.install_status === 'installed'"
          type="primary"
          size="small"
          ghost
          @click="emit('update', skill.name, target)"
        >
          {{ t('common.reinstall') }}
        </NButton>
      </NSpace>
    </template>
  </NCard>
</template>

<style scoped>
.skill-card {
  margin-bottom: 12px;
}

.skill-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.skill-name {
  font-size: 15px;
}

.skill-version {
  font-size: 13px;
  font-family: monospace;
}

.skill-desc {
  margin-top: 6px;
}

.skill-meta-line {
  margin-top: 4px;
  display: flex;
  align-items: center;
}

.skill-tags {
  display: flex;
  gap: 4px;
  margin-top: 8px;
  flex-wrap: wrap;
}

.skill-hash,
.skill-time {
  margin-top: 4px;
}
</style>
