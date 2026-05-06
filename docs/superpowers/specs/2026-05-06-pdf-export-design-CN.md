# PDF 导出设计

## 背景

XNote 当前已经支持从工具栏导出按钮导出 Markdown。现有流程在 `ActionBar.vue` 中让用户选择导出目录，然后调用 Tauri 的 `export_note` 命令。Rust 命令会写出 Markdown 文件，并把引用到的附件复制到导出目录中。

笔记预览内容由前端渲染。`ViewPane.vue` 调用 `src/utils/markdown.ts` 中的 `renderMarkdown()`，该函数负责 Markdown 解析、代码高亮、表格包装，以及通过 Tauri asset URL 转换本地图片路径。为了让 PDF 输出和预览效果一致，PDF 导出应复用这条前端渲染链路，而不是在 Rust 中重新实现 Markdown 渲染。

## 目标

- 添加 PDF 作为导出格式，同时不移除、不改变现有 Markdown 导出行为。
- 将当前笔记渲染后的 Markdown 内容导出为 PDF。
- 保留标题、段落、列表、代码块、表格、链接和图片的预览样式。
- 包含本地图片、HTML 图片标签、data URL 图片，以及当前应用安全策略允许加载的远程 HTTPS 图片。
- 使用 A4 纵向输出，页边距为 12 mm，并自动分页。
- 将 PDF 保存到用户选择的 `.pdf` 路径。
- 导出前先保存当前笔记，确保 PDF 内容和持久化后的笔记内容一致。

## 非目标

- 第一版不提供 PDF 导出设置。
- 不添加页眉、页脚、页码、目录或元数据编辑器。
- 不实现后端 Markdown 到 PDF 渲染器。
- 不改变现有 Markdown 导出的目录结构。
- 不把新增前端测试运行器作为本功能的前置条件。

## 用户流程

1. 用户点击现有导出按钮。
2. XNote 显示格式选择：`Markdown` 和 `PDF`。
3. 如果用户选择 `Markdown`，XNote 按现有 Markdown 导出流程执行，不做改动。
4. 如果用户选择 `PDF`，XNote 先保存当前笔记。
5. XNote 显示保存文件对话框，过滤为 PDF 文件，默认文件名为 `<note-title>.pdf`。
6. 如果用户选择了路径，XNote 渲染笔记内容并生成 PDF。
7. XNote 将 PDF 写入选定路径，并显示成功消息。

取消格式选择或保存文件对话框时，流程安静退出。

## 架构

### 共享 Markdown 渲染

`src/utils/markdown.ts` 继续作为预览和 PDF 导出的唯一 Markdown 到 HTML 渲染入口。PDF 导出应调用 `renderMarkdown(content)`，使代码高亮、表格 HTML 和图片 URL 转换与预览保持一致。

### 共享预览样式

将当前 `ViewPane.vue` 内 scoped 的 Markdown 内容样式移动到可复用样式表中，例如：

- `src/styles/markdown-content.scss`

`ViewPane.vue` 和 PDF 导出的 DOM 都应使用相同的根类名：

```html
<div class="markdown-content">...</div>
```

这样可以避免预览样式和 PDF 样式逐渐漂移。`ViewPane.vue` 可以继续保留容器布局和滚动条相关的本地样式。

### 导出协调

`ContentPane.vue` 应负责协调导出请求，因为它拥有可编辑标题和草稿内容状态。`ActionBar.vue` 应发出导出请求，而不是在 PDF 路径中直接读取并导出 `currentNote`。

导出协调逻辑应根据当前模式获取最新内容：

- 查看模式：`currentNote.content`。
- 编辑模式：`EditPane` 通过 `update:content` 发出的最新草稿。
- 分屏模式：`splitContent`。

PDF 导出开始前，`ContentPane.vue` 应通过 `notesStore.updateNote()` 保存最新标题和内容。导出应使用 store 更新后返回的已保存笔记，而不是可能过期的 `currentNote` 快照。

### PDF 导出工具

新增一个前端工具，例如 `src/utils/pdfExport.ts`。它负责：

