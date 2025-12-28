<template>
  <div class="edit-pane-container">
    <!-- 工具栏 -->
    <div class="toolbar">
      <button class="toolbar-button" @click="insertMarkdown('**', '**')" title="Bold">
        <strong>B</strong>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('*', '*')" title="Italic">
        <em>I</em>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('~~', '~~')" title="Strikethrough">
        <s>S</s>
      </button>
      <div class="toolbar-separator"></div>
      <button class="toolbar-button" @click="insertMarkdown('# ', '')" title="Heading 1">
        H1
      </button>
      <button class="toolbar-button" @click="insertMarkdown('## ', '')" title="Heading 2">
        H2
      </button>
      <button class="toolbar-button" @click="insertMarkdown('### ', '')" title="Heading 3">
        H3
      </button>
      <div class="toolbar-separator"></div>
      <button class="toolbar-button" @click="insertMarkdown('[', '](url)')" title="Link">
        🔗
      </button>
      <button class="toolbar-button" @click="insertMarkdown('![', '](url)')" title="Image">
        🖼️
      </button>
      <button class="toolbar-button" @click="insertMarkdown('`', '`')" title="Inline Code">
        &lt;/&gt;
      </button>
      <button class="toolbar-button" @click="insertCodeBlock" title="Code Block">
        { }
      </button>
      <div class="toolbar-separator"></div>
      <button class="toolbar-button" @click="insertMarkdown('> ', '')" title="Quote">
        ❝
      </button>
      <button class="toolbar-button" @click="insertMarkdown('- ', '')" title="Bullet List">
        • 
      </button>
      <button class="toolbar-button" @click="insertMarkdown('1. ', '')" title="Numbered List">
        1.
      </button>
      <button class="toolbar-button" @click="insertTable" title="Table">
        ⊞
      </button>
    </div>

    <!-- 编辑区域 -->
    <div class="editor-container">
      <textarea
        ref="textareaRef"
        v-model="localContent"
        class="editor-textarea"
        placeholder="edit your Note here with Markdown..."
        spellcheck="false"
        @input="handleInput"
        @scroll="handleScroll"
        @keydown="handleKeydown"
      ></textarea>
      
      <!-- 行号 (可选) -->
      <div v-if="showLineNumbers" class="line-numbers">
        <div 
          v-for="n in lineCount" 
          :key="n"
          class="line-number"
        >
          {{ n }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'

// 定义 props
const props = defineProps<{
  content?: string
}>()

// 定义 emits
const emit = defineEmits<{
  (e: 'update:content', content: string): void
}>()

const textareaRef = ref<HTMLTextAreaElement>()
const localContent = ref(props.content || '')
const showLineNumbers = ref(false)
let saveTimeout: NodeJS.Timeout | null = null

// 计算行数
const lineCount = computed(() => {
  return localContent.value.split('\n').length
})

// 监听 props.content 的变化
watch(() => props.content, (newContent) => {
  if (newContent !== undefined) {
    localContent.value = newContent
  }
}, { immediate: true })

// 处理输入
const handleInput = () => {
  // 发出更新事件
  emit('update:content', localContent.value)
  
  // 自动保存逻辑
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }
  
  saveTimeout = setTimeout(() => {
    // 保存逻辑由父组件处理
  }, 2000) // 2秒后自动保存
}

// 处理滚动（用于分屏模式同步滚动，暂时预留）
const handleScroll = () => {
  // 可以在这里实现与预览面板的同步滚动
}

// 处理键盘事件
const handleKeydown = (event: KeyboardEvent) => {
  // Ctrl/Cmd + S 手动保存（由父组件处理）
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault()
    // 保存逻辑由父组件处理
    return
  }

  // Tab 键插入空格而不是切换焦点
  if (event.key === 'Tab') {
    event.preventDefault()
    insertText('  ') // 插入2个空格
    return
  }

  // 自动补全括号等
  const pairs: Record<string, string> = {
    '(': ')',
    '[': ']',
    '{': '}',
    '"': '"',
    "'": "'",
    '`': '`'
  }

  if (pairs[event.key] && textareaRef.value) {
    const textarea = textareaRef.value
    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    
    if (start !== end) {
      // 有选中文本，包围选中内容
      event.preventDefault()
      const selectedText = localContent.value.substring(start, end)
      const newText = event.key + selectedText + pairs[event.key]
      
      const newContent = localContent.value.substring(0, start) + newText + localContent.value.substring(end)
      localContent.value = newContent
      emit('update:content', localContent.value)
      
      nextTick(() => {
        textarea.setSelectionRange(start + 1, start + 1 + selectedText.length)
      })
    }
  }
}

