<script setup lang="ts">
import {
  NDataTable,
  NTag,
  NButton,
  NSpace,
  NTooltip,
  NText,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { h, computed } from 'vue'
import type { SkillComparison, SkillMeta, ComparisonStatus } from '@/types'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from '@/stores/config'

const { t } = useI18n()
const configStore = useConfigStore()

const props = defineProps<{
  comparisons: SkillComparison[]
  target: string
  maxHeight?: number
  flexHeight?: boolean
}>()

const emit = defineEmits<{
  install: [name: string, target: string, repoId?: string]
  update: [name: string, target: string, repoId?: string]
  uninstall: [name: string, target: string]
  preview: [name: string, repoId?: string]
}>()

function truncateHash(hash?: string): string {
  if (!hash) return '-'
  return hash.length > 12 ? hash.slice(0, 12) + '...' : hash
}

function statusTag(status: ComparisonStatus) {
  const map: Record<ComparisonStatus, { labelKey: string; type: 'success' | 'warning' | 'info' | 'error' | 'default' }> = {
    Same: { labelKey: 'status.same', type: 'success' },
    Outdated: { labelKey: 'status.outdated', type: 'warning' },
    LocalOnly: { labelKey: 'status.localOnly', type: 'info' },
    RemoteOnly: { labelKey: 'status.remoteOnly', type: 'default' },
    Unknown: { labelKey: 'status.unknown', type: 'error' },
  }
  const info = map[status]
  return h(NTag, { type: info.type, size: 'small', round: true }, { default: () => t(info.labelKey) })
}

function versionCell(meta: SkillMeta | null) {
  if (!meta) return h(NText, { depth: 3 }, () => '-')
  return h('span', { style: 'font-family: var(--font-mono); font-size: 13px' }, `v${meta.version || '?'}`)
}

function hashCell(meta: SkillMeta | null) {
  if (!meta) return h(NText, { depth: 3 }, () => '-')
  const hash = meta.checksum?.value
  if (!hash) {
    return h(NText, { depth: 3, style: 'font-size: 12px' }, () => t('table.none'))
  }
  return h(NTooltip, {}, {
    trigger: () => h(NText, { style: 'font-family: var(--font-mono); font-size: 12px; cursor: pointer' }, () => truncateHash(hash)),
    default: () => `${meta.checksum?.algorithm || 'sha256'}: ${hash}`,
  })
}

function timeCell(meta: SkillMeta | null) {
  if (!meta?.updated_at) return h(NText, { depth: 3 }, () => '-')
  const d = meta.updated_at.slice(0, 10)
  return h(NText, { depth: 2, style: 'font-size: 12px' }, () => d)
}

function descCell(meta: SkillMeta | null) {
  if (!meta) return h(NText, { depth: 3 }, () => '-')
  const desc = meta.description.length > 40 ? meta.description.slice(0, 40) + '...' : meta.description
  return h(NTooltip, {}, {
    trigger: () => h(NText, { depth: 2, style: 'font-size: 12px' }, () => desc),
    default: () => meta.description,
  })
}

function repoCell(row: SkillComparison) {
  const repoId = row.source_repo_id
  if (!repoId) return h(NText, { depth: 3 }, () => '-')
  const repo = configStore.config.repos.find(r => r.id === repoId)
  const name = repo?.name || repoId
  return h(NTag, { size: 'small', type: 'info', round: true }, () => name)
}

function actionsCell(row: SkillComparison) {
  const buttons: ReturnType<typeof h>[] = []
  const repoId = row.source_repo_id || undefined

  // Preview button always available (shows remote if no local)
  if (row.local || row.remote) {
    buttons.push(
      h(NButton, { size: 'tiny', quaternary: true, onClick: () => emit('preview', row.name, repoId) }, () => t('common.preview'))
    )
  }

  if (row.status === 'RemoteOnly') {
    buttons.push(
      h(NButton, { type: 'primary', size: 'tiny', onClick: () => emit('install', row.name, props.target, repoId) }, () => t('common.install'))
    )
  }

  if (row.status === 'Outdated') {
    buttons.push(
      h(NButton, { type: 'warning', size: 'tiny', onClick: () => emit('update', row.name, props.target, repoId) }, () => t('common.update'))
    )
  }

  if (row.local) {
    if (row.status === 'Same') {
      buttons.push(
        h(NButton, { size: 'tiny', onClick: () => emit('update', row.name, props.target, repoId) }, () => t('common.reinstallShort'))
      )
    }
    buttons.push(
      h(NButton, { size: 'tiny', type: 'error', ghost: true, onClick: () => emit('uninstall', row.name, props.target) }, () => t('common.uninstall'))
    )
  }

  return h(NSpace, { size: 4, align: 'center' }, () => buttons)
}

const columns = computed<DataTableColumns<SkillComparison>>(() => [
  {
    title: t('table.status'),
    key: 'status',
    width: 80,
    render: (row) => statusTag(row.status),
  },
  {
    title: t('table.skillName'),
    key: 'name',
    width: 160,
    render: (row) => h(NText, { strong: true }, () => row.name),
  },
  {
    title: t('table.sourceRepo'),
    key: 'source_repo',
    width: 100,
    render: (row) => repoCell(row),
  },
  {
    title: t('table.localVersion'),
    key: 'local_version',
    width: 90,
    render: (row) => versionCell(row.local),
  },
  {
    title: t('table.remoteVersion'),
    key: 'remote_version',
    width: 90,
    render: (row) => versionCell(row.remote),
  },
  {
    title: t('table.localHash'),
    key: 'local_hash',
    width: 100,
    render: (row) => hashCell(row.local),
  },
  {
    title: t('table.remoteHash'),
    key: 'remote_hash',
    width: 100,
    render: (row) => hashCell(row.remote),
  },
  {
    title: t('table.updatedTime'),
    key: 'updated_at',
    width: 95,
    render: (row) => timeCell(row.remote || row.local),
  },
  {
    title: t('table.description'),
    key: 'description',
    ellipsis: { tooltip: true },
    render: (row) => descCell(row.remote || row.local),
  },
  {
    title: t('table.actions'),
    key: 'actions',
    width: 180,
    fixed: 'right',
    render: (row) => actionsCell(row),
  },
])
</script>

<template>
  <NDataTable
    :columns="columns"
    :data="comparisons"
    :bordered="false"
    :scroll-x="1050"
    :max-height="maxHeight"
    :flex-height="flexHeight"
    size="small"
    striped
    :row-key="(row: SkillComparison) => row.name + '_' + (row.source_repo_id || '')"
  />
</template>
