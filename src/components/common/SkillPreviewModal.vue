<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { NModal, NTree, NSpin, NEmpty, NText, NDescriptions, NDescriptionsItem, NTag } from 'naive-ui'
import type { TreeOption } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface FileNode {
  name: string
  path: string
  is_dir: boolean
  children?: FileNode[]
}

interface FrontmatterData {
  name: string
  version: string
  description: string
  tags: string[]
  license: string
  updated_at: string
  // catch-all for unknown keys
  extra: Record<string, string>
}

const props = defineProps<{
  skillName: string
  target: string
}>()

const emit = defineEmits<{ close: [] }>()

const fileTree = ref<FileNode[]>([])
const currentFile = ref('SKILL.md')
const rawContent = ref('')
const loading = ref(false)
const treeLoading = ref(false)
const frontmatter = ref<FrontmatterData | null>(null)

const isMarkdown = computed(() => {
  const ext = currentFile.value.split('.').pop()?.toLowerCase()
  return ext === 'md' || ext === 'mdx'
})

/// Parse YAML frontmatter into structured data
function parseFrontmatter(content: string): { fm: FrontmatterData | null; body: string } {
  const trimmed = content.trimStart()
  if (!trimmed.startsWith('---')) return { fm: null, body: content }

  const rest = trimmed.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return { fm: null, body: content }

  const yaml = rest.slice(0, end)
  const body = rest.slice(end + 4).trimStart()

  const fm: FrontmatterData = {
    name: '',
    version: '',
    description: '',
    tags: [],
    license: '',
    updated_at: '',
    extra: {},
  }

  for (const line of yaml.split('\n')) {
    const trimmedLine = line.trim()
    if (!trimmedLine || trimmedLine.startsWith('#')) continue
    const colon = trimmedLine.indexOf(':')
    if (colon === -1) continue
    const key = trimmedLine.slice(0, colon).trim()
    let val = trimmedLine.slice(colon + 1).trim()

    // Handle list values like [a, b, c]
    if (val.startsWith('[') && val.endsWith(']')) {
      val = val.slice(1, -1)
      const items = val.split(',').map(s => s.trim()).filter(Boolean)
      if (key === 'tags') {
        fm.tags = items
        continue
      }
      fm.extra[key] = items.join(', ')
      continue
    }

    switch (key) {
      case 'name': fm.name = val; break
      case 'version': fm.version = val; break
      case 'description': fm.description = val; break
      case 'license': fm.license = val; break
      case 'updated_at': fm.updated_at = val; break
      default: fm.extra[key] = val
    }
  }

  return { fm, body }
}

/// Strip frontmatter, keep body only
function stripFrontmatter(content: string): string {
  const trimmed = content.trimStart()
  if (!trimmed.startsWith('---')) return content
  const rest = trimmed.slice(3)
  const end = rest.indexOf('\n---')
  if (end === -1) return content
  return rest.slice(end + 4).trimStart()
}

const renderedContent = computed(() => {
  if (!isMarkdown.value) return ''
  try {
    const cleaned = stripFrontmatter(rawContent.value)
    return marked(cleaned) as string
  } catch {
    return `<p>${t('preview.renderFailed')}</p>`
  }
})

function toTreeOption(node: FileNode): TreeOption {
  const icon = node.is_dir ? '📁 ' : getFileIcon(node.name)
  return {
    key: node.path,
    label: icon + node.name,
    isLeaf: !node.is_dir,
    children: node.children?.map(toTreeOption),
  }
}

function getFileIcon(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase()
  const icons: Record<string, string> = {
    md: '📄', py: '🐍', cs: '💻', ts: '📜', js: '📜',
    json: '📋', yaml: '📋', yml: '📋', txt: '📝',
  }
  return icons[ext || ''] || '📄'
}

async function loadFileTree() {
  treeLoading.value = true
  try {
    fileTree.value = await invoke<FileNode[]>('get_skill_file_tree', {
      skillName: props.skillName,
      target: props.target,
    })
  } catch {
    fileTree.value = []
  } finally {
    treeLoading.value = false
  }
}

async function loadFile(path: string) {
  loading.value = true
  try {
    rawContent.value = await invoke<string>('read_skill_file', {
      skillName: props.skillName,
      filePath: path,
      target: props.target,
    })
    currentFile.value = path
    // Parse frontmatter for SKILL.md
    if (path === 'SKILL.md' || path.endsWith('/SKILL.md')) {
      const { fm } = parseFrontmatter(rawContent.value)
      frontmatter.value = fm
    } else {
      frontmatter.value = null
    }
  } catch (e: any) {
    rawContent.value = t('preview.loadFailed', { error: e })
    frontmatter.value = null
  } finally {
    loading.value = false
  }
}

