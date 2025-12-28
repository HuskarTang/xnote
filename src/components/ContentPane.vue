<template>
  <div class="content-pane-container">
    <!-- Action Bar -->
    <ActionBar />

    <!-- 主要内容区域 -->
    <div class="content-main">
      <!-- 无选中笔记时的空状态 -->
      <div v-if="!currentNote" class="empty-content">
        <div class="empty-icon">
          <Icons name="note" :size="64" />
        </div>
        <h3>Select a note to get started</h3>
        <p>Choose a note from the list or create a new one to begin writing.</p>
      </div>

      <!-- 有选中笔记时的内容 -->
      <div v-else class="note-content">
        <!-- 内容区域 -->
        <div class="content-section" :class="contentSectionClass">
          <!-- 编辑模式 -->
          <div v-if="viewMode === 'edit'" class="pane-wrapper edit-wrapper">
            <div class="title-section">
              <input
                v-model="noteTitle"
                type="text"
                class="title-input"
                placeholder="Note title..."
                @blur="updateTitle"
                @keyup.enter="updateTitle"
              />
            </div>
            <div class="edit-pane-container">
              <EditPane :content="currentNote?.content" :note-id="currentNote?.id" @update:content="handleContentUpdate" @save="handleAutoSave" />
            </div>
          </div>

          <!-- 查看模式 -->
          <div v-else-if="viewMode === 'view'" class="pane-wrapper view-wrapper">
            <div class="title-section">
              <h1 class="title-display">
                {{ currentNote.title || 'Untitled' }}
              </h1>
            </div>
            <div class="view-pane-container">
              <ViewPane />
            </div>
          </div>

          <!-- 分屏模式 -->
          <div v-else-if="viewMode === 'split'" class="split-pane-container">
            <div class="split-half edit-half">
              <div class="title-section">
                <input
                  v-model="noteTitle"
                  type="text"
                  class="title-input"
                  placeholder="Note title..."
                  @blur="updateTitle"
                  @keyup.enter="updateTitle"
                />
              </div>
              <div class="edit-pane-container">
                <EditPane :content="splitContent" :note-id="currentNote?.id" @update:content="updateSplitContent" @save="handleAutoSave" />
              </div>
            </div>
            <div class="split-divider"></div>
            <div class="split-half view-half">
              <div class="title-section">
                <h1 class="title-display">
                  {{ noteTitle || 'Untitled' }}
                </h1>
              </div>
              <div class="view-pane-container">
                <ViewPane :content="splitContent" />
              </div>
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
import Icons from '@/components/Icons.vue'

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

// 处理内容更新（用于编辑模式下的实时预览）
const handleContentUpdate = (content: string) => {
}

// 自动保存处理
const handleAutoSave = async (content: string) => {
  if (!currentNote.value) {
    console.log('No current note to save')
    return
  }
  
  try {
    saveStatus.value = 'Saving...'
    await notesStore.updateNote(currentNote.value.id, { content })
    saveStatus.value = 'Saved'
  } catch (err) {
    saveStatus.value = 'Error saving'
    console.error('Failed to auto save note:', err)
  }
}

// 方法
const updateTitle = async () => {
  if (!currentNote.value || noteTitle.value === currentNote.value.title) return
  
  try {
    saveStatus.value = 'Saving...'
    const updatedNote = await notesStore.updateNote(currentNote.value.id, { title: noteTitle.value })
    if (updatedNote) {
      noteTitle.value = updatedNote.title
    }
    saveStatus.value = 'Saved'
  } catch (err) {
    saveStatus.value = 'Error saving'
    console.error('Failed to update title:', err)
    if (currentNote.value) {
      noteTitle.value = currentNote.value.title
    }
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
    if (viewMode.value === 'split') {
      splitContent.value = newNote.content
    }
  } else {
    noteTitle.value = ''
    splitContent.value = ''
  }
}, { immediate: true })

watch(viewMode, (newMode) => {
  if (newMode === 'split' && currentNote.value) {
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

.content-section {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.pane-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.title-section {
  padding: 6px 6px 6px 6px;
  background-color: #ffffff;
  border-bottom: 1px solid #e5e7eb;
}

.title-input {
  width: 100%;
  font-size: 24px;
  font-weight: 600;
  border: 1px solid #e5e7eb;
  outline: none;
  background: #fdfdfd;
  color: #374151;
  padding: 8px 16px;
  border-radius: 6px;
  transition: all 0.2s ease;
  font-family: inherit;
  line-height: 1.3;
}

.title-input:hover {
  border-color: #d1d5db;
}

.title-input:focus {
  background-color: #ffffff;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.title-display {
  font-size: 24px;
  font-weight: 600;
  margin: 0;
  padding: 8px 16px;
  color: #374151;
  line-height: 1.3;
  border: 1px solid transparent;
}

.edit-pane-container {
  flex: 1;
  overflow: hidden;
}

.view-pane-container {
  flex: 1;
  overflow: auto;
}

.split-pane-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.split-half {
  flex: 1;
  display: flex;
  flex-direction: column;
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