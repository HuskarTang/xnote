<template>
  <div class="note-pane-container">
    <!-- 搜索和新建区域 -->
    <div class="note-pane-header" data-tauri-drag-region>
      <div class="search-container" data-tauri-drag-region="false">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search..."
          class="search-input"
          @input="handleSearch"
          @focus="onSearchFocus"
          @blur="onSearchBlur"
          @click="onSearchClick"
          @keydown="onSearchKeydown"
          @keyup="handleSearch"
          data-tauri-drag-region="false"
          ref="searchInput"
          tabindex="0"
        />
        <button 
          v-if="searchQuery" 
          class="search-button clear-button" 
          data-tauri-drag-region="false" 
          @click="clearSearch"
          title="Clear search"
        >
          <Icons name="close" :size="16" />
        </button>
        <button 
          v-else
          class="search-button" 
          data-tauri-drag-region="false" 
          @click="focusSearch"
          title="Search"
        >
          <Icons name="search" :size="16" />
        </button>
      </div>
      <button 
        class="new-note-button"
        @click="handleCreateNote"
        title="Create new note"
        data-tauri-drag-region="false"
      >
        <Icons name="add" :size="16" />
      </button>
    </div>

    <!-- 笔记列表 -->
    <div class="note-list">
      <div v-if="loading" class="loading">
        Loading...
      </div>
      
      <div v-else-if="error" class="error">
        {{ error }}
      </div>
      
      <div v-else-if="sortedNotes.length === 0" class="empty">
        No notes found
      </div>
      
      <div 
        v-for="note in sortedNotes" 
        :key="note.id"
        class="note-item"
        :class="{ active: currentNote?.id === note.id }"
        @click="selectNote(note)"
      >
        <div class="note-header">
          <h3 class="note-title">{{ note.title || 'Untitled' }}</h3>
          <div class="note-icons">
            <Icons v-if="note.is_favorite" name="favorite" :size="12" class="favorite-icon" title="Favorite" />
            <Icons v-if="note.has_attachments" name="attachment" :size="12" class="attachment-icon" title="Has attachments" />
          </div>
        </div>
        
        <div class="note-preview">
          {{ getPreview(note.content) }}
        </div>
        
        <div class="note-meta">
          <span class="note-date">{{ formatDate(note.modified_at) }}</span>
          <div class="note-tags">
            <span 
              v-for="tag in note.tags.slice(0, 3)" 
              :key="tag"
              class="note-tag"
            >
              {{ tag }}
            </span>
            <span v-if="note.tags.length > 3" class="note-tag-more">
              +{{ note.tags.length - 3 }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useNotesStore } from '@/stores/notes'
import { useAppStore } from '@/stores/app'
import { useTagsStore } from '@/stores/tags'
import Icons from '@/components/Icons.vue'
import type { Note } from '@/types'

const notesStore = useNotesStore()
const appStore = useAppStore()
const tagsStore = useTagsStore()

const { sortedNotes, currentNote, loading, error } = storeToRefs(notesStore)
const { selectedTag } = storeToRefs(tagsStore)
const searchQuery = ref('')
const searchInput = ref<HTMLInputElement | null>(null)
let searchTimeout: NodeJS.Timeout | null = null

// 处理搜索（带防抖）
const handleSearch = () => {
  // 清除之前的定时器
  if (searchTimeout) {
    clearTimeout(searchTimeout)
  }
  
  // 设置新的定时器
  searchTimeout = setTimeout(async () => {
    const query = searchQuery.value.trim()
    if (query) {
      await notesStore.searchNotes(query, selectedTag.value !== 'All Notes' ? selectedTag.value : undefined)
    } else {
      // 清空搜索时重新加载当前标签的笔记
      await reloadCurrentTagNotes()
    }
  }, 300) // 300ms 防抖延迟
}

// 清空搜索
const clearSearch = async () => {
  // 清除防抖定时器
  if (searchTimeout) {
    clearTimeout(searchTimeout)
    searchTimeout = null
  }
  
  searchQuery.value = ''
  await reloadCurrentTagNotes()
}

// 重新加载当前标签的笔记
const reloadCurrentTagNotes = async () => {
  switch (selectedTag.value) {
    case 'All Notes':
      await notesStore.loadNotes()
      break
    case 'Favorites':
      await notesStore.loadFavorites()
      break
    case 'Untagged':
      await notesStore.loadUntagged()
      break
    case 'Trash':
      await notesStore.loadTrash()
      break
    default:
      await notesStore.loadNotesByTag(selectedTag.value)
      break
  }
}

// 监听笔记创建事件
const handleNoteCreated = () => {
  // 重新加载当前标签的笔记以显示新创建的笔记
  reloadCurrentTagNotes()
}

onMounted(() => {
  // 添加事件监听器
  window.addEventListener('note-created', handleNoteCreated)
})

onUnmounted(() => {
  // 移除事件监听器
  window.removeEventListener('note-created', handleNoteCreated)
  
  // 清理搜索定时器
  if (searchTimeout) {
    clearTimeout(searchTimeout)
    searchTimeout = null
  }
})

// 选择笔记
const selectNote = (note: Note) => {
  notesStore.setCurrentNote(note)
}