function handleFileSelect(keys: string[]) {
  if (keys.length > 0) {
    loadFile(keys[0])
  }
}

const treeOptions = computed(() => fileTree.value.map(toTreeOption))
const defaultExpandedKeys = computed(() =>
  fileTree.value.filter((n) => n.is_dir).map((n) => n.path)
)

onMounted(async () => {
  await loadFileTree()
  await loadFile('SKILL.md')
})
</script>

<template>
  <NModal :show="true" preset="card" :title="skillName" style="width: 800px; max-height: 80vh" @update:show="emit('close')">
    <div class="preview-layout">
      <div class="file-tree-panel">
        <NText depth="3" style="font-size: 12px; padding: 8px; display: block">{{ t('preview.fileList') }}</NText>
        <NSpin :show="treeLoading" size="small">
          <NTree
            v-if="treeOptions.length > 0"
            :data="treeOptions"
            :default-expanded-keys="defaultExpandedKeys"
            :selected-keys="[currentFile]"
            block-line
            selectable
            @update:selected-keys="handleFileSelect"
            style="font-size: 13px"
          />
          <NEmpty v-else :description="t('preview.noFiles')" size="small" />
        </NSpin>
      </div>
      <div class="file-content-panel">
        <div class="file-tab">
          <NText strong style="font-size: 13px">{{ currentFile }}</NText>
        </div>
        <div class="file-content-scroll">
          <NSpin :show="loading" size="small">
            <!-- Frontmatter metadata card for SKILL.md -->
            <div v-if="frontmatter" class="fm-card">
              <NDescriptions label-placement="left" bordered size="small" :column="1">
                <NDescriptionsItem :label="t('preview.name')">
                  <NText strong>{{ frontmatter.name }}</NText>
                </NDescriptionsItem>
                <NDescriptionsItem :label="t('preview.version')">
                  <NText code>{{ frontmatter.version }}</NText>
                </NDescriptionsItem>
                <NDescriptionsItem :label="t('preview.description')">
                  {{ frontmatter.description }}
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.tags.length > 0" :label="t('preview.tags')">
                  <NTag v-for="tag in frontmatter.tags" :key="tag" size="small" round type="info" style="margin-right: 4px">
                    {{ tag }}
                  </NTag>
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.license" :label="t('preview.license')">
                  {{ frontmatter.license }}
                </NDescriptionsItem>
                <NDescriptionsItem v-if="frontmatter.updated_at" :label="t('preview.updatedAt')">
                  {{ frontmatter.updated_at }}
                </NDescriptionsItem>
                <NDescriptionsItem v-for="(val, key) in frontmatter.extra" :key="key" :label="key">
                  {{ val }}
                </NDescriptionsItem>
              </NDescriptions>
            </div>
            <!-- Markdown rendered body -->
            <div v-if="isMarkdown && renderedContent" class="md-content" v-html="renderedContent" />
            <!-- Raw content for non-markdown -->
            <pre v-else-if="rawContent && !isMarkdown" class="raw-content">{{ rawContent }}</pre>
            <NEmpty v-else-if="!isMarkdown && !rawContent" :description="t('preview.selectFile')" size="small" />
          </NSpin>
        </div>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.preview-layout {
  display: flex;
  gap: 0;
  height: 500px;
  border: 1px solid #eee;
  border-radius: 6px;
  overflow: hidden;
}
.file-tree-panel {
  width: 200px;
  min-width: 200px;
  border-right: 1px solid #eee;
  overflow-y: auto;
  background: #fafafa;
}
.file-content-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.file-tab {
  padding: 6px 12px;
  border-bottom: 1px solid #eee;
  background: #fafafa;
  flex-shrink: 0;
}
.file-content-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.fm-card {
  padding: 12px 16px;
  border-bottom: 1px solid #eee;
  background: #f9fafb;
}
.md-content {
  padding: 16px;
  font-size: 14px;
  line-height: 1.6;
}
.md-content :deep(h1), .md-content :deep(h2), .md-content :deep(h3) {
  margin-top: 16px;
  margin-bottom: 8px;
}
.md-content :deep(code) {
  background: #f0f0f0;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 13px;
}
.md-content :deep(pre) {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
}
.md-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
}
.md-content :deep(th), .md-content :deep(td) {
  border: 1px solid #ddd;
  padding: 6px 10px;
  text-align: left;
}
.raw-content {
  padding: 16px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}
</style>
