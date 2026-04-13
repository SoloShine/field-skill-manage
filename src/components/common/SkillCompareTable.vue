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
import { h } from 'vue'
import type { SkillComparison, SkillMeta, ComparisonStatus } from '@/types'

const props = defineProps<{
  comparisons: SkillComparison[]
  target: string
}>()

const emit = defineEmits<{
  install: [name: string, target: string]
  update: [name: string, target: string]
  uninstall: [name: string, target: string]
  preview: [name: string]
}>()

function truncateHash(hash?: string): string {
  if (!hash) return '-'
  return hash.length > 12 ? hash.slice(0, 12) + '...' : hash
}

function statusTag(status: ComparisonStatus) {
  const map: Record<ComparisonStatus, { label: string; type: 'success' | 'warning' | 'info' | 'error' | 'default' }> = {
    Same: { label: '一致', type: 'success' },
    Outdated: { label: '可更新', type: 'warning' },
    LocalOnly: { label: '仅本地', type: 'info' },
    RemoteOnly: { label: '仅远端', type: 'default' },
    Unknown: { label: '未知', type: 'error' },
  }
  const info = map[status]
  return h(NTag, { type: info.type, size: 'small', round: true }, { default: () => info.label })
}

function versionCell(meta: SkillMeta | null) {
  if (!meta) return h(NText, { depth: 3 }, () => '-')
  return h('span', { style: 'font-family: monospace; font-size: 13px' }, `v${meta.version || '?'}`)
}

function hashCell(meta: SkillMeta | null) {
  if (!meta) return h(NText, { depth: 3 }, () => '-')
  const hash = meta.checksum?.value
  if (!hash) {
    // No checksum, show "(无)"
    return h(NText, { depth: 3, style: 'font-size: 12px' }, () => '(无)')
  }
  return h(NTooltip, {}, {
    trigger: () => h(NText, { style: 'font-family: monospace; font-size: 12px; cursor: pointer' }, () => truncateHash(hash)),
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

function actionsCell(row: SkillComparison) {
  const buttons: ReturnType<typeof h>[] = []

  // Preview button always available (shows remote if no local)
  if (row.local || row.remote) {
    buttons.push(
      h(NButton, { size: 'tiny', quaternary: true, onClick: () => emit('preview', row.name) }, () => '预览')
    )
  }

  if (row.status === 'RemoteOnly') {
    buttons.push(
      h(NButton, { type: 'primary', size: 'tiny', onClick: () => emit('install', row.name, props.target) }, () => '安装')
    )
  }

  if (row.status === 'Outdated') {
    buttons.push(
      h(NButton, { type: 'warning', size: 'tiny', onClick: () => emit('update', row.name, props.target) }, () => '更新')
    )
  }

  if (row.local) {
    if (row.status === 'Same') {
      buttons.push(
        h(NButton, { size: 'tiny', onClick: () => emit('update', row.name, props.target) }, () => '重装')
      )
    }
    buttons.push(
      h(NButton, { size: 'tiny', type: 'error', ghost: true, onClick: () => emit('uninstall', row.name, props.target) }, () => '卸载')
    )
  }

  return h(NSpace, { size: 4, align: 'center' }, () => buttons)
}

const columns: DataTableColumns<SkillComparison> = [
  {
    title: '状态',
    key: 'status',
    width: 80,
    render: (row) => statusTag(row.status),
  },
  {
    title: 'Skill 名称',
    key: 'name',
    width: 160,
    render: (row) => h(NText, { strong: true }, () => row.name),
  },
  {
    title: '本地版本',
    key: 'local_version',
    width: 90,
    render: (row) => versionCell(row.local),
  },
  {
    title: '远端版本',
    key: 'remote_version',
    width: 90,
    render: (row) => versionCell(row.remote),
  },
  {
    title: '本地哈希',
    key: 'local_hash',
    width: 100,
    render: (row) => hashCell(row.local),
  },
  {
    title: '远端哈希',
    key: 'remote_hash',
    width: 100,
    render: (row) => hashCell(row.remote),
  },
  {
    title: '更新时间',
    key: 'updated_at',
    width: 95,
    render: (row) => timeCell(row.remote || row.local),
  },
  {
    title: '说明',
    key: 'description',
    ellipsis: { tooltip: true },
    render: (row) => descCell(row.remote || row.local),
  },
  {
    title: '操作',
    key: 'actions',
    width: 180,
    fixed: 'right',
    render: (row) => actionsCell(row),
  },
]
</script>

<template>
  <NDataTable
    :columns="columns"
    :data="comparisons"
    :bordered="false"
    :scroll-x="950"
    size="small"
    striped
    :row-key="(row: SkillComparison) => row.name"
  />
</template>
