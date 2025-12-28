import { marked } from 'marked'
import hljs from 'highlight.js'

// 配置 marked 选项
marked.setOptions({
  highlight: function(code, lang) {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value
      } catch (err) {
        console.warn('Highlight.js error:', err)
      }
    }
    return hljs.highlightAuto(code).value
  },
  langPrefix: 'hljs language-',
  breaks: true, // 支持 GFM 换行
  gfm: true, // GitHub Flavored Markdown
})

// 自定义渲染器
const renderer = new marked.Renderer()

// 自定义链接渲染（在新标签页打开外部链接）
renderer.link = function(href, title, text) {
  const isExternal = href && (href.startsWith('http') || href.startsWith('https'))
  const target = isExternal ? 'target="_blank" rel="noopener noreferrer"' : ''
  const titleAttr = title ? `title="${title}"` : ''
  return `<a href="${href}" ${target} ${titleAttr}>${text}</a>`
}

// 自定义图片渲染（支持调整大小）
renderer.image = function(href, title, text) {
  let out = `<img src="${href}" alt="${text}"`
  if (title) {
    out += ` title="${title}"`
  }
  out += ' style="max-width: 100%; height: auto;">'
  return out
}

// 自定义代码块渲染
renderer.code = function(code, language) {
  const validLanguage = language && hljs.getLanguage(language) ? language : 'plaintext'
  const highlighted = hljs.highlight(code, { language: validLanguage }).value
  return `<pre class="code-block"><code class="hljs language-${validLanguage}">${highlighted}</code></pre>`
}

// 自定义表格渲染
renderer.table = function(header, body) {
  return `<div class="table-container"><table class="markdown-table">
    <thead>${header}</thead>
    <tbody>${body}</tbody>
  </table></div>`
}

marked.use({ renderer })

export function renderMarkdown(markdown: string): string {
  try {
    return marked(markdown)
  } catch (error) {
    console.error('Markdown parsing error:', error)
    return `<p style="color: red;">Error parsing markdown: ${error}</p>`
  }
}