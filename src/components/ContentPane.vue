<template>
  <div class="content-pane-container">
    <!-- Action Bar -->
    <ActionBar />

    <!-- 主要内容区域 -->
    <div class="content-main">
      <!-- 无选中笔记时的空状态 -->
      <div v-if="!currentNote" class="empty-content">
        <div class="empty-icon">📝</div>
        <h3>Select a note to get started</h3>
        <p>Choose a note from the list or create a new one to begin writing.</p>
      </div>

      <!-- 有选中笔记时的内容 -->
      <div v-else class="note-content">
        <!-- 标题编辑区 -->
        <div class="title-section">
          <input
            v-if="viewMode === 'edit' || viewMode === 'split'"
            v-model="noteTitle"
            type="text"
            class="title-input"
            placeholder="Note title..."
            @blur="updateTitle"
            @keyup.enter="updateTitle"
          />
          <h1 v-else class="title-display">
            {{ currentNote.title || 'Untitled' }}
          </h1>
        </div>

        <!-- 内容区域 -->
        <div class="content-section" :class="contentSectionClass">
          <!-- 编辑模式 -->
          <div v-if="viewMode === 'edit'" class="edit-pane-container">
            <EditPane />
          </div>

          <!-- 查看模式 -->
          <div v-else-if="viewMode === 'view'" class="view-pane-container">
            <ViewPane />
          </div>

          <!-- 分屏模式 -->
          <div v-else-if="viewMode === 'split'" class="split-pane-container">
            <div class="edit-section">
              <EditPane :content="splitContent" @update:content="updateSplitContent" />
            </div>
            <div class="split-divider"></div>
            <div class="view-section">
              <ViewPane :content="splitContent" />
            </div>
          </div>
        </div>

        <!-- 状态栏 -->
        <div class="status-bar">
          <span class="status-item">{{ getWordCount() }} words</span>
          <span class="status-item">{{ getCharCount() }} characters</span>
          <span class="status-item" :class="saveStatusClass">{{ saveStatus }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { useNotesStore } from '@/stores/notes'
import ActionBar from '@/components/ActionBar.vue'
import EditPane from '@/components/EditPane.vue'
import ViewPane from '@/components/ViewPane.vue'

const appStore = useAppStore()
const notesStore = useNotesStore()

const { viewMode } = storeToRefs(appStore)
const { currentNote } = storeToRefs(notesStore)

const noteTitle = ref('')
const saveStatus = ref('Saved')
const splitContent = ref('')

// 计算属性
const contentSectionClass = computed(() => ({
  'mode-edit': viewMode.value === 'edit',
  'mode-view': viewMode.value === 'view',
  'mode-split': viewMode.value === 'split'
}))

const saveStatusClass = computed(() => ({
  'status-saved': saveStatus.value === 'Saved',
  'status-saving': saveStatus.value === 'Saving...',
  'status-error': saveStatus.value.includes('Error')
}))

// 更新分屏内容
const updateSplitContent = (content: string) => {
  splitContent.value = content
}

// 方法
const updateTitle = async () => {
  if (!currentNote.value || noteTitle.value === currentNote.value.title) return
  
  try {
    saveStatus.value = 'Saving...'
    await notesStore.updateNote(currentNote.value.id, { title: noteTitle.value })
    saveStatus.value = 'Saved'
  } catch (err) {
    saveStatus.value = 'Error saving'
    console.error('Failed to update title:', err)
  }
}

const getWordCount = () => {
  const content = viewMode.value === 'split' ? splitContent.value : (currentNote.value?.content || '')
  if (!content) return 0
  const words = content
    .replace(/[^\w\s\u4e00-\u9fff]/g, ' ')
    .split(/\s+/)
    .filter(word => word.length > 0)
  return words.length
}

const getCharCount = () => {
  const content = viewMode.value === 'split' ? splitContent.value : (currentNote.value?.content || '')
  return content.length || 0
}

// 监听器
watch(currentNote, (newNote) => {
  if (newNote) {
    noteTitle.value = newNote.title
    saveStatus.value = 'Saved'
    // 在分屏模式下初始化内容
    if (viewMode.value === 'split') {
      splitContent.value = newNote.content
    }
  } else {
    noteTitle.value = ''
    splitContent.value = ''
  }
}, { immediate: true })

// 监听视图模式变化
watch(viewMode, (newMode) => {
  if (newMode === 'split' && currentNote.value) {
    // 进入分屏模式时初始化内容
    splitContent.value = currentNote.value.content
  }
})
</script>

<style scoped>
.content-pane-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}

.content-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: #666;
  padding: 40px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-content h3 {
  margin: 0 0 8px 0;
  font-size: 20px;
  font-weight: 600;
}

.empty-content p {
  margin: 0;
  font-size: 14px;
  opacity: 0.7;
}

.note-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.title-section {
  padding: 16px 24px 0 24px;
  border-bottom: 1px solid #f0f0f0;
}

.title-input {
  width: 100%;
  font-size: 24px;
  font-weight: 600;
  border: none;
  outline: none;
  background: transparent;
  color: #333;
  padding: 8px 0 16px 0;
  border-radius: 4px;
}

.title-input:focus {
  background-color: #f8f8f8;
  padding-left: 12px;
  padding-right: 12px;
}

.title-display {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  padding: 8px 0 16px 0;
  color: #333;
  line-height: 1.3;
}

.content-section {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.edit-pane-container,
.view-pane-container {
  flex: 1;
  overflow: hidden;
}

.split-pane-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.edit-section,
.view-section {
  flex: 1;
  overflow: hidden;
}

.split-divider {
  width: 1px;
  background-color: #e5e5e5;
  flex-shrink: 0;
}

.status-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 16px;
  padding: 8px 24px;
  background-color: #f8f8f8;
  border-top: 1px solid #e5e5e5;
  font-size: 12px;
  color: #666;
  min-height: 32px;
}

.status-item {
  white-space: nowrap;
}

.status-saved {
  color: #28a745;
}

.status-saving {
  color: #007acc;
}

.status-error {
  color: #dc3545;
}
</style>