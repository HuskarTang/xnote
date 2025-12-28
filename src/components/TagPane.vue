<template>
  <div class="tag-pane-container">
    <!-- 主要视图 -->
    <div class="tag-sections">
      <!-- All Notes -->
      <div 
        class="tag-item main-tag"
        :class="{ active: selectedTag === 'All Notes' }"
        @click="selectTag('All Notes')"
      >
        <div class="tag-icon">📝</div>
        <div class="tag-content">
          <span class="tag-name">All Notes</span>
          <span class="tag-count">{{ allNotesCount }}</span>
        </div>
      </div>

      <!-- Favorites -->
      <div 
        class="tag-item main-tag"
        :class="{ active: selectedTag === 'Favorites' }"
        @click="selectTag('Favorites')"
      >
        <div class="tag-icon">⭐</div>
        <div class="tag-content">
          <span class="tag-name">Favorites</span>
          <span class="tag-count">{{ favoritesCount }}</span>
        </div>
      </div>

      <!-- Tags Section -->
      <div 
        class="tag-item main-tag"
        :class="{ active: selectedTag === 'Tags' }"
        @click="selectTag('Tags')"
      >
        <div class="tag-icon">🏷️</div>
        <div class="tag-content">
          <span class="tag-name">Tags</span>
          <span class="tag-count">{{ tags.length }}</span>
        </div>
      </div>

      <!-- User Tags (展开显示) -->
      <div v-if="selectedTag === 'Tags' || showAllTags" class="user-tags">
        <div 
          v-for="tag in tags" 
          :key="tag.id"
          class="tag-item user-tag"
          :class="{ active: selectedTag === tag.name }"
          @click="selectTag(tag.name)"
        >
          <div class="tag-icon">📋</div>
          <div class="tag-content">
            <span class="tag-name">{{ tag.name }}</span>
            <span class="tag-count">{{ tag.note_count }}</span>
          </div>
        </div>
      </div>

      <!-- Untagged -->
      <div 
        class="tag-item main-tag"
        :class="{ active: selectedTag === 'Untagged' }"
        @click="selectTag('Untagged')"
      >
        <div class="tag-icon">🔖</div>
        <div class="tag-content">
          <span class="tag-name">Untagged</span>
          <span class="tag-count">{{ untaggedCount }}</span>
        </div>
      </div>

      <!-- Trash -->
      <div 
        class="tag-item main-tag"
        :class="{ active: selectedTag === 'Trash' }"
        @click="selectTag('Trash')"
      >
        <div class="tag-icon">🗑️</div>
        <div class="tag-content">
          <span class="tag-name">Trash</span>
          <span class="tag-count">{{ trashCount }}</span>
        </div>
      </div>
    </div>
    
    <!-- 同步按钮 -->
    <div class="sync-section" v-if="showSyncButton">
      <button class="sync-btn" @click="openSyncDialog" :disabled="!gitSyncEnabled" title="同步">
        🔄
      </button>
    </div>
    
    <!-- 同步对话框 -->
    <SyncDialog v-if="showSyncDialog" @close="closeSyncDialog" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useTagsStore } from '@/stores/tags'
import { useNotesStore } from '@/stores/notes'
import { api } from '@/utils/api'
import SyncDialog from './SyncDialog.vue'

const tagsStore = useTagsStore()
const notesStore = useNotesStore()

const { tags, selectedTag } = storeToRefs(tagsStore)

const showAllTags = ref(true) // 始终展开用户标签

// 统计数据 - 直接从API获取准确数据
const allNotesCount = ref(0)
const favoritesCount = ref(0) 
const untaggedCount = ref(0)
const trashCount = ref(0)

// 同步相关
const showSyncButton = ref(false)
const gitSyncEnabled = ref(false)
const showSyncDialog = ref(false)


// 加载统计数据
const loadStatistics = async () => {
  try {
    const [allNotes, favorites, untagged, trash] = await Promise.all([
      api.getAllNotes(),
      api.getFavorites(), 
      api.getUntagged(),
      api.getTrash()
    ])
    
    allNotesCount.value = allNotes.length
    favoritesCount.value = favorites.length
    untaggedCount.value = untagged.length
    trashCount.value = trash.length
  } catch (error) {
    console.error('Failed to load statistics:', error)
  }
}

