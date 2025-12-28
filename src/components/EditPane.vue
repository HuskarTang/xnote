<template>
  <div class="edit-pane-container">
    <!-- 工具栏 -->
    <div class="toolbar">
      <!-- 文本格式 -->
      <button class="toolbar-button" @click="insertMarkdown('**', '**')" title="Bold">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4 2h4.5c1.38 0 2.5 1.12 2.5 2.5 0 .74-.4 1.39-1 1.73.6.34 1 .99 1 1.73 0 1.38-1.12 2.5-2.5 2.5H4V2zm2 3.5h2.5c.28 0 .5-.22.5-.5s-.22-.5-.5-.5H6v1zm0 3h2.5c.28 0 .5-.22.5-.5s-.22-.5-.5-.5H6v1z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('*', '*')" title="Italic">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M6 2h6v2H9.5l-2 8H10v2H4v-2h2.5l2-8H6V2z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('~~', '~~')" title="Strikethrough">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 1c1.93 0 3.5.86 3.5 2.5 0 .83-.67 1.5-1.5 1.5s-1.5-.67-1.5-1.5c0-.28-.22-.5-.5-.5s-.5.22-.5.5v1h6v2H2V6h6V5c0-1.64 1.57-3 3.5-3zM2 8h12v1H2V8zm6 3c-.28 0-.5.22-.5.5s.22.5.5.5.5-.22.5-.5-.22-.5-.5-.5z"/>
        </svg>
      </button>
      
      <div class="toolbar-separator"></div>
      
      <!-- 标题 -->
      <button class="toolbar-button" @click="insertMarkdown('# ', '')" title="Heading 1">
        <span class="heading-text">H1</span>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('## ', '')" title="Heading 2">
        <span class="heading-text">H2</span>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('### ', '')" title="Heading 3">
        <span class="heading-text">H3</span>
      </button>
      
      <div class="toolbar-separator"></div>
      
      <!-- 链接和媒体 -->
      <button class="toolbar-button" @click="insertMarkdown('[', '](url)')" title="Link">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M7.775 3.275a.75.75 0 001.06 1.06l1.25-1.25a2 2 0 112.83 2.83l-2.5 2.5a2 2 0 01-2.83 0 .75.75 0 00-1.06 1.06 3.5 3.5 0 004.95 0l2.5-2.5a3.5 3.5 0 00-4.95-4.95l-1.25 1.25zm-4.69 9.64a2 2 0 010-2.83l2.5-2.5a2 2 0 012.83 0 .75.75 0 001.06-1.06 3.5 3.5 0 00-4.95 0l-2.5 2.5a3.5 3.5 0 004.95 4.95l1.25-1.25a.75.75 0 00-1.06-1.06l-1.25 1.25a2 2 0 01-2.83 0z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="showImageDialog" title="Image">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1.75 2.5a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25v-2.5L10.5 8.5l-2.5 2.5L6.5 9.5 1.5 12V2.75a.25.25 0 01.25-.25h12.5a.25.25 0 01.25.25v5.25h1.5V2.75A1.75 1.75 0 0014.25 1H1.75A1.75 1.75 0 000 2.75v10.5C0 14.216.784 15 1.75 15h12.5A1.75 1.75 0 0016 13.25V8.5l-4-4-2.5 2.5L8 8.5 1.75 2.5z"/>
          <circle cx="4.5" cy="5.5" r="1.5"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="showAttachmentManager" title="Attachments">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M10.5 5a2.5 2.5 0 00-5 0v6a4 4 0 008 0V6a.5.5 0 00-1 0v5a3 3 0 01-6 0V5a1.5 1.5 0 013 0v5.5a.5.5 0 01-1 0V5a.5.5 0 00-1 0v5.5a1.5 1.5 0 003 0V5z"/>
        </svg>
      </button>
      
      <div class="toolbar-separator"></div>
      
      <!-- 代码 -->
      <button class="toolbar-button" @click="insertMarkdown('`', '`')" title="Inline Code">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M4.72 3.22a.75.75 0 011.06 1.06L2.06 8l3.72 3.72a.75.75 0 11-1.06 1.06L.47 8.53a.75.75 0 010-1.06l4.25-4.25zm6.56 0a.75.75 0 10-1.06 1.06L13.94 8l-3.72 3.72a.75.75 0 101.06 1.06l4.25-4.25a.75.75 0 000-1.06l-4.25-4.25z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertCodeBlock" title="Code Block">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25V2.75zm1.75-.25a.25.25 0 00-.25.25v10.5c0 .138.112.25.25.25h12.5a.25.25 0 00.25-.25V2.75a.25.25 0 00-.25-.25H1.75zM7.25 8L5.5 6.25 6.56 5.19 9.37 8l-2.81 2.81L5.5 9.75 7.25 8z"/>
        </svg>
      </button>
      
      <div class="toolbar-separator"></div>
      
      <!-- 列表和引用 -->
      <button class="toolbar-button" @click="insertMarkdown('> ', '')" title="Quote">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1.75 2.5a.25.25 0 00-.25.25v8.5c0 .138.112.25.25.25h2.5a.75.75 0 000-1.5h-1.75V4h1.75a.75.75 0 000-1.5H1.75zm6 0a.25.25 0 00-.25.25v8.5c0 .138.112.25.25.25h2.5a.75.75 0 000-1.5H8.5V4h1.75a.75.75 0 000-1.5H7.75z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('- ', '')" title="Bullet List">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M2 4a1 1 0 100-2 1 1 0 000 2zm3.75-1.5a.75.75 0 000 1.5h8.5a.75.75 0 000-1.5h-8.5zm0 5a.75.75 0 000 1.5h8.5a.75.75 0 000-1.5h-8.5zm0 5a.75.75 0 000 1.5h8.5a.75.75 0 000-1.5h-8.5zM2 9a1 1 0 100-2 1 1 0 000 2zm0 5a1 1 0 100-2 1 1 0 000 2z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertMarkdown('1. ', '')" title="Numbered List">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M2.003 2.5a.5.5 0 00-.723-.447l-1.003.5a.5.5 0 00.446.895l.28-.14V6H.5a.5.5 0 000 1h2.006a.5.5 0 100-1h-.503V2.5zM5 3.25a.75.75 0 01.75-.75h8.5a.75.75 0 010 1.5h-8.5A.75.75 0 015 3.25zm0 5a.75.75 0 01.75-.75h8.5a.75.75 0 010 1.5h-8.5A.75.75 0 015 8.25zm0 5a.75.75 0 01.75-.75h8.5a.75.75 0 010 1.5h-8.5a.75.75 0 01-.75-.75zM.924 10.32l.003-.004a.851.851 0 01.144-.153A.66.66 0 011.5 10c.195 0 .306.068.374.146a.57.57 0 01.128.376c0 .453-.269.682-.8 1.078l-.035.025C.692 11.98 0 12.495 0 13.5a.5.5 0 00.5.5h2.003a.5.5 0 000-1H1.146c.132-.197.351-.372.654-.597l.047-.035c.47-.35 1.156-.858 1.156-1.845 0-.365-.118-.744-.377-1.038-.268-.303-.658-.484-1.126-.484-.48 0-.84.202-1.068.392a1.858 1.858 0 00-.348.384l-.007.011-.002.004-.001.002-.001.001a.5.5 0 00.851.525zM.5 10.055l-.427-.26.427.26z"/>
        </svg>
      </button>
      <button class="toolbar-button" @click="insertTable" title="Table">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
          <path d="M0 2.75C0 1.784.784 1 1.75 1h12.5c.966 0 1.75.784 1.75 1.75v10.5A1.75 1.75 0 0114.25 15H1.75A1.75 1.75 0 010 13.25V2.75zm1.75-.25a.25.25 0 00-.25.25V5h4V2.5H1.75zm4.75 0V5h4V2.5H6.5zm5.5 0V5h2.25a.25.25 0 00.25-.25V2.75a.25.25 0 00-.25-.25H12zm2.25 3.75H12V9h2.25a.25.25 0 00.25-.25V6.5a.25.25 0 00-.25-.25zM12 10.5h2.25a.25.25 0 00.25-.25v-2.75a.25.25 0 00-.25-.25H12v3.25zm-1.5 0V6.5H6.5V9.5h4zm-5.5 0V6.5H1.5v2.75c0 .138.112.25.25.25H5zm-3.75 1.5H5V15H1.75a.25.25 0 01-.25-.25v-2.75zm5.5 0H10.5V15H6.5v-3zm5.5 0H14.25a.25.25 0 01.25.25V13.25a.25.25 0 01-.25.25H12v-1.25z"/>
        </svg>
      </button>
    </div>

    <!-- 编辑区域 -->
    <div class="editor-container">
      <div ref="editorRef" class="codemirror-editor"></div>
    </div>

    <!-- 图片管理器对话框 -->
    <ImageManager
      :visible="imageManagerVisible"
      @close="imageManagerVisible = false"
      @insert="insertImage"
    />
    
    <!-- 附件管理器对话框 -->
    <AttachmentManager
      :visible="attachmentManagerVisible"
      :note-id="noteId"
      @close="attachmentManagerVisible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { EditorView, keymap, lineNumbers, drawSelection, Decoration, ViewPlugin, DecorationSet } from '@codemirror/view'
