<script setup lang="ts">
import { NTag } from 'naive-ui'
import type { InstallStatus } from '@/types'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = defineProps<{
  status?: InstallStatus
}>()

const statusMap: Record<string, { labelKey: string; type: 'success' | 'warning' | 'error' | 'default' }> = {
  installed: { labelKey: 'status.installed', type: 'success' },
  outdated: { labelKey: 'status.outdated', type: 'warning' },
  not_installed: { labelKey: 'status.notInstalled', type: 'default' },
  unknown: { labelKey: 'status.unknown', type: 'error' },
}

const info = statusMap[props.status ?? 'unknown'] ?? statusMap.unknown
</script>

<template>
  <NTag :type="info.type" size="small" round>
    {{ t(info.labelKey) }}
  </NTag>
</template>