- 清理笔记标题，生成默认 PDF 文件名。
- 打开带 PDF 过滤器的 Tauri `save()` 对话框。
- 通过 `renderMarkdown()` 将 Markdown 渲染为 HTML。
- 创建一个使用共享 `.markdown-content` 类名的离屏导出容器。
- 等待导出容器中的图片加载成功或失败。
- 从 DOM 生成 A4 纵向 PDF。
- 通过 Tauri 文件系统 API 将 PDF 字节写入选定路径。
- 无论导出成功或失败，都移除临时 DOM 节点。

`ActionBar.vue` 只负责展示导出动作并发出请求。`ContentPane.vue` 负责导出前保存，并使用已保存的笔记数据调用该工具。

### PDF 生成

使用前端 DOM 到 PDF 的方案，使生成结果遵循浏览器渲染后的 HTML 和 CSS。实现应使用 A4 纵向设置，页边距为 12 mm。可以使用 `html2pdf.js`，或等价的 `html2canvas` + `jsPDF` 包装方案，因为它们可以渲染与预览相同的 DOM 和 CSS。生成的 PDF 应尽量保留渲染后的图片、表格、代码块样式、颜色和字体排版。

第一版应优先保证视觉一致性，而不是增加高级 PDF 语义。文本可选择性的优先级低于和预览效果保持一致。

## 数据流

1. `ActionBar.vue` 接收导出按钮点击。
2. Element Plus 让用户选择 `Markdown` 或 `PDF`。
3. `ActionBar.vue` 将选中的导出格式发送给 `ContentPane.vue`。
4. 对于 `Markdown`，`ContentPane.vue` 调用现有 Tauri 命令，不改变行为。
5. 对于 `PDF`，`ContentPane.vue` 先保存最新标题和内容。
6. `pdfExport.ts` 请求用户选择目标 PDF 路径。
7. `pdfExport.ts` 调用 `renderMarkdown(savedNote.content)`。
8. 渲染后的 HTML 被挂载到临时 `.markdown-content` 导出容器中。
9. 工具等待图片加载完成或失败。
10. PDF 库将导出容器渲染为 A4 纵向 PDF 输出。
11. 工具将生成的 PDF 字节写入选定路径。
12. `ContentPane.vue` 显示成功或错误反馈。

## 错误处理

- 没有当前笔记：不执行任何操作。
- 用户取消格式选择：不执行任何操作。
- 用户取消保存文件对话框：不执行任何操作。
- 自动保存失败：停止导出，并提示笔记无法保存，因此 PDF 导出已取消。
- Markdown 渲染失败：使用现有 `renderMarkdown()` fallback 输出，并记录失败。
- 部分图片加载失败：继续导出，但报告部分图片可能缺失。
- PDF 生成失败：显示导出错误，不显示成功消息。
- 文件写入失败：显示导出错误，不显示成功消息。

临时导出 DOM 节点必须在 `finally` 路径中移除。

## 布局与分页

- 默认纸张：A4。
- 方向：纵向。
- 页边距：四边均为 12 mm。
- 内容宽度应适配可打印页面。
- 图片应使用 `max-width: 100%` 和 `height: auto`。
- 表格和代码块应尽可能避免溢出页面。宽内容可以根据所选前端 PDF 库的能力进行缩放或换行。
- 长内容应自动分页。

## 测试与验证

自动化验证：

- 运行 `npm run build`，验证前端 TypeScript 和 Vue 构建。
- 运行 `cd src-tauri && cargo test`，确认后端现有行为仍然通过。

手动验证：

- 导出 Markdown，确认现有行为没有改变。
- 导出一篇包含标题、段落、列表、引用、代码块、表格、本地 Markdown 图片、HTML `<img>` 标签、data URL 图片和长内容的笔记为 PDF。
- 对比 PDF 和预览的视觉一致性。
- 确认本地图片出现在 PDF 中。
- 确认表格边框、代码块样式和标题层级与预览一致。
- 确认取消格式选择和保存对话框不会显示错误。
- 编辑笔记但不手动保存，然后导出 PDF，确认 PDF 中包含最新内容。

## 实现说明

- 保持后端 `export_note` 命令专注于 Markdown 导出。
- 优先使用小而聚焦的 `pdfExport.ts` 工具，避免继续扩张 `ActionBar.vue`。
- 将导出前保存逻辑保留在 `ContentPane.vue`，因为那里可以访问最新草稿内容。
- 文件名清理应与 Rust 的 `sanitize_filename()` 对非法文件名字符的处理保持一致。
- 不提交生成的 PDF 或本地笔记数据。