// 创建新笔记
const handleCreateNote = async () => {
  try {
    const specialTags = ['All Notes', 'Favorites', 'Untagged', 'Trash', 'Tags']
    const tags = !specialTags.includes(selectedTag.value) ? [selectedTag.value] : []
    
    const newNote = await notesStore.createNote('Untitled', '', tags)
    appStore.setViewMode('edit')
  } catch (err) {
    console.error('Failed to create note:', err)
  }
}

// 获取内容预览
const getPreview = (content: string) => {
  if (!content) return 'No content'
  
  // 移除 Markdown 语法
  const plainText = content
    .replace(/^#+\s+/gm, '') // 标题
    .replace(/\*\*(.*?)\*\*/g, '$1') // 粗体
    .replace(/\*(.*?)\*/g, '$1') // 斜体
    .replace(/`(.*?)`/g, '$1') // 行内代码
    .replace(/\[([^\]]+)\]\([^\)]+\)/g, '$1') // 链接
    .replace(/!\[([^\]]*)\]\([^\)]+\)/g, '$1') // 图片
    .replace(/^\s*[-*+]\s+/gm, '') // 列表
    .replace(/^\s*\d+\.\s+/gm, '') // 有序列表
    .replace(/\n\s*\n/g, ' ') // 多个换行
    .replace(/\n/g, ' ') // 单个换行
    .trim()
  
  return plainText.length > 100 ? plainText.substring(0, 100) + '...' : plainText
}

// 格式化日期
const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60)
  
  if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } else if (diffInHours < 24 * 7) {
    return date.toLocaleDateString([], { weekday: 'short' })
  } else {
    return date.toLocaleDateString([], { month: 'short', day: 'numeric' })
  }
}

// 监听标签变化，清空搜索
watch(selectedTag, () => {
  searchQuery.value = ''
})

// 搜索框焦点处理
const onSearchFocus = () => {
  console.log('Search input focused')
}

const onSearchBlur = () => {
  console.log('Search input blurred')
}

const onSearchClick = (event: Event) => {
  console.log('Search input clicked')
  event.stopPropagation()
  if (searchInput.value) {
    searchInput.value.focus()
  }
}

const onSearchKeydown = (event: KeyboardEvent) => {
  console.log('Search keydown:', event.key)
  event.stopPropagation()
}

const focusSearch = () => {
  console.log('Focus search button clicked')
  if (searchInput.value) {
    searchInput.value.focus()
    searchInput.value.select()
  }
}
</script>

<style scoped>
.note-pane-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}

.note-pane-header {
  display: flex;
  align-items: center;
  padding: 0 12px;
  height: 48px;
  border-bottom: 1px solid #e5e5e5;
  gap: 8px;
  pointer-events: auto;
}

.search-container {
  flex: 1;
  position: relative;
  display: flex;
  align-items: center;
  pointer-events: auto;
  z-index: 1;
}

.search-input {
  width: 100%;
  padding: 8px 32px 8px 12px;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
  background-color: #ffffff;
  color: #333;
  cursor: text;
  user-select: text;
  pointer-events: auto;
}

.search-input:focus {
  border-color: #007acc;
  box-shadow: 0 0 0 2px rgba(0, 122, 204, 0.2);
}

.search-input:hover {
  border-color: #ccc;
}

.search-button {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  padding: 4px;
  color: #666;
  border-radius: 3px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
}

.search-button:hover {
  background-color: #f0f0f0;
}

.clear-button {
  color: #999;
  font-size: 16px;
  font-weight: bold;
  width: 24px;
  height: 24px;
}

.clear-button:hover {
  background-color: #ff4444;
  color: white;
}

.new-note-button {
  width: 32px;
  height: 32px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 18px;
  font-weight: bold;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.2s;
}

.new-note-button:hover {
  background-color: #0066b3;
}

.note-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.loading, .error, .empty {
  text-align: center;
  padding: 40px 20px;
  color: #666;
}

.note-item {
  padding: 12px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: background-color 0.2s;
}

.note-item:hover {
  background-color: #f8f8f8;
}

.note-item.active {
  background-color: #e3f2fd;
  border-left: 3px solid #007acc;
}

.note-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 6px;
}

.note-title {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: #333;
  flex: 1;
  margin-right: 8px;
  line-height: 1.3;
}

.note-icons {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
}

.favorite-icon,
.attachment-icon {
  font-size: 12px;
  opacity: 0.7;
}

.note-preview {
  font-size: 13px;
  color: #666;
  line-height: 1.4;
  margin-bottom: 8px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.note-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: #999;
}

.note-date {
  flex-shrink: 0;
}

.note-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  justify-content: flex-end;
  max-width: 60%;
}

.note-tag {
  background-color: #f0f0f0;
  padding: 1px 6px;
  border-radius: 8px;
  font-size: 10px;
  color: #666;
  white-space: nowrap;
}

.note-tag-more {
  color: #999;
  font-size: 10px;
}

/* 滚动条样式 */
.note-list::-webkit-scrollbar {
  width: 6px;
}

.note-list::-webkit-scrollbar-track {
  background: transparent;
}

.note-list::-webkit-scrollbar-thumb {
  background-color: #e0e0e0;
  border-radius: 3px;
}

.note-list::-webkit-scrollbar-thumb:hover {
  background-color: #c0c0c0;
}
</style>