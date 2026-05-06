<template>
  <div class="conflict-resolver">
    <div class="conflict-sidebar">
      <button
        v-for="file in files"
        :key="file.file_path"
        class="conflict-file"
        :class="{ active: file.file_path === activePath }"
        @click="activePath = file.file_path"
      >
        <span>{{ file.file_path }}</span>
        <small>{{ statusText(file.status) }}</small>
      </button>
    </div>

    <div v-if="activeFile" class="conflict-main">
      <div class="conflict-toolbar">
        <strong>{{ activeFile.file_path }}</strong>
        <span class="conflict-badge">{{ statusText(activeFile.status) }}</span>
      </div>

      <div class="conflict-columns">
        <section class="conflict-panel local-panel">
          <header>本地内容</header>
          <textarea
            v-model="drafts[activeFile.file_path]"
            :disabled="activeFile.is_binary"
          ></textarea>
        </section>

        <section class="conflict-panel remote-panel">
          <header>远端内容</header>
          <pre>{{ remoteText(activeFile) }}</pre>
        </section>
      </div>

      <div class="conflict-actions">
        <button class="abort-btn" @click="$emit('abort')">中止同步</button>
        <button class="continue-btn" @click="submit">继续</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import type { GitConflictFile, ResolvedConflictFile } from '@/types'

const props = defineProps<{
  files: GitConflictFile[]
}>()

const emit = defineEmits<{
  continue: [files: ResolvedConflictFile[]]
  abort: []
}>()

const activePath = ref('')
const drafts = reactive<Record<string, string>>({})

watch(
  () => props.files,
  (files) => {
    for (const file of files) {
      if (!(file.file_path in drafts)) {
        drafts[file.file_path] = file.local_content ?? ''
      }
    }

    if (!activePath.value && files.length > 0) {
      activePath.value = files[0].file_path
    }
  },
  { immediate: true }
)

const activeFile = computed(() => {
  return props.files.find(file => file.file_path === activePath.value) || props.files[0]
})

const statusText = (status: string) => {
  const map: Record<string, string> = {
    both_modified: '双方修改',
    local_deleted: '本地删除',
    remote_deleted: '远端删除',
    both_added: '双方新增',
    unsupported: '不支持自动展示'
  }
  return map[status] || status
}

const remoteText = (file: GitConflictFile) => {
  if (file.is_binary) return '二进制文件无法预览'
  return file.remote_content ?? '远端为空或已删除'
}

const submit = () => {
  emit('continue', props.files.map(file => ({
    file_path: file.file_path,
    final_content: file.is_binary ? file.local_content : drafts[file.file_path]
  })))
}
</script>

<style scoped>
.conflict-resolver {
  display: grid;
  grid-template-columns: 220px 1fr;
  min-height: 520px;
  border: 1px solid #e5e7eb;
  background: #fff;
}

.conflict-sidebar {
  border-right: 1px solid #e5e7eb;
  overflow: auto;
}

.conflict-file {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border: 0;
  border-bottom: 1px solid #f0f0f0;
  background: transparent;
  text-align: left;
  cursor: pointer;
}

.conflict-file.active {
  background: #eef6ff;
}

.conflict-file small {
  color: #b45309;
}

.conflict-main {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.conflict-toolbar,
.conflict-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.conflict-badge {
  color: #b45309;
  border: 1px solid #f59e0b;
  padding: 2px 8px;
  border-radius: 4px;
}

.conflict-columns {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
  padding: 12px;
  min-height: 420px;
}

.conflict-panel {
  border: 2px solid #f59e0b;
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.conflict-panel header {
  padding: 8px 10px;
  background: #fffbeb;
  border-bottom: 1px solid #fde68a;
  font-weight: 600;
}

.conflict-panel textarea,
.conflict-panel pre {
  flex: 1;
  margin: 0;
  padding: 12px;
  border: 0;
  resize: none;
  overflow: auto;
  font-family: Monaco, Menlo, 'Ubuntu Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  white-space: pre-wrap;
}

.conflict-panel textarea:focus {
  outline: 2px solid #2563eb;
  outline-offset: -2px;
}

.conflict-actions {
  border-top: 1px solid #e5e7eb;
  border-bottom: 0;
}

.abort-btn,
.continue-btn {
  padding: 8px 14px;
  border-radius: 6px;
  border: 1px solid #d1d5db;
  cursor: pointer;
}

.continue-btn {
  background: #2563eb;
  color: #fff;
  border-color: #2563eb;
}

@media (max-width: 760px) {
  .conflict-resolver {
    grid-template-columns: 1fr;
  }

  .conflict-sidebar {
    display: flex;
    border-right: 0;
    border-bottom: 1px solid #e5e7eb;
  }

  .conflict-file {
    min-width: 180px;
    border-bottom: 0;
    border-right: 1px solid #f0f0f0;
  }

  .conflict-columns {
    grid-template-columns: 1fr;
  }
}
</style>
