<script setup lang="ts">
import { NTag } from 'naive-ui'
import type { InstallStatus } from '@/types'

const props = defineProps<{
  status?: InstallStatus
}>()

const statusMap: Record<string, { label: string; type: 'success' | 'warning' | 'error' | 'default' }> = {
  installed: { label: '已安装', type: 'success' },
  outdated: { label: '可更新', type: 'warning' },
  not_installed: { label: '未安装', type: 'default' },
  unknown: { label: '未知', type: 'error' },
}

const info = statusMap[props.status ?? 'unknown'] ?? statusMap.unknown
</script>

<template>
  <NTag :type="info.type" size="small" round>
    {{ info.label }}
  </NTag>
</template>
