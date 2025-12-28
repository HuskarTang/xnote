<template>
  <div class="action-bar" data-tauri-drag-region>
    <!-- 视图模式切换 -->
    <div class="mode-buttons" data-tauri-drag-region="false">
      <button 
        class="action-button"
        :class="{ active: viewMode === 'view' }"
        @click="setViewMode('view')"
        title="View mode"
        data-tauri-drag-region="false"
      >
        👁️
      </button>
      <button 
        class="action-button"
        :class="{ active: viewMode === 'edit' }"
        @click="setViewMode('edit')"
        title="Edit mode"
        data-tauri-drag-region="false"
      >
        ✏️
      </button>
      <button 
        class="action-button"
        :class="{ active: viewMode === 'split' }"
        @click="setViewMode('split')"
        title="Split mode"
        data-tauri-drag-region="false"
      >
        📱
      </button>
    </div>

    <!-- 分隔线 -->
    <div class="separator"></div>

    <!-- 笔记操作 -->
    <div class="note-actions" v-if="currentNote" data-tauri-drag-region="false">
      <button 
        class="action-button"
        :class="{ active: currentNote.is_favorite }"
        @click="toggleFavorite"
        title="Toggle favorite"
        data-tauri-drag-region="false"
      >
        ⭐
      </button>
      <button 
        class="action-button"
        @click="showTagManager"
        title="Manage tags"
        data-tauri-drag-region="false"
      >
        🏷️
      </button>
      <button 
        class="action-button"
        @click="showAttachmentManager"
        title="Manage attachments"
        data-tauri-drag-region="false"
      >
        📎
      </button>
      <button 
        class="action-button delete-button"
        :class="{ 'permanent-delete': isInTrashView }"
        @click="deleteNote"
        :title="deleteButtonTitle"
        data-tauri-drag-region="false"
      >
        {{ deleteButtonIcon }}
      </button>
      <!-- 恢复按钮 (仅在垃圾桶视图显示) -->
      <button 
        v-if="isInTrashView"
        class="action-button restore-button"
        @click="restoreNote"
        title="Restore note"
        data-tauri-drag-region="false"
      >
        ↩️
      </button>
      <button 
        class="action-button"
        @click="exportNote"
        title="Export note"
        data-tauri-drag-region="false"
      >
        📤
      </button>
    </div>

    <!-- 空状态提示 -->
    <div v-else class="empty-state">
      Select a note to view actions
    </div>

    <!-- 标签管理弹窗 -->
    <TagManager
      :visible="tagManagerVisible"
      :note-id="currentNote?.id || null"
      @close="tagManagerVisible = false"
      @tag-added="onTagAdded"
      @tag-removed="onTagRemoved"
    />

    <!-- 附件管理弹窗 -->
    <AttachmentManager
      :visible="attachmentManagerVisible"
      :note-id="currentNote?.id || null"
      @close="attachmentManagerVisible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { useNotesStore } from '@/stores/notes'
import { useTagsStore } from '@/stores/tags'
import { ask, message } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
import TagManager from './TagManager.vue'
import AttachmentManager from './AttachmentManager.vue'
import type { Tag } from '@/types'

const appStore = useAppStore()
const notesStore = useNotesStore()
const tagsStore = useTagsStore()

const { viewMode } = storeToRefs(appStore)
const { currentNote } = storeToRefs(notesStore)
const { selectedTag } = storeToRefs(tagsStore)

// 标签管理器可见性
const tagManagerVisible = ref(false)
// 附件管理器可见性  
const attachmentManagerVisible = ref(false)

// 计算当前是否在垃圾桶视图
const isInTrashView = computed(() => selectedTag.value === 'Trash')

// 计算删除按钮的图标和标题
const deleteButtonIcon = computed(() => isInTrashView.value ? '🔥' : '🗑️')
const deleteButtonTitle = computed(() => isInTrashView.value ? 'Permanently delete note' : 'Move to trash')

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
  
  const isTrash = isInTrashView.value
  const actionText = isTrash ? 'permanently delete' : 'delete'
  const warningText = isTrash 
    ? 'This action cannot be undone. The note and its file will be permanently deleted.'
    : 'The note will be moved to trash.'
  
  const confirmed = await ask(
    `Are you sure you want to ${actionText} "${currentNote.value.title}"?\n${warningText}`,
    { title: 'XNote', type: 'warning' }
  )
  
  if (confirmed) {
    try {
      if (isTrash) {
        await notesStore.permanentlyDeleteNote(currentNote.value.id)
      } else {
        await notesStore.deleteNote(currentNote.value.id)
      }
    } catch (err) {
      console.error(`Failed to ${actionText} note:`, err)
    }
  }
}

const restoreNote = async () => {
  if (!currentNote.value) return
  
  try {
    await notesStore.restoreNote(currentNote.value.id)
  } catch (err) {
    console.error('Failed to restore note:', err)
  }
}

const showTagManager = () => {
  tagManagerVisible.value = true
}

const onTagAdded = async (tag: Tag) => {
  console.log('Tag added:', tag.name)
  // 刷新当前笔记以更新标签显示
  if (currentNote.value) {
    await notesStore.refreshNote(currentNote.value.id)
  }
  // 刷新标签列表统计
  await tagsStore.loadTags()
}

const onTagRemoved = async (tagName: string) => {
  console.log('Tag removed:', tagName)
  // 刷新当前笔记以更新标签显示
  if (currentNote.value) {
    await notesStore.refreshNote(currentNote.value.id)
  }
  // 刷新标签列表统计
  await tagsStore.loadTags()
}

// 显示附件管理器
const showAttachmentManager = () => {
  attachmentManagerVisible.value = true
}

const exportNote = async () => {
  if (!currentNote.value) return
  
  try {
    // Show directory selection dialog
    const exportPath = await invoke('show_export_dialog')
    
    if (!exportPath) {
      // User cancelled the dialog
      return
    }
    
    // Export the note
    const result = await invoke('export_note', {
      noteId: currentNote.value.id,
      exportPath: exportPath
    })
    
    // Show success message
    await message(result, {
      title: 'Export Successful',
      type: 'info'
    })
    
  } catch (err) {
    console.error('Failed to export note:', err)
    
    // Show error message
    await message(
      `Failed to export note: ${err}`,
      {
        title: 'Export Failed',
        type: 'error'
      }
    )
  }
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

.action-button.delete-button.permanent-delete:hover {
  background-color: #cc0000;
  border-color: #cc0000;
  color: white;
}

.action-button.restore-button:hover {
  background-color: #28a745;
  border-color: #28a745;
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