<template>
  <div class="view-pane-container">
    <div 
      class="markdown-content"
      v-html="renderedContent"
      @scroll="handleScroll"
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
const renderedContent = computed(() => {
  // 如果提供了 content prop（分屏模式），则使用它
  if (props.content !== undefined) {
    if (!props.content) {
      return '<div class="empty-preview">No content to preview</div>'
    }
    return renderMarkdown(props.content)
  }
  
  // 否则使用 currentNote.content（普通模式）
  if (!currentNote.value?.content) {
    return '<div class="empty-preview">No content to preview</div>'
  }
  
  return renderMarkdown(currentNote.value.content)
})

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

.markdown-content {
  padding: 16px 24px;
  max-width: none;
  line-height: 1.6;
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

.empty-preview {
  text-align: center;
  color: #999;
  font-style: italic;
  padding: 40px 20px;
}

/* Markdown 样式 */
.markdown-content :deep(h1) {
  font-size: 2em;
  font-weight: 600;
  margin: 24px 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 2px solid #e5e5e5;
  color: #333;
}

.markdown-content :deep(h2) {
  font-size: 1.5em;
  font-weight: 600;
  margin: 20px 0 12px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid #e5e5e5;
  color: #333;
}

.markdown-content :deep(h3) {
  font-size: 1.25em;
  font-weight: 600;
  margin: 16px 0 8px 0;
  color: #333;
}

.markdown-content :deep(h4) {
  font-size: 1.1em;
  font-weight: 600;
  margin: 16px 0 8px 0;
  color: #333;
}

.markdown-content :deep(h5) {
  font-size: 1em;
  font-weight: 600;
  margin: 16px 0 8px 0;
  color: #333;
}

.markdown-content :deep(h6) {
  font-size: 0.9em;
  font-weight: 600;
  margin: 16px 0 8px 0;
  color: #666;
}

.markdown-content :deep(p) {
  margin: 12px 0;
  line-height: 1.7;
}

.markdown-content :deep(blockquote) {
  margin: 16px 0;
  padding: 8px 16px;
  background-color: #f8f9fa;
  border-left: 4px solid #007acc;
  color: #666;
  font-style: italic;
}

.markdown-content :deep(blockquote p) {
  margin: 8px 0;
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  margin: 12px 0;
  padding-left: 24px;
}

.markdown-content :deep(li) {
  margin: 4px 0;
  line-height: 1.6;
}

.markdown-content :deep(li p) {
  margin: 4px 0;
}

.markdown-content :deep(code) {
  background-color: #f1f3f4;
  padding: 2px 4px;
  border-radius: 3px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 0.9em;
  color: #e74c3c;
}

.markdown-content :deep(.code-block) {
  background-color: #f8f8f8;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  padding: 16px;
  margin: 16px 0;
  overflow-x: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.4;
}

.markdown-content :deep(.code-block code) {
  background: none;
  padding: 0;
  color: inherit;
  border-radius: 0;
}

.markdown-content :deep(.table-container) {
  margin: 16px 0;
  overflow-x: auto;
}

.markdown-content :deep(.markdown-table) {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid #e5e5e5;
}

.markdown-content :deep(.markdown-table th),
.markdown-content :deep(.markdown-table td) {
  border: 1px solid #e5e5e5;
  padding: 8px 12px;
  text-align: left;
}

.markdown-content :deep(.markdown-table th) {
  background-color: #f8f9fa;
  font-weight: 600;
}

.markdown-content :deep(.markdown-table tr:nth-child(even)) {
  background-color: #f9f9f9;
}

.markdown-content :deep(img) {
  max-width: 100%;
  height: auto;
  margin: 8px 0;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.markdown-content :deep(a) {
  color: #007acc;
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(hr) {
  margin: 24px 0;
  border: none;
  border-top: 1px solid #e5e5e5;
}

.markdown-content :deep(del) {
  color: #999;
}

.markdown-content :deep(strong) {
  font-weight: 600;
  color: #333;
}

.markdown-content :deep(em) {
  font-style: italic;
  color: #666;
}

/* Highlight.js 样式 */
.markdown-content :deep(.hljs) {
  background: #f8f8f8;
  color: #333;
}

.markdown-content :deep(.hljs-keyword) {
  color: #007acc;
  font-weight: bold;
}

.markdown-content :deep(.hljs-string) {
  color: #22863a;
}

.markdown-content :deep(.hljs-comment) {
  color: #6f42c1;
  font-style: italic;
}

.markdown-content :deep(.hljs-number) {
  color: #005cc5;
}

.markdown-content :deep(.hljs-function) {
  color: #6f42c1;
}

.markdown-content :deep(.hljs-tag) {
  color: #d73a49;
}

.markdown-content :deep(.hljs-attr) {
  color: #005cc5;
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