// 插入文本
const insertText = (text: string) => {
  if (!textareaRef.value) return
  
  const textarea = textareaRef.value
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  
  const newContent = localContent.value.substring(0, start) + text + localContent.value.substring(end)
  localContent.value = newContent
  emit('update:content', localContent.value)
  
  nextTick(() => {
    textarea.focus()
    textarea.setSelectionRange(start + text.length, start + text.length)
  })
}

// 插入 Markdown 语法
const insertMarkdown = (before: string, after: string) => {
  if (!textareaRef.value) return
  
  const textarea = textareaRef.value
  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const selectedText = localContent.value.substring(start, end)
  
  let newText: string
  let newCursorPos: number
  
  if (selectedText) {
    // 有选中文本
    newText = before + selectedText + after
    newCursorPos = start + before.length + selectedText.length + after.length
  } else {
    // 无选中文本
    newText = before + after
    newCursorPos = start + before.length
  }
  
  const newContent = localContent.value.substring(0, start) + newText + localContent.value.substring(end)
  localContent.value = newContent
  emit('update:content', localContent.value)
  
  nextTick(() => {
    textarea.focus()
    textarea.setSelectionRange(newCursorPos, newCursorPos)
  })
}

// 插入代码块
const insertCodeBlock = () => {
  const codeBlock = `
\`\`\`

\`\`\`
`
  insertText(codeBlock)
  
  nextTick(() => {
    if (textareaRef.value) {
      const pos = textareaRef.value.selectionStart - 5 // 定位到代码块内部
      textareaRef.value.setSelectionRange(pos, pos)
    }
  })
}

// 插入表格
const insertTable = () => {
  const table = `
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
`
  insertText(table)
}

// 组件卸载时保存
onUnmounted(() => {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }
})
</script>

<style scoped>
.edit-pane-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: #ffffff;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  background-color: #f8f8f8;
  border-bottom: 1px solid #e5e5e5;
  flex-wrap: wrap;
}

.toolbar-button {
  width: 32px;
  height: 32px;
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s;
}

.toolbar-button:hover {
  background-color: #e5e5e5;
  border-color: #d0d0d0;
}

.toolbar-separator {
  width: 1px;
  height: 20px;
  background-color: #e5e5e5;
  margin: 0 4px;
}

.editor-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.editor-textarea {
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  resize: none;
  padding: 16px 24px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.6;
  background-color: #ffffff;
  color: #333;
  tab-size: 2;
  overflow-y: auto;
}

.editor-textarea::placeholder {
  color: #999;
  font-style: italic;
}

.line-numbers {
  position: absolute;
  left: 0;
  top: 16px;
  width: 50px;
  padding-right: 8px;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 14px;
  line-height: 1.6;
  color: #999;
  background-color: #f8f8f8;
  border-right: 1px solid #e5e5e5;
  user-select: none;
  pointer-events: none;
}

.line-number {
  text-align: right;
  padding-right: 8px;
  min-height: 22.4px; /* 匹配行高 */
}

/* 滚动条样式 */
.editor-textarea::-webkit-scrollbar {
  width: 8px;
}

.editor-textarea::-webkit-scrollbar-track {
  background: transparent;
}

.editor-textarea::-webkit-scrollbar-thumb {
  background-color: #e0e0e0;
  border-radius: 4px;
}

.editor-textarea::-webkit-scrollbar-thumb:hover {
  background-color: #c0c0c0;
}

/* 语法高亮效果（基础版，后续可用更完整的高亮库） */
.editor-textarea {
  /* 预留样式空间 */
}
</style>