import { EditorState, Range } from '@codemirror/state'
import { markdown } from '@codemirror/lang-markdown'
import { indentOnInput, syntaxHighlighting, defaultHighlightStyle, bracketMatching, HighlightStyle } from '@codemirror/language'
import { search, searchKeymap } from '@codemirror/search'
import { defaultKeymap, indentWithTab } from '@codemirror/commands'
import { tags } from '@lezer/highlight'
import { invoke } from '@tauri-apps/api/tauri'
import ImageManager from './ImageManager.vue'
import AttachmentManager from './AttachmentManager.vue'

// 定义 props
const props = defineProps<{
  content?: string
  noteId?: string
}>()

// 定义 emits
const emit = defineEmits<{
  (e: 'update:content', content: string): void
  (e: 'save', content: string): void
}>()

const editorRef = ref<HTMLElement>()
let editorView: EditorView | null = null
let saveTimeout: NodeJS.Timeout | null = null

// 图片管理器可见性
const imageManagerVisible = ref(false)

// 附件管理器可见性
const attachmentManagerVisible = ref(false)

// 创建删除线标记装饰
const strikethroughMarkDecoration = Decoration.mark({
  class: 'cm-strikethrough-mark',
  attributes: { style: 'color: #ef4444; font-weight: bold;' }
})

