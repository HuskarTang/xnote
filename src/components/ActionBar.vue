<template>
  <div class="action-bar">
    <!-- 视图模式切换 -->
    <div class="mode-buttons">
      <button 
        class="action-button"
        :class="{ active: viewMode === 'view' }"
        @click="setViewMode('view')"
        title="View mode"
      >
        👁️
      </button>
      <button 
        class="action-button"
        :class="{ active: viewMode === 'edit' }"
        @click="setViewMode('edit')"
        title="Edit mode"
      >
        ✏️
      </button>
      <button 
        class="action-button"
        :class="{ active: viewMode === 'split' }"
        @click="setViewMode('split')"
        title="Split mode"
      >
        📱
      </button>
    </div>

    <!-- 分隔线 -->
    <div class="separator"></div>

    <!-- 笔记操作 -->
    <div class="note-actions" v-if="currentNote">
      <button 
        class="action-button"
        :class="{ active: currentNote.is_favorite }"
        @click="toggleFavorite"
        title="Toggle favorite"
      >
        ⭐
      </button>
      <button 
        class="action-button"
        @click="showTagManager"
        title="Manage tags"
      >
        🏷️
      </button>
      <button 
        class="action-button"
        @click="showAttachmentManager"
        title="Manage attachments"
      >
        📎
      </button>
      <button 
        class="action-button delete-button"
        @click="deleteNote"
        title="Delete note"
      >
        🗑️
      </button>
      <button 
        class="action-button"
        @click="exportNote"
        title="Export note"
      >
        📤
      </button>
    </div>

    <!-- 空状态提示 -->
    <div v-else class="empty-state">
      Select a note to view actions
    </div>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { useNotesStore } from '@/stores/notes'

const appStore = useAppStore()
const notesStore = useNotesStore()

const { viewMode } = storeToRefs(appStore)
const { currentNote } = storeToRefs(notesStore)

const setViewMode = (mode: 'view' | 'edit' | 'split') => {
  appStore.setViewMode(mode)
}

const toggleFavorite = async () => {
  if (!currentNote.value) return
  
  try {
    await notesStore.updateNote(currentNote.value.id, {
      is_favorite: !currentNote.value.is_favorite
    })
  } catch (err) {
    console.error('Failed to toggle favorite:', err)
  }
}

const deleteNote = async () => {
  if (!currentNote.value) return
  
  if (confirm(`Are you sure you want to delete "${currentNote.value.title}"?`)) {
    try {
      await notesStore.deleteNote(currentNote.value.id)
    } catch (err) {
      console.error('Failed to delete note:', err)
    }
  }
}

// 占位函数，后续实现
const showTagManager = () => {
  alert('Tag management will be implemented')
}

const showAttachmentManager = () => {
  alert('Attachment management will be implemented')
}

const exportNote = () => {
  alert('Export functionality will be implemented')
}
</script>

<style scoped>
.action-bar {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background-color: #f8f8f8;
  border-bottom: 1px solid #e5e5e5;
  min-height: 48px;
  gap: 8px;
}

.mode-buttons {
  display: flex;
  gap: 4px;
}

.separator {
  width: 1px;
  height: 20px;
  background-color: #e5e5e5;
  margin: 0 8px;
}

.note-actions {
  display: flex;
  gap: 4px;
}

.empty-state {
  font-size: 14px;
  color: #999;
  margin-left: 8px;
}

.action-button {
  width: 32px;
  height: 32px;
  background-color: transparent;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  font-size: 14px;
}

.action-button:hover {
  background-color: #f0f0f0;
  border-color: #d0d0d0;
}

.action-button.active {
  background-color: #007acc;
  border-color: #007acc;
  color: white;
}

.action-button.delete-button:hover {
  background-color: #ff4444;
  border-color: #ff4444;
  color: white;
}

.action-button[title]:hover::after {
  content: attr(title);
  position: absolute;
  bottom: -25px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  white-space: nowrap;
  z-index: 1000;
}
</style>