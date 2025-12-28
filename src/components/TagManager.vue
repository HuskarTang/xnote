<template>
  <div v-if="visible" class="tag-manager-overlay" @click="handleOverlayClick">
    <div class="tag-manager-dialog" @click.stop>
      <div class="dialog-header">
        <h3>标签管理</h3>
        <button class="close-button" @click="closeDialog">×</button>
      </div>
      
      <div class="dialog-content">
        <!-- 添加新标签 -->
        <div class="add-tag-section">
          <div class="input-group">
            <input
              ref="tagInputRef"
              v-model="newTagName"
              type="text"
              placeholder="输入标签名..."
              class="tag-input"
              @keyup.enter="addTag"
              @input="handleInputChange"
            />
            <button 
              class="add-button"
              :disabled="!newTagName.trim() || loading"
              @click="addTag"
            >
              添加
            </button>
          </div>
          
          <!-- 标签建议列表 -->
          <div v-if="filteredSuggestions.length > 0" class="suggestions-list">
            <div
              v-for="tag in filteredSuggestions"
              :key="tag.id"
              class="suggestion-item"
              @click="selectSuggestion(tag)"
            >
              <span class="suggestion-name">{{ tag.name }}</span>
              <span class="suggestion-count">({{ tag.note_count }})</span>
            </div>
          </div>
        </div>
        
        <!-- 当前笔记的标签 -->
        <div v-if="currentNoteTags.length > 0" class="current-tags-section">
          <h4>当前标签</h4>
          <div class="tags-list">
            <div
              v-for="tag in currentNoteTags"
              :key="tag.id"
              class="tag-item current-tag"
            >
              <span class="tag-name">{{ tag.name }}</span>
              <button 
                class="remove-tag-button"
                @click="removeTagFromNote(tag.name)"
                title="删除标签"
              >
                ×
              </button>
            </div>
          </div>
        </div>
        
        <!-- 所有标签 -->
        <div v-if="availableTags.length > 0" class="all-tags-section">
          <h4>可用标签</h4>
          <div class="tags-list">
            <div
              v-for="tag in availableTags"
              :key="tag.id"
              class="tag-item available-tag"
              @click="addTagToCurrentNote(tag.name)"
            >
              <span class="tag-name">{{ tag.name }}</span>
              <span class="tag-count">({{ tag.note_count }})</span>
            </div>
          </div>
        </div>
        
        <!-- 加载状态 -->
        <div v-if="loading" class="loading-indicator">
          <span>加载中...</span>
        </div>
        
        <!-- 错误信息 -->
        <div v-if="error" class="error-message">
          {{ error }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { storeToRefs } from 'pinia'
import { useTagsStore } from '@/stores/tags'
import type { Tag } from '@/types'

// Props
const props = defineProps<{
  visible: boolean
  noteId: string | null
}>()

// Emits
const emit = defineEmits<{
  close: []
  tagAdded: [tag: Tag]
  tagRemoved: [tagName: string]
}>()

const tagsStore = useTagsStore()
const { tags, loading, error } = storeToRefs(tagsStore)

// Reactive data
const newTagName = ref('')
const currentNoteTags = ref<Tag[]>([])
const tagInputRef = ref<HTMLInputElement>()

// 过滤建议标签（基于输入内容）
const filteredSuggestions = computed(() => {
  if (!newTagName.value.trim()) return []
  
  const query = newTagName.value.toLowerCase()
  const currentTagNames = currentNoteTags.value.map(tag => tag.name.toLowerCase())
  
  return tags.value
    .filter(tag => 
      tag.name.toLowerCase().includes(query) && 
      !currentTagNames.includes(tag.name.toLowerCase())
    )
    .slice(0, 5) // 最多显示5个建议
})

// 可用标签（排除当前笔记已有的标签）
const availableTags = computed(() => {
  if (!currentNoteTags.value.length) return tags.value
  
  const currentTagNames = currentNoteTags.value.map(tag => tag.name.toLowerCase())
  return tags.value.filter(tag => !currentTagNames.includes(tag.name.toLowerCase()))
})

// 处理输入变化
const handleInputChange = () => {
  // 可以在这里实现实时搜索
}

// 选择建议标签
const selectSuggestion = (tag: Tag) => {
  newTagName.value = tag.name
  addTag()
}

// 添加标签
const addTag = async () => {
  if (!newTagName.value.trim() || !props.noteId) return
  
  const tagName = newTagName.value.trim()
  const tag = await tagsStore.addTagToNote(props.noteId, tagName)
  
  if (tag) {
    newTagName.value = ''
    await loadCurrentNoteTags()
    emit('tagAdded', tag)
  }
}

// 从笔记移除标签
const removeTagFromNote = async (tagName: string) => {
  if (!props.noteId) return
  
  const success = await tagsStore.removeTagFromNote(props.noteId, tagName)
  if (success) {
    await loadCurrentNoteTags()
    emit('tagRemoved', tagName)
  }
}

// 添加标签到当前笔记
const addTagToCurrentNote = async (tagName: string) => {
  if (!props.noteId) return
  
  const tag = await tagsStore.addTagToNote(props.noteId, tagName)
  if (tag) {
    await loadCurrentNoteTags()
    emit('tagAdded', tag)
  }
}

// 加载当前笔记的标签
const loadCurrentNoteTags = async () => {
  if (!props.noteId) {
    currentNoteTags.value = []
    return
  }
  
  currentNoteTags.value = await tagsStore.getNoteTags(props.noteId)
}

// 关闭弹窗
const closeDialog = () => {
  newTagName.value = ''
  emit('close')
}

// 处理遮罩点击
const handleOverlayClick = (event: MouseEvent) => {
  if (event.target === event.currentTarget) {
    closeDialog()
  }
}

// 监听弹窗显示状态
watch(() => props.visible, async (visible) => {
  if (visible) {
    await tagsStore.loadTags()
    await loadCurrentNoteTags()
    // 聚焦输入框
    nextTick(() => {
      tagInputRef.value?.focus()
    })
  }
})

// 监听笔记ID变化
watch(() => props.noteId, async () => {
  if (props.visible) {
    await loadCurrentNoteTags()
  }
})
</script>

<style scoped>
.tag-manager-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.tag-manager-dialog {
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e5e5;
  background-color: #f8f9fa;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.close-button {
  background: none;
  border: none;
  font-size: 24px;
  color: #666;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.close-button:hover {
  background-color: #e9ecef;
  color: #333;
}

.dialog-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.add-tag-section {
  margin-bottom: 24px;
  position: relative;
}

.input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.tag-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.tag-input:focus {
  border-color: #007acc;
  box-shadow: 0 0 0 3px rgba(0, 122, 204, 0.1);
}

.add-button {
  padding: 8px 16px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: background-color 0.2s;
  white-space: nowrap;
}

.add-button:hover:not(:disabled) {
  background-color: #0066b3;
}

.add-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.suggestions-list {
  background: white;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  max-height: 150px;
  overflow-y: auto;
}

.suggestion-item {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  transition: background-color 0.2s;
}

.suggestion-item:hover {
  background-color: #f8f9fa;
}

.suggestion-item:not(:last-child) {
  border-bottom: 1px solid #f0f0f0;
}

.suggestion-name {
  font-size: 14px;
  color: #333;
}

.suggestion-count {
  font-size: 12px;
  color: #666;
}

.current-tags-section,
.all-tags-section {
  margin-bottom: 20px;
}

.current-tags-section h4,
.all-tags-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.tags-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag-item {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  border-radius: 16px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  gap: 6px;
}

.current-tag {
  background-color: #007acc;
  color: white;
}

.current-tag:hover {
  background-color: #0066b3;
}

.available-tag {
  background-color: #f1f3f4;
  color: #333;
  border: 1px solid #e5e5e5;
}

.available-tag:hover {
  background-color: #e9ecef;
  border-color: #d1d5db;
}

.tag-name {
  font-size: 14px;
}

.tag-count {
  font-size: 12px;
  opacity: 0.7;
}

.remove-tag-button {
  background: none;
  border: none;
  color: inherit;
  cursor: pointer;
  padding: 0;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  font-size: 14px;
  transition: background-color 0.2s;
}

.remove-tag-button:hover {
  background-color: rgba(255, 255, 255, 0.2);
}

.loading-indicator {
  text-align: center;
  padding: 20px;
  color: #666;
  font-size: 14px;
}

.error-message {
  background-color: #fee;
  color: #c33;
  padding: 12px;
  border-radius: 6px;
  font-size: 14px;
  margin-top: 12px;
  border: 1px solid #fcc;
}

/* 滚动条样式 */
.dialog-content::-webkit-scrollbar,
.suggestions-list::-webkit-scrollbar {
  width: 6px;
}

.dialog-content::-webkit-scrollbar-track,
.suggestions-list::-webkit-scrollbar-track {
  background: transparent;
}

.dialog-content::-webkit-scrollbar-thumb,
.suggestions-list::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 3px;
}

.dialog-content::-webkit-scrollbar-thumb:hover,
.suggestions-list::-webkit-scrollbar-thumb:hover {
  background-color: #9ca3af;
}
</style>