// 删除线标记高亮插件
const strikethroughMarkPlugin = ViewPlugin.fromClass(class {
  decorations: DecorationSet

  constructor(view: EditorView) {
    this.decorations = this.buildDecorations(view)
  }

  update(update: any) {
    if (update.docChanged || update.viewportChanged) {
      this.decorations = this.buildDecorations(update.view)
    }
  }

  buildDecorations(view: EditorView): DecorationSet {
    const decorations: Range<Decoration>[] = []
    const doc = view.state.doc
    
    for (let i = 1; i <= doc.lines; i++) {
      const line = doc.line(i)
      const text = line.text
      
      // 匹配 ~~content~~ 模式
      const strikethroughRegex = /~~(.+?)~~/g
      let match
      
      while ((match = strikethroughRegex.exec(text)) !== null) {
        const start = line.from + match.index
        const startMarkEnd = start + 2 // ~~
        const endMarkStart = line.from + match.index + match[0].length - 2
        const end = line.from + match.index + match[0].length
        
        // 高亮开始的 ~~
        decorations.push(strikethroughMarkDecoration.range(start, startMarkEnd))
        // 高亮结束的 ~~
        decorations.push(strikethroughMarkDecoration.range(endMarkStart, end))
      }
    }
    
    return Decoration.set(decorations)
  }
}, {
  decorations: v => v.decorations
})