// 检查Git同步配置
const checkGitSyncConfig = async () => {
  try {
    const config = await api.getGitSyncConfig()
    if (config && config.enabled && config.repository_url) {
      showSyncButton.value = true
      gitSyncEnabled.value = true
    }
  } catch (error) {
    console.error('Failed to check git sync config:', error)
  }
}

// 打开同步对话框
const openSyncDialog = () => {
  showSyncDialog.value = true
}

// 关闭同步对话框
const closeSyncDialog = () => {
  showSyncDialog.value = false
}

async function selectTag(tagName: string) {
  tagsStore.setSelectedTag(tagName)
  
  // 根据选中的标签加载对应的笔记
  switch (tagName) {
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
    case 'Tags':
      await notesStore.loadNotes() // Tags 视图显示所有笔记
      break
    default:
      // 用户自定义标签
      await notesStore.loadNotesByTag(tagName)
      break
  }
}

// 监听笔记变化，刷新统计
const refreshStatistics = () => {
  loadStatistics()
  tagsStore.loadTags()
}

onMounted(async () => {
  // 加载标签和统计数据
  await Promise.all([
    tagsStore.loadTags(),
    loadStatistics(),
    checkGitSyncConfig()
  ])
  
  // 监听笔记变化事件
  window.addEventListener('note-created', refreshStatistics)
  window.addEventListener('note-updated', refreshStatistics)
  window.addEventListener('note-deleted', refreshStatistics)
  window.addEventListener('note-permanently-deleted', refreshStatistics)
  window.addEventListener('note-restored', refreshStatistics)
})

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('note-created', refreshStatistics)
  window.removeEventListener('note-updated', refreshStatistics)
  window.removeEventListener('note-deleted', refreshStatistics)
  window.removeEventListener('note-permanently-deleted', refreshStatistics)
  window.removeEventListener('note-restored', refreshStatistics)
})

// 暴露刷新方法给外部组件使用
defineExpose({
  refreshStatistics,
  checkGitSyncConfig
})
</script>

<style scoped>
.tag-pane-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #2d2d2d;
  color: #ffffff;
}

.tag-pane-header {
  padding: 16px;
  border-bottom: 1px solid #3d3d3d;
}

.app-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #ffffff;
}

.tag-sections {
  flex: 1;
  padding: 8px 0;
  overflow-y: auto;
}

.tag-item {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
  user-select: none;
}

.tag-item:hover {
  background-color: #3d3d3d;
}

.tag-item.active {
  background-color: #007acc;
}

.tag-item.active:hover {
  background-color: #0066b3;
}

.main-tag {
  font-weight: 500;
}

.user-tag {
  margin-left: 16px;
  font-size: 14px;
  color: #cccccc;
}

.user-tag .tag-icon {
  font-size: 14px;
}

.tag-icon {
  font-size: 16px;
  margin-right: 8px;
  min-width: 20px;
  text-align: center;
}

.tag-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex: 1;
}

.tag-name {
  flex: 1;
}

.tag-count {
  font-size: 12px;
  color: #999999;
  background-color: #404040;
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
}

.tag-item.active .tag-count {
  background-color: rgba(255, 255, 255, 0.2);
  color: #ffffff;
}

/* 滚动条样式 */
.tag-sections::-webkit-scrollbar {
  width: 6px;
}

.tag-sections::-webkit-scrollbar-track {
  background: transparent;
}

.tag-sections::-webkit-scrollbar-thumb {
  background-color: #404040;
  border-radius: 3px;
}

.tag-sections::-webkit-scrollbar-thumb:hover {
  background-color: #4d4d4d;
}

/* 同步按钮样式 */
.sync-section {
  padding: 8px 16px;
  border-top: 1px solid #3d3d3d;
  display: flex;
  justify-content: flex-start;
}

.sync-btn {
  background: none;
  border: none;
  color: #cccccc;
  cursor: pointer;
  font-size: 16px;
  padding: 4px;
  border-radius: 4px;
  transition: color 0.2s, background-color 0.2s;
}

.sync-btn:hover:not(:disabled) {
  color: #ffffff;
  background-color: #3d3d3d;
}

.sync-btn:disabled {
  color: #666666;
  cursor: not-allowed;
}

</style>