import { Marked } from 'marked';
import { markedHighlight } from 'marked-highlight';
import hljs from 'highlight.js';
import { convertFileSrc, invoke } from '@tauri-apps/api/tauri';
import logger from './logger';

let dataDirectory: string | null = null;

// 获取数据目录
async function getDataDirectory(): Promise<string> {
  if (!dataDirectory) {
    try {
      dataDirectory = await invoke<string>('get_data_directory');
    } catch (error) {
      console.error('Failed to get data directory:', error);
      return ''; // 回退到空字符串
    }
  }
  return dataDirectory;
}

// 配置 marked，禁用已弃用的选项
const marked = new Marked({
  breaks: true,
  gfm: true
})

// 添加高亮插件
marked.use(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      const language = hljs.getLanguage(lang) ? lang : 'plaintext';
      try {
        const highlighted = hljs.highlight(code, { language }).value;
        // 解码HTML实体，避免显示转义字符
        return highlighted
          .replace(/&quot;/g, '"')
          .replace(/&#x27;/g, "'")
          .replace(/&#x60;/g, '`')
          .replace(/&lt;/g, '<')
          .replace(/&gt;/g, '>')
          .replace(/&amp;/g, '&'); // 注意：&amp; 要最后处理，避免影响其他转换
      } catch (err) {
        logger.warn('Highlight.js error:', err);
        return code;
      }
    }
  })
);

// 自定义渲染器
const renderer = {
  // 自定义链接渲染（在新标签页打开外部链接）
  link(href: string, title: string, text: string) {
    const isExternal = href && (href.startsWith('http') || href.startsWith('https'));
    const target = isExternal ? 'target="_blank" rel="noopener noreferrer"' : '';
    const titleAttr = title ? `title="${title}"` : '';
    return `<a href="${href}" ${target} ${titleAttr}>${text}</a>`;
  },

  // 自定义图片渲染（使用convertFileSrc处理相对路径）
  image(href: string, title: string, text: string) {
    // 对于相对路径，标记为需要处理
    let imageUrl = href;
    if (href && !href.startsWith('http') && !href.startsWith('data:') && !href.startsWith('https:')) {
      // 这是相对路径，添加标记以便后续处理
      imageUrl = `tauri-convert:${href}`;
    }
    
    let out = `<img src="${imageUrl}" alt="${text}"`;
    if (title) {
      out += ` title="${title}"`;
    }
    out += ' style="max-width: 100%; height: auto;">';
    return out;
  },

  // 自定义代码块渲染
  code(code: string, language: string) {
    const validLanguage = language && hljs.getLanguage(language) ? language : 'plaintext';
    const highlighted = hljs.highlight(code, { language: validLanguage }).value;
    
    // 解码HTML实体，避免显示转义字符
    const unescapedHighlighted = highlighted
      .replace(/&quot;/g, '"')
      .replace(/&#x27;/g, "'")
      .replace(/&#x60;/g, '`')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&amp;/g, '&'); // 注意：&amp; 要最后处理，避免影响其他转换
    
    return `<pre class="code-block"><code class="hljs language-${validLanguage}">${unescapedHighlighted}</code></pre>`;
  },

  // 自定义表格渲染
  table(header: string, body: string) {
    return `<div class="table-container"><table class="markdown-table">
      <thead>${header}</thead>
      <tbody>${body}</tbody>
    </table></div>`;
  }
};

marked.use({ renderer });

export async function renderMarkdown(content: string): Promise<string> {
  try {
    console.log('Rendering markdown content:', content);
    
    // 首先解析markdown
    let result = await marked.parse(content);
    
    // 处理所有需要convertFileSrc的图片
    const tauriConvertRegex = /src="tauri-convert:([^"]+)"/g;
    const matches = [...result.matchAll(tauriConvertRegex)];
    
    // 同时处理HTML中直接写的img标签的相对路径
    const htmlImgRegex = /src="([^"]+)"/g;
    const htmlMatches = [...result.matchAll(htmlImgRegex)];
    
    if (matches.length > 0 || htmlMatches.length > 0) {
      // 获取数据目录
      const dataDir = await getDataDirectory();
      
      // 处理Markdown生成的标记图片
      for (const match of matches) {
        const relativePath = match[1];
        try {
          // 构建完整路径并使用convertFileSrc
          const fullPath = `${dataDir}/${relativePath}`;
          const convertedUrl = convertFileSrc(fullPath);
          result = result.replace(match[0], `src="${convertedUrl}"`);
          console.log(`Converted image path: ${relativePath} -> ${convertedUrl}`);
        } catch (error) {
          console.error(`Failed to convert image path: ${relativePath}`, error);
          // 如果转换失败，回退到原始路径
          result = result.replace(match[0], `src="${relativePath}"`);
        }
      }
      
      // 处理HTML img标签中的相对路径
      for (const match of htmlMatches) {
        const srcPath = match[1];
        // 跳过已经处理的tauri-convert路径和绝对URL
        if (srcPath.startsWith('tauri-convert:') || 
            srcPath.startsWith('http') || 
            srcPath.startsWith('https') || 
            srcPath.startsWith('data:') ||
            srcPath.startsWith('asset:') ||
            srcPath.startsWith('tauri://')) {
          continue;
        }
        
        // 这是一个相对路径，需要转换
        try {
          const fullPath = `${dataDir}/${srcPath}`;
          const convertedUrl = convertFileSrc(fullPath);
          result = result.replace(match[0], `src="${convertedUrl}"`);
          console.log(`Converted HTML img path: ${srcPath} -> ${convertedUrl}`);
        } catch (error) {
          console.error(`Failed to convert HTML img path: ${srcPath}`, error);
        }
      }
    }
    
    console.log('Rendered markdown result:', result);
    return result;
  } catch (error) {
    logger.error('Markdown parsing error:', error);
    console.error('Markdown parsing error:', error);
    return `<pre>${content}</pre>`;
  }
}