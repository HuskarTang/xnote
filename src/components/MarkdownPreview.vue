<template>
  <div class="markdown-preview">
    <div class="preview-content" v-html="renderedContent"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js';

interface Props {
  content: string;
}

const props = defineProps<Props>();

// Configure marked
marked.setOptions({
  highlight: function(code, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value;
      } catch (err) {
        console.error('Highlight.js error:', err);
      }
    }
    return hljs.highlightAuto(code).value;
  },
  breaks: true,
  gfm: true,
});

const renderedContent = computed(() => {
  if (!props.content) return '';
  
  try {
    const html = marked(props.content);
    return DOMPurify.sanitize(html);
  } catch (error) {
    console.error('Markdown rendering error:', error);
    return '<p>Error rendering markdown</p>';
  }
});
</script>

<style lang="scss" scoped>
.markdown-preview {
  height: 100%;
  overflow-y: auto;
  background: white;
}

.preview-content {
  padding: 20px;
  max-width: none;
  
  // Markdown styles
  :deep(h1), :deep(h2), :deep(h3), :deep(h4), :deep(h5), :deep(h6) {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
  }
  
  :deep(h1) {
    font-size: 2em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 10px;
  }
  
  :deep(h2) {
    font-size: 1.5em;
    border-bottom: 1px solid #eaecef;
    padding-bottom: 8px;
  }
  
  :deep(h3) { font-size: 1.25em; }
  :deep(h4) { font-size: 1em; }
  :deep(h5) { font-size: 0.875em; }
  :deep(h6) { font-size: 0.85em; color: #6a737d; }
  
  :deep(p) {
    margin-bottom: 16px;
    line-height: 1.6;
  }
  
  :deep(ul), :deep(ol) {
    margin-bottom: 16px;
    padding-left: 2em;
  }
  
  :deep(li) {
    margin-bottom: 4px;
  }
  
  :deep(blockquote) {
    margin: 0 0 16px 0;
    padding: 0 1em;
    color: #6a737d;
    border-left: 4px solid #dfe2e5;
  }
  
  :deep(code) {
    background-color: #f6f8fa;
    border-radius: 3px;
    font-size: 85%;
    margin: 0;
    padding: 0.2em 0.4em;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  }
  
  :deep(pre) {
    background-color: #f6f8fa;
    border-radius: 6px;
    font-size: 85%;
    line-height: 1.45;
    overflow: auto;
    padding: 16px;
    margin-bottom: 16px;
    
    code {
      background: transparent;
      border-radius: 0;
      font-size: 100%;
      margin: 0;
      padding: 0;
      word-break: normal;
      word-wrap: normal;
    }
  }
  
  :deep(table) {
    border-collapse: collapse;
    border-spacing: 0;
    width: 100%;
    margin-bottom: 16px;
  }
  
  :deep(th), :deep(td) {
    border: 1px solid #dfe2e5;
    padding: 6px 13px;
    text-align: left;
  }
  
  :deep(th) {
    background-color: #f6f8fa;
    font-weight: 600;
  }
  
  :deep(tr:nth-child(2n)) {
    background-color: #f6f8fa;
  }
  
  :deep(img) {
    max-width: 100%;
    height: auto;
    border-radius: 6px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
  
  :deep(a) {
    color: #0366d6;
    text-decoration: none;
    
    &:hover {
      text-decoration: underline;
    }
  }
  
  :deep(hr) {
    border: none;
    border-top: 1px solid #eaecef;
    margin: 24px 0;
  }
  
  // Task lists
  :deep(input[type="checkbox"]) {
    margin-right: 8px;
  }
  
  // Highlight.js styles
  :deep(.hljs) {
    background: #f6f8fa !important;
    color: #24292e;
  }
}
</style>