// 自定义 Markdown 高亮样式 - 语法关键字红色高亮
const markdownHighlighting = HighlightStyle.define([
  // 标题标记和内容
  { tag: tags.heading1, fontWeight: 'bold', fontSize: '1.4em' },
  { tag: tags.heading2, fontWeight: 'bold', fontSize: '1.25em' },
  { tag: tags.heading3, fontWeight: 'bold', fontSize: '1.1em' },
  { tag: tags.heading4, fontWeight: 'bold' },
  { tag: tags.heading5, fontWeight: 'bold' },
  { tag: tags.heading6, fontWeight: 'bold' },
  
  // 所有 Markdown 语法标记统一为红色
  { tag: tags.processingInstruction, color: '#ef4444' }, // # 标题标记
  { tag: tags.punctuation, color: '#ef4444' }, // **, *, ~~, `, [, ], (, ) 等
  { tag: tags.bracket, color: '#ef4444' }, // [] 括号
  { tag: tags.paren, color: '#ef4444' }, // () 括号  
  { tag: tags.operator, color: '#ef4444' }, // > 引用标记
  { tag: tags.keyword, color: '#ef4444' }, // 列表标记 -, *, +, 1.
  { tag: tags.meta, color: '#ef4444' }, // 其他语法标记
  { tag: tags.atom, color: '#ef4444' }, // 特殊符号
  { tag: tags.special, color: '#ef4444' }, // 特殊标记
  { tag: tags.modifier, color: '#ef4444' }, // 修饰符标记
  
  // 删除线特殊处理 - 标记和内容都处理
  { tag: tags.strikethrough, textDecoration: 'line-through' },
  
  // 内容样式保持简洁
  { tag: tags.strong, fontWeight: 'bold' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.monospace, backgroundColor: '#f1f5f9', padding: '1px 3px', borderRadius: '2px', fontFamily: 'Monaco, Menlo, monospace', fontSize: '0.9em' },
  { tag: tags.link, textDecoration: 'underline' },
  { tag: tags.quote, fontStyle: 'italic' },
  
  // 默认内容颜色
  { tag: tags.content, color: '#374151' }
])

