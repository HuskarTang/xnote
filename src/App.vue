<template>
  <div id="app" class="app-container">
    <SetupPage v-if="showSetup" />
    <div v-else class="app-layout">
      <!-- TagPane - 左侧标签面板 -->
      <div 
        class="tag-pane"
        :style="{ width: sidebarWidth + 'px' }"
      >
        <TagPane ref="tagPaneRef" />
      </div>
      
      <!-- 分割线 -->
      <div 
        class="resize-handle resize-handle-left"
        @mousedown="startResize('left', $event)"
      ></div>
      
      <!-- NotePane - 中间笔记列表面板 -->
      <div 
        class="note-pane"
        :style="{ width: notePaneWidth + 'px' }"
      >
        <NotePane />
      </div>
      
      <!-- 分割线 -->
      <div 
        class="resize-handle resize-handle-right"
        @mousedown="startResize('right', $event)"
      ></div>
      
      <!-- ContentPane - 右侧内容展示区 -->
      <div class="content-pane">
        <ContentPane />
      </div>
    </div>
    
    <!-- 设置对话框 -->
    <SettingsPane v-if="showSettings" @close="showSettings = false" @saved="onSettingsSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, markRaw } from 'vue'
import { storeToRefs } from 'pinia'
import { useAppStore } from '@/stores/app'
import { useNotesStore } from '@/stores/notes'
import { useTagsStore } from '@/stores/tags'
import TagPane from '@/components/TagPane.vue'
import NotePane from '@/components/NotePane.vue'
import ContentPane from '@/components/ContentPane.vue'
import SetupPage from '@/components/SetupPage.vue'
import SettingsPane from '@/components/SettingsPane.vue'
import { api } from '@/utils/api'
import { listen } from '@tauri-apps/api/event'
import { ElMessageBox } from 'element-plus'
import { InfoFilled } from '@element-plus/icons-vue'

const appStore = useAppStore()
const notesStore = useNotesStore()
const tagsStore = useTagsStore()

const { sidebarWidth, notePaneWidth } = storeToRefs(appStore)
const showSetup = ref(false)
const showSettings = ref(false)
const tagPaneRef = ref<InstanceType<typeof TagPane>>()

// 拖拽调整大小相关
const isResizing = ref(false)
const resizeType = ref<'left' | 'right' | null>(null)
const startX = ref(0)
const startSidebarWidth = ref(0)
const startNotePaneWidth = ref(0)

// 开始调整大小
const startResize = (type: 'left' | 'right', event: MouseEvent) => {
  isResizing.value = true
  resizeType.value = type
  startX.value = event.clientX
  startSidebarWidth.value = sidebarWidth.value
  startNotePaneWidth.value = notePaneWidth.value
  
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

// 处理调整大小
const handleResize = (event: MouseEvent) => {
  if (!isResizing.value || !resizeType.value) return
  
  const deltaX = event.clientX - startX.value
  
  if (resizeType.value === 'left') {
    // 调整左侧边栏宽度
    const newWidth = Math.max(180, Math.min(400, startSidebarWidth.value + deltaX))
    appStore.setSidebarWidth(newWidth)
  } else if (resizeType.value === 'right') {
    // 调整中间笔记列表宽度
    const newWidth = Math.max(280, Math.min(500, startNotePaneWidth.value + deltaX))
    appStore.setNotePaneWidth(newWidth)
  }
}

// 停止调整大小
const stopResize = () => {
  isResizing.value = false
  resizeType.value = null
  
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

// 设置保存回调
const onSettingsSaved = () => {
  // 刷新TagPane统计数据和同步配置
  if (tagPaneRef.value) {
    tagPaneRef.value.refreshStatistics()
    tagPaneRef.value.checkGitSyncConfig()
  }
}

onMounted(async () => {
  try {
    // Check if setup is required
    const setupRequired = await api.isSetupRequired()
    if (setupRequired) {
      showSetup.value = true
      return
    }
    
    // 初始化数据
    await tagsStore.loadTags()
    await notesStore.loadNotes()
    
    // 自动选择 All Notes 并显示第一篇笔记
    tagsStore.setSelectedTag('All Notes')
    if (notesStore.sortedNotes.length > 0) {
      notesStore.setCurrentNote(notesStore.sortedNotes[0])
    }
    
    // 监听菜单事件
    setupMenuListeners()
  } catch (error) {
    console.error('Failed to initialize app:', error)
    // 默认显示主界面
  }
})

// 设置菜单事件监听器
const setupMenuListeners = async () => {
  try {
    // 监听偏好设置菜单事件
    await listen('open-preferences', () => {
      showSettings.value = true
    })
    
    // 监听关于菜单事件
    await listen('show-about', () => {
      ElMessageBox.alert(
        'XNote 是一个超轻量、完全基于文件的 Markdown 笔记应用， 支持通过git仓库进行远程协同<br/>'
          + 'Version: 0.2.0<br/>'
          + 'Contact: tanly6@chinatelecom.cn',
        '关于 XNote',
        {
          confirmButtonText: '确定',
          icon: markRaw(InfoFilled),
          center: true,
          draggable: true,
          dangerouslyUseHTMLString: true // 允许使用HTML字符串
        }
      )
    })
  } catch (error) {
    console.error('Failed to setup menu listeners:', error)
  }
}

onUnmounted(() => {
  // 清理事件监听器
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
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
  display: flex;
  flex-direction: column;
}

.tag-pane-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #3d3d3d;
  background-color: #2d2d2d;
}

.app-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #ffffff;
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
  position: relative;
  flex-shrink: 0;
}

.resize-handle:hover {
  background-color: #007acc;
}

.resize-handle:active {
  background-color: #005a9e;
}

.resize-handle-left {
  background-color: #3d3d3d;
}

.resize-handle-left:hover {
  background-color: #007acc;
}

.resize-handle-left:active {
  background-color: #005a9e;
}

/* 添加视觉提示 */
.resize-handle::before {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 2px;
  height: 20px;
  background-color: rgba(255, 255, 255, 0.3);
  border-radius: 1px;
  opacity: 0;
  transition: opacity 0.2s;
}

.resize-handle:hover::before {
  opacity: 1;
}

.resize-handle-left::before {
  background-color: rgba(255, 255, 255, 0.5);
}
</style>