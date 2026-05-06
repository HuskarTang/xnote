<template>
  <div class="view-pane-container" @scroll="handleScroll">
    <div 
      class="markdown-content"
      v-html="renderedContent"
    ></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useNotesStore } from '@/stores/notes'
import { renderMarkdown } from '@/utils/markdown'

const notesStore = useNotesStore()
const { currentNote } = storeToRefs(notesStore)

// 定义 props
const props = defineProps<{
  content?: string
}>()

// 渲染后的内容
const renderedContent = ref('<div class="empty-preview">No content to preview</div>')

// 渲染内容的函数
const updateRenderedContent = async () => {
  let content = ''
  
  // 如果提供了 content prop（分屏模式），则使用它
  if (props.content !== undefined) {
    console.log('ViewPane: Using props.content', props.content);
    if (!props.content) {
      renderedContent.value = '<div class="empty-preview">No content to preview</div>'
      return
    }
    content = props.content
  } else {
    // 否则使用 currentNote.content（普通模式）
    console.log('ViewPane: Using currentNote.content', currentNote.value?.content);
    if (!currentNote.value?.content) {
      renderedContent.value = '<div class="empty-preview">No content to preview</div>'
      return
    }
    content = currentNote.value.content
  }
  
  try {
    const result = await renderMarkdown(content)
    console.log('ViewPane: Rendered result', result);
    renderedContent.value = result
  } catch (error) {
    console.error('ViewPane: Error rendering markdown', error);
    renderedContent.value = `<pre>${content}</pre>`
  }
}

// 监听内容变化
watch(() => props.content, updateRenderedContent, { immediate: true })
watch(() => currentNote.value?.content, updateRenderedContent, { immediate: true })

// 处理滚动（用于分屏模式同步滚动，暂时预留）
const handleScroll = () => {
  // 可以在这里实现与编辑面板的同步滚动
}
</script>

<style scoped>
.view-pane-container {
  height: 100%;
  overflow-y: auto;
  background-color: #ffffff;
}

.empty-preview {
  text-align: center;
  color: #999;
  font-style: italic;
  padding: 40px 20px;
}

/* 滚动条样式 */
.view-pane-container::-webkit-scrollbar {
  width: 8px;
}

.view-pane-container::-webkit-scrollbar-track {
  background: transparent;
}

.view-pane-container::-webkit-scrollbar-thumb {
  background-color: #e0e0e0;
  border-radius: 4px;
}

.view-pane-container::-webkit-scrollbar-thumb:hover {
  background-color: #c0c0c0;
}
</style>