// 创建编辑器
const createEditor = () => {
  if (!editorRef.value) return

  const updateListener = EditorView.updateListener.of((update: any) => {
    if (update.docChanged) {
      const content = update.state.doc.toString()
      emit('update:content', content)
      
      // 自动保存逻辑
      if (saveTimeout) {
        clearTimeout(saveTimeout)
      }
      
      saveTimeout = setTimeout(() => {
        console.log('Auto saving content:', content)
        emit('save', content)
      }, 2000) // 2秒后自动保存
    }
  })

  // 添加粘贴事件监听器
  const pasteListener = EditorView.domEventHandlers({
    paste: (event: ClipboardEvent) => {
      const items = event.clipboardData?.items
      if (!items) return
      
      const files: File[] = []
      for (let i = 0; i < items.length; i++) {
        const item = items[i]
        if (item.type.startsWith('image/')) {
          const file = item.getAsFile()
          if (file) {
            files.push(file)
          }
        }
      }
      
      if (files.length > 0) {
        event.preventDefault()
        const fileList = files as unknown as FileList
        handleImagePaste(fileList)
      }
    }
  })

  const state = EditorState.create({
    doc: props.content || '',
    extensions: [
      // 基础扩展
      lineNumbers(),
      drawSelection(),
      indentOnInput(),
      bracketMatching(),
      search(),
      
      // 语法高亮 - 使用markdown语言支持和自定义样式
      markdown(),
      syntaxHighlighting(markdownHighlighting, { fallback: true }),
      
      // 删除线标记高亮插件
      strikethroughMarkPlugin,
      
      // 键盘映射
      keymap.of([
        ...defaultKeymap,
        ...searchKeymap,
        indentWithTab,
      ]),
      
      // 更新监听器
      updateListener,
      
      // 粘贴事件监听器
      pasteListener,
      
      // 主题和样式
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: '14px',
          fontFamily: "'Monaco', 'Menlo', 'Ubuntu Mono', monospace",
        },
        '.cm-content': {
          padding: '16px 24px',
          minHeight: '100%',
          lineHeight: '1.6',
        },
        '.cm-focused': {
          outline: 'none',
        },
        '.cm-editor': {
          height: '100%',
        },
        '.cm-scroller': {
          height: '100%',
          overflow: 'auto',
        },
        '.cm-line': {
          lineHeight: '1.6',
        },
        // 行号样式
        '.cm-lineNumbers': {
          color: '#9ca3af',
          backgroundColor: '#f8fafc',
          borderRight: '1px solid #e2e8f0',
          padding: '0 8px',
        },
        '.cm-activeLineGutter': {
          backgroundColor: '#f1f5f9',
        },
        '.cm-activeLine': {
          backgroundColor: 'transparent',
        },
        // 选择样式
        '.cm-selectionBackground': {
          backgroundColor: '#dbeafe !important',
        },
        '&.cm-focused .cm-selectionBackground': {
          backgroundColor: '#bfdbfe !important',
        },
      }),
      
      // 编辑器设置
      EditorView.lineWrapping,
    ]
  })

  editorView = new EditorView({
    state,
    parent: editorRef.value,
  })
}

// 更新编辑器内容
const updateContent = (newContent: string) => {
  if (!editorView || editorView.state.doc.toString() === newContent) return
  
  editorView.dispatch({
    changes: {
      from: 0,
      to: editorView.state.doc.length,
      insert: newContent,
    }
  })
}

// 插入文本到编辑器
const insertText = (text: string) => {
  if (!editorView) return
  
  const { from, to } = editorView.state.selection.main
  editorView.dispatch({
    changes: { from, to, insert: text },
    selection: { anchor: from + text.length }
  })
  editorView.focus()
}

// 插入 Markdown 语法
const insertMarkdown = (before: string, after: string) => {
  if (!editorView) return
  
  const { from, to } = editorView.state.selection.main
  const selectedText = editorView.state.doc.sliceString(from, to)
  
  let newText: string
  let newCursorPos: number
  
  if (selectedText) {
    // 有选中文本
    newText = before + selectedText + after
    newCursorPos = from + before.length + selectedText.length + after.length
  } else {
    // 无选中文本
    newText = before + after
    newCursorPos = from + before.length
  }
  
  editorView.dispatch({
    changes: { from, to, insert: newText },
    selection: { anchor: newCursorPos }
  })
  editorView.focus()
}

