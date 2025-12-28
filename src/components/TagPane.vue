<template>
  <div class="tag-pane-container">
    <!-- 标题 -->
    <div class="tag-pane-header">
      <h2 class="app-title">XNote</h2>
    </div>

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
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useTagsStore } from '@/stores/tags'
import { useNotesStore } from '@/stores/notes'

const tagsStore = useTagsStore()
const notesStore = useNotesStore()

const { tags, selectedTag } = storeToRefs(tagsStore)
const { notes } = storeToRefs(notesStore)

const showAllTags = ref(true) // 始终展开用户标签

// 计算各种统计数量
const allNotesCount = computed(() => {
  return notes.value.filter(note => !note.is_deleted).length
})

const favoritesCount = computed(() => {
  return notes.value.filter(note => !note.is_deleted && note.is_favorite).length
})

const untaggedCount = computed(() => {
  return notes.value.filter(note => !note.is_deleted && note.tags.length === 0).length
})

const trashCount = computed(() => {
  return notes.value.filter(note => note.is_deleted).length
})

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

onMounted(() => {
  // 默认选择 All Notes
  selectTag('All Notes')
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
</style>