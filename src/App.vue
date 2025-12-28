<template>
  <div id="app" class="app-container">
    <div class="app-layout">
      <!-- TagPane - 左侧标签面板 -->
      <div 
        class="tag-pane"
        :style="{ width: sidebarWidth + 'px' }"
      >
        <TagPane />
      </div>
      
      <!-- 分割线 -->
      <div class="resize-handle resize-handle-left"></div>
      
      <!-- NotePane - 中间笔记列表面板 -->
      <div 
        class="note-pane"
        :style="{ width: notePaneWidth + 'px' }"
      >
        <NotePane />
      </div>
      
      <!-- 分割线 -->
      <div class="resize-handle resize-handle-right"></div>
      
      <!-- ContentPane - 右侧内容展示区 -->
      <div class="content-pane">
        <ContentPane />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { useNotesStore } from '@/stores/notes'
import { useTagsStore } from '@/stores/tags'
import TagPane from '@/components/TagPane.vue'
import NotePane from '@/components/NotePane.vue'
import ContentPane from '@/components/ContentPane.vue'

const appStore = useAppStore()
const notesStore = useNotesStore()
const tagsStore = useTagsStore()

const { sidebarWidth, notePaneWidth } = storeToRefs(appStore)

onMounted(async () => {
  // 初始化数据
  await tagsStore.loadTags()
  await notesStore.loadNotes()
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100vh;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background-color: #f5f5f5;
  overflow: hidden;
}

.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-layout {
  display: flex;
  flex: 1;
  height: 100vh;
  overflow: hidden;
}

.tag-pane {
  background-color: #2d2d2d;
  color: #ffffff;
  min-width: 180px;
  max-width: 400px;
  border-right: 1px solid #3d3d3d;
}

.note-pane {
  background-color: #ffffff;
  border-right: 1px solid #e5e5e5;
  min-width: 280px;
  max-width: 500px;
}

.content-pane {
  flex: 1;
  background-color: #ffffff;
  min-width: 400px;
}

.resize-handle {
  width: 4px;
  background-color: #e5e5e5;
  cursor: col-resize;
  transition: background-color 0.2s;
}

.resize-handle:hover {
  background-color: #007acc;
}

.resize-handle-left {
  background-color: #3d3d3d;
}

.resize-handle-left:hover {
  background-color: #007acc;
}
</style>