// 插入代码块
const insertCodeBlock = () => {
  const codeBlock = '\n```\n\n```\n'
  if (!editorView) return
  
  const { from, to } = editorView.state.selection.main
  editorView.dispatch({
    changes: { from, to, insert: codeBlock },
    selection: { anchor: from + 5 } // 定位到代码块内部
  })
  editorView.focus()
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

// 显示图片对话框
const showImageDialog = () => {
  imageManagerVisible.value = true
}

// 显示附件管理器
const showAttachmentManager = () => {
  attachmentManagerVisible.value = true
}

// 插入图片
const insertImage = (imageMarkdown: string) => {
  insertText(imageMarkdown)
}

// 处理图片粘贴
const handleImagePaste = async (files: FileList) => {
  for (const file of files) {
    if (file.type.startsWith('image/')) {
      try {
        // 生成文件名
        const now = new Date()
        const timestamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`
        const extension = file.name.split('.').pop() || 'png'
        const filename = `Clipboard_${timestamp}.${extension}`
        
        // 转换为字节数组
        const arrayBuffer = await file.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)
        
        // 保存图片
        const savedPath = await invoke<string>('save_image_to_attachments', {
          filename,
          data: Array.from(uint8Array)
        })
        
        // 插入markdown
        const imageMarkdown = `![clipboard image](${savedPath})`
        insertText(imageMarkdown)
        
        console.log('Image pasted and saved:', savedPath)
      } catch (error) {
        console.error('Failed to paste image:', error)
        alert('Failed to paste image')
      }
      break // 只处理第一个图片文件
    }
  }
}

// 监听 props.content 的变化
watch(() => props.content, (newContent) => {
  if (newContent !== undefined) {
    updateContent(newContent)
  }
}, { immediate: false })

// 组件挂载
onMounted(() => {
  nextTick(() => {
    createEditor()
  })
})

// 组件卸载
onUnmounted(() => {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }
  if (editorView) {
    editorView.destroy()
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
  gap: 2px;
  padding: 8px 16px;
  background-color: #ffffff;
  border-bottom: 1px solid #e5e5e5;
  flex-wrap: wrap;
  min-height: 48px;
}

.toolbar-button {
  width: 32px;
  height: 32px;
  background-color: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.15s ease;
  color: #6b7280;
  position: relative;
}

.toolbar-button:hover {
  background-color: #f3f4f6;
  border-color: #e5e7eb;
  color: #374151;
}

.toolbar-button:active {
  background-color: #e5e7eb;
  transform: translateY(1px);
}

.toolbar-button svg {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.heading-text {
  font-size: 11px;
  font-weight: 600;
  color: currentColor;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.toolbar-separator {
  width: 1px;
  height: 24px;
  background-color: #e5e7eb;
  margin: 0 6px;
  flex-shrink: 0;
}

.editor-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.codemirror-editor {
  height: 100%;
  width: 100%;
}

/* CodeMirror 全局样式覆盖 */
:deep(.cm-editor) {
  height: 100% !important;
}

:deep(.cm-scroller) {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace !important;
}

:deep(.cm-content) {
  caret-color: #2563eb;
}

/* 优化滚动条样式 */
:deep(.cm-scroller::-webkit-scrollbar) {
  width: 8px;
}

:deep(.cm-scroller::-webkit-scrollbar-track) {
  background: transparent;
}

:deep(.cm-scroller::-webkit-scrollbar-thumb) {
  background-color: #e0e0e0;
  border-radius: 4px;
}

:deep(.cm-scroller::-webkit-scrollbar-thumb:hover) {
  background-color: #c0c0c0;
}

/* Markdown 特殊样式增强 */
:deep(.cm-header1) {
  font-size: 1.5em;
  font-weight: bold;
  color: #1e40af;
  border-bottom: 2px solid #e5e7eb;
  padding-bottom: 2px;
}

:deep(.cm-header2) {
  font-size: 1.3em;
  font-weight: bold;
  color: #1e40af;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 1px;
}

:deep(.cm-header3) {
  font-size: 1.1em;
  font-weight: bold;
  color: #1e40af;
}

:deep(.cm-monospace) {
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  background-color: #f1f5f9;
  padding: 1px 3px;
  border-radius: 3px;
  font-size: 0.9em;
}

/* 表格样式 */
:deep(.cm-table) {
  border-collapse: collapse;
  margin: 8px 0;
}

/* 引用块样式增强 */
:deep(.cm-quote) {
  border-left: 4px solid #cbd5e1;
  padding-left: 12px;
  margin-left: 4px;
  background-color: #f8fafc;
  color: #64748b;
  font-style: italic;
}

/* 链接样式增强 */
:deep(.cm-link) {
  color: #2563eb;
  text-decoration: none;
}

:deep(.cm-link:hover) {
  text-decoration: underline;
}
</style>