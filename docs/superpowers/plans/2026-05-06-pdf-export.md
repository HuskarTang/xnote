# PDF Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a PDF export option that saves the latest rendered Markdown note as an A4 portrait PDF while preserving preview styling, images, tables, and code blocks.

**Architecture:** Keep Markdown export behavior intact and route export format selection through `ActionBar.vue` to `ContentPane.vue`. Move Markdown preview styles into a shared global SCSS file, then use the same `.markdown-content` class for both preview and the temporary PDF export DOM. Generate PDFs in the frontend with `html2pdf.js`, write the bytes through Tauri fs APIs, and save the latest note before PDF export.

**Tech Stack:** Vue 3, TypeScript, Pinia, Element Plus, Tauri v1 dialog/fs APIs, `marked`, `highlight.js`, SCSS, `html2pdf.js`.

---

## File Structure

- Modify `package.json` and `package-lock.json`: add `html2pdf.js` and the `sass` preprocessor required for imported SCSS.
- Create `src/types/html2pdf.d.ts`: provide the minimal TypeScript declaration needed by this app.
- Create `src/styles/markdown-content.scss`: shared rendered Markdown styles and PDF export layout rules.
- Modify `src/main.ts`: import shared Markdown content styles.
- Modify `src/components/ViewPane.vue`: remove duplicated Markdown styles and rely on the shared stylesheet.
- Create `src/utils/pdfExport.ts`: filename sanitization, save dialog, image wait logic, DOM setup, PDF generation, and binary write.
- Modify `src/components/ActionBar.vue`: emit export format requests and keep format selection in the toolbar action.
- Modify `src/components/ContentPane.vue`: own save-before-export, route Markdown export through the existing Tauri command, and call `exportNoteAsPdf()` with saved note data.

## Task 1: Install PDF Dependency And Types

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Create: `src/types/html2pdf.d.ts`

- [ ] **Step 1: Add the frontend PDF dependency**

Run:

```bash
npm install html2pdf.js
npm install -D sass
```

Expected: `package.json` contains `html2pdf.js` under dependencies, `sass` under devDependencies, and `package-lock.json` is updated.

- [ ] **Step 2: Add a focused type declaration**

Create `src/types/html2pdf.d.ts` with:

```ts
declare module 'html2pdf.js' {
  interface Html2PdfWorker {
    from(element: HTMLElement): Html2PdfWorker
    set(options: Record<string, unknown>): Html2PdfWorker
    outputPdf(type: 'arraybuffer'): Promise<ArrayBuffer>
  }

  function html2pdf(): Html2PdfWorker

  export default html2pdf
}
```

- [ ] **Step 3: Verify TypeScript sees the declaration**

Run:

```bash
npm run build
```

Expected: the build may still fail because implementation files are not present yet, but it must not fail because `src/types/html2pdf.d.ts` is malformed.

## Task 2: Share Rendered Markdown Styles

**Files:**
- Create: `src/styles/markdown-content.scss`
- Modify: `src/main.ts`
- Modify: `src/components/ViewPane.vue`

- [ ] **Step 1: Create the shared stylesheet**

Create `src/styles/markdown-content.scss` with the rendered Markdown rules currently in `ViewPane.vue`, converted from scoped `:deep()` selectors to global `.markdown-content` selectors:

```scss
.markdown-content {
  padding: 16px 24px;
  max-width: none;
  line-height: 1.6;
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;

  h1 {
    font-size: 2em;
    font-weight: 600;
    margin: 24px 0 16px 0;
    padding-bottom: 8px;
    border-bottom: 2px solid #e5e5e5;
    color: #333;
  }

  h2 {
    font-size: 1.5em;
    font-weight: 600;
    margin: 20px 0 12px 0;
    padding-bottom: 4px;
    border-bottom: 1px solid #e5e5e5;
    color: #333;
  }

  h3 {
    font-size: 1.25em;
    font-weight: 600;
    margin: 16px 0 8px 0;
    color: #333;
  }

  h4 {
    font-size: 1.1em;
    font-weight: 600;
    margin: 16px 0 8px 0;
    color: #333;
  }

  h5 {
    font-size: 1em;
    font-weight: 600;
    margin: 16px 0 8px 0;
    color: #333;
  }

  h6 {
    font-size: 0.9em;
    font-weight: 600;
    margin: 16px 0 8px 0;
    color: #666;
  }

  p {
    margin: 12px 0;
    line-height: 1.7;
  }

  blockquote {
    margin: 16px 0;
    padding: 8px 16px;
    background-color: #f8f9fa;
    border-left: 4px solid #007acc;
    color: #666;
    font-style: italic;
  }

  blockquote p {
    margin: 8px 0;
  }

  ul,
  ol {
    margin: 12px 0;
    padding-left: 24px;
  }

  li {
    margin: 4px 0;
    line-height: 1.6;
  }

  li p {
    margin: 4px 0;
  }

  code {
    background-color: #f1f3f4;
    padding: 2px 4px;
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9em;
    color: #e74c3c;
  }

  .code-block {
    background-color: #f8f8f8;
    border: 1px solid #e5e5e5;
    border-radius: 6px;
    padding: 16px;
    margin: 16px 0;
    overflow-x: auto;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 14px;
    line-height: 1.4;
    page-break-inside: avoid;
    break-inside: avoid;
  }

  .code-block code {
    background: none;
    padding: 0;
    color: inherit;
    border-radius: 0;
  }

  .table-container {
    margin: 16px 0;
    overflow-x: auto;
    page-break-inside: avoid;
    break-inside: avoid;
  }

  .markdown-table {
    width: 100%;
    border-collapse: collapse;
    border: 1px solid #e5e5e5;
  }

  .markdown-table th,
  .markdown-table td {
    border: 1px solid #e5e5e5;
    padding: 8px 12px;
    text-align: left;
  }

  .markdown-table th {
    background-color: #f8f9fa;
    font-weight: 600;
  }

  .markdown-table tr:nth-child(even) {
    background-color: #f9f9f9;
  }

  img {
    max-width: 100%;
    height: auto;
    margin: 8px 0;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    page-break-inside: avoid;
    break-inside: avoid;
  }

  a {
    color: #007acc;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  hr {
    margin: 24px 0;
    border: none;
    border-top: 1px solid #e5e5e5;
  }

  del {
    color: #999;
  }

  strong {
    font-weight: 600;
    color: #333;
  }

  em {
    font-style: italic;
    color: #666;
  }

  .hljs {
    background: #f8f8f8;
    color: #333;
  }

  .hljs-keyword {
    color: #007acc;
    font-weight: bold;
  }

  .hljs-string {
    color: #22863a;
  }

  .hljs-comment {
    color: #6f42c1;
    font-style: italic;
  }

  .hljs-number,
  .hljs-attr {
    color: #005cc5;
  }

  .hljs-function {
    color: #6f42c1;
  }

  .hljs-tag {
    color: #d73a49;
  }
}

.markdown-content.pdf-export-content {
  width: 186mm;
  min-height: 273mm;
  padding: 0;
  background: #ffffff;
  color: #333;
}
```

- [ ] **Step 2: Import shared Markdown styles in the app entry**

Modify `src/main.ts` to include:

```ts
import './styles/markdown-content.scss'
```

The imports should appear with the other global style imports.

- [ ] **Step 3: Remove duplicated Markdown styling from `ViewPane.vue`**

Keep `.view-pane-container`, `.empty-preview`, and scrollbar styles in `ViewPane.vue`. Remove the `.markdown-content` block and all `.markdown-content :deep(...)` rules from the scoped style because those are now in `src/styles/markdown-content.scss`.

- [ ] **Step 4: Verify preview style compilation**

Run:

```bash
npm run build
```

Expected: build succeeds or fails only on later unfinished tasks, not on SCSS syntax.

## Task 3: Implement PDF Export Utility

**Files:**
- Create: `src/utils/pdfExport.ts`

- [ ] **Step 1: Create the utility with explicit exports**

Create `src/utils/pdfExport.ts` with:

```ts
import { save } from '@tauri-apps/api/dialog'
import { writeBinaryFile } from '@tauri-apps/api/fs'
import html2pdf from 'html2pdf.js'
import { renderMarkdown } from '@/utils/markdown'
import type { Note } from '@/types'

export interface PdfExportResult {
  filePath: string
  missingImages: number
}

interface ImageLoadResult {
  loaded: number
  failed: number
}

export const sanitizePdfFileName = (title: string): string => {
  const invalidChars = /[<>:"/\\|?*\x00-\x1f]/g
  const sanitized = title.replace(invalidChars, '_').trim()
  const baseName = sanitized || 'Untitled'
  return baseName.length > 100 ? baseName.slice(0, 100).trim() : baseName
}

export const ensurePdfExtension = (filePath: string): string => {
  return filePath.toLowerCase().endsWith('.pdf') ? filePath : `${filePath}.pdf`
}

const waitForImages = async (container: HTMLElement): Promise<ImageLoadResult> => {
  const images = Array.from(container.querySelectorAll('img'))
  let loaded = 0
  let failed = 0

  await Promise.all(images.map((image) => {
    if (image.complete) {
      if (image.naturalWidth > 0) {
        loaded += 1
      } else {
        failed += 1
      }
      return Promise.resolve()
    }

    return new Promise<void>((resolve) => {
      image.onload = () => {
        loaded += 1
        resolve()
      }
      image.onerror = () => {
        failed += 1
        resolve()
      }
    })
  }))

  return { loaded, failed }
}

const createExportContainer = (html: string): HTMLElement => {
  const wrapper = document.createElement('div')
  wrapper.style.position = 'fixed'
  wrapper.style.left = '-10000px'
  wrapper.style.top = '0'
  wrapper.style.width = '210mm'
  wrapper.style.background = '#ffffff'
  wrapper.style.zIndex = '-1'

  const content = document.createElement('div')
  content.className = 'markdown-content pdf-export-content'
  content.innerHTML = html
  wrapper.appendChild(content)
  document.body.appendChild(wrapper)

  return wrapper
}

const createPdfBytes = async (contentElement: HTMLElement): Promise<Uint8Array> => {
  const options = {
    margin: 12,
    filename: 'xnote-export.pdf',
    image: { type: 'jpeg', quality: 0.98 },
    html2canvas: {
      scale: 2,
      useCORS: true,
      allowTaint: true,
      backgroundColor: '#ffffff'
    },
    jsPDF: {
      unit: 'mm',
      format: 'a4',
      orientation: 'portrait'
    },
    pagebreak: {
      mode: ['css', 'legacy'],
      avoid: ['img', 'table', 'pre', '.code-block', '.table-container']
    }
  }

  const pdfArrayBuffer = await html2pdf()
    .from(contentElement)
    .set(options)
    .outputPdf('arraybuffer')

  return new Uint8Array(pdfArrayBuffer)
}

export const exportNoteAsPdf = async (note: Pick<Note, 'title' | 'content'>): Promise<PdfExportResult | null> => {
  const defaultPath = `${sanitizePdfFileName(note.title || 'Untitled')}.pdf`
  const selectedPath = await save({
    defaultPath,
    filters: [
      {
        name: 'PDF',
        extensions: ['pdf']
      }
    ]
  })

  if (!selectedPath) {
    return null
  }

  const filePath = ensurePdfExtension(selectedPath)
  const renderedHtml = await renderMarkdown(note.content)
  const wrapper = createExportContainer(renderedHtml)

  try {
    const contentElement = wrapper.querySelector('.pdf-export-content')
    if (!(contentElement instanceof HTMLElement)) {
      throw new Error('PDF export content container was not created')
    }

    const imageResult = await waitForImages(contentElement)
    const pdfBytes = await createPdfBytes(contentElement)
    await writeBinaryFile(filePath, pdfBytes)

    return {
      filePath,
      missingImages: imageResult.failed
    }
  } finally {
    wrapper.remove()
  }
}
```

- [ ] **Step 2: Build-check the utility**

Run:

```bash
npm run build
```

Expected: TypeScript accepts `pdfExport.ts` and the `html2pdf.js` declaration.

## Task 4: Route Export Format Requests From ActionBar

**Files:**
- Modify: `src/components/ActionBar.vue`

- [ ] **Step 1: Add an export event type**

Add after imports and before store setup:

```ts
type ExportFormat = 'markdown' | 'pdf'

const emit = defineEmits<{
  (event: 'export-note', format: ExportFormat): void
}>()
```

- [ ] **Step 2: Replace direct Markdown export logic with format selection**

Replace `exportNote` with:

```ts
const exportNote = async () => {
  if (!currentNote.value) return

  try {
    const format = await ElMessageBox.confirm(
      'Choose an export format for this note.',
      'Export Note',
      {
        confirmButtonText: 'PDF',
        cancelButtonText: 'Markdown',
        distinguishCancelAndClose: true,
        type: 'info',
        center: true,
        draggable: true
      }
    )

    if (format === 'confirm') {
      emit('export-note', 'pdf')
    }
  } catch (err) {
    if (err === 'cancel') {
      emit('export-note', 'markdown')
      return
    }

    if (err !== 'close') {
      console.error('Failed to choose export format:', err)
      ElMessage.error('导出失败')
    }
  }
}
```

- [ ] **Step 3: Remove unused Tauri dialog imports from `ActionBar.vue`**

Remove:

```ts
import { message } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri'
```

Keep Element Plus imports because `ElMessageBox` and `ElMessage` are still used.

- [ ] **Step 4: Build-check event typing**

Run:

```bash
npm run build
```

Expected: build fails only if the parent has not yet handled the required event. That is resolved in Task 5.

## Task 5: Coordinate Save-Before-Export In ContentPane

**Files:**
- Modify: `src/components/ContentPane.vue`

- [ ] **Step 1: Listen to the export event**

Change:

```vue
<ActionBar />
```

to:

```vue
<ActionBar @export-note="handleExportNote" />
```

- [ ] **Step 2: Add imports**

Add to the script imports:

```ts
import { invoke } from '@tauri-apps/api/tauri'
import { message } from '@tauri-apps/api/dialog'
import { ElMessage } from 'element-plus'
import { exportNoteAsPdf } from '@/utils/pdfExport'
```

- [ ] **Step 3: Track the latest edit-mode draft**

Add near the existing refs:

```ts
const editContent = ref('')
```

Update `handleContentUpdate` to:

```ts
const handleContentUpdate = (content: string) => {
  editContent.value = content
}
```

Update the `currentNote` watcher to set `editContent.value = newNote.content` when a note exists, and reset it to `''` when no note exists.

- [ ] **Step 4: Add helpers for latest content and saving**

Add below `updateTitle`:

```ts
type ExportFormat = 'markdown' | 'pdf'

const getLatestContent = () => {
  if (viewMode.value === 'split') {
    return splitContent.value
  }

  if (viewMode.value === 'edit') {
    return editContent.value
  }

  return currentNote.value?.content || ''
}

const saveCurrentDraft = async () => {
  if (!currentNote.value) {
    return null
  }

  saveStatus.value = 'Saving...'
  const updatedNote = await notesStore.updateNote(currentNote.value.id, {
    title: noteTitle.value,
    content: getLatestContent()
  })
  saveStatus.value = 'Saved'

  if (updatedNote) {
    noteTitle.value = updatedNote.title
    editContent.value = updatedNote.content
    splitContent.value = updatedNote.content
  }

  return updatedNote
}
```

- [ ] **Step 5: Add Markdown export handler using existing commands**

Add:

```ts
const exportMarkdownNote = async () => {
  if (!currentNote.value) return

  const exportPath = await invoke('show_export_dialog')

  if (!exportPath) {
    return
  }

  const result = await invoke('export_note', {
    noteId: currentNote.value.id,
    exportPath
  })

  await message(result?.toString() || 'Export completed successfully', {
    title: 'Export Successful',
    type: 'info'
  })
}
```

- [ ] **Step 6: Add PDF export handler with save-before-export**

Add:

```ts
const exportPdfNote = async () => {
  const savedNote = await saveCurrentDraft()

  if (!savedNote) {
    return
  }

  const result = await exportNoteAsPdf(savedNote)

  if (!result) {
    return
  }

  if (result.missingImages > 0) {
    ElMessage.warning(`PDF exported, but ${result.missingImages} image(s) may be missing.`)
  } else {
    ElMessage.success('PDF exported successfully')
  }
}
```

- [ ] **Step 7: Add the main export router with errors**

Add:

```ts
const handleExportNote = async (format: ExportFormat) => {
  if (!currentNote.value) return

  try {
    if (format === 'markdown') {
      await exportMarkdownNote()
      return
    }

    await exportPdfNote()
  } catch (err) {
    saveStatus.value = saveStatus.value === 'Saving...' ? 'Error saving' : saveStatus.value
    console.error('Failed to export note:', err)
    await message(`Failed to export note: ${err}`, {
      title: 'Export Failed',
      type: 'error'
    })
  }
}
```

- [ ] **Step 8: Build-check export coordination**

Run:

```bash
npm run build
```

Expected: Vue and TypeScript build successfully.

## Task 6: Final Verification And Cleanup

**Files:**
- Verify: all changed files

- [ ] **Step 1: Run frontend build**

Run:

```bash
npm run build
```

Expected: build exits successfully.

- [ ] **Step 2: Run backend tests**

Run:

```bash
cd src-tauri && cargo test
```

Expected: all Rust tests pass.

- [ ] **Step 3: Inspect git diff**

Run:

```bash
git diff --stat
git diff -- package.json src/components/ActionBar.vue src/components/ContentPane.vue src/components/ViewPane.vue src/main.ts src/styles/markdown-content.scss src/utils/pdfExport.ts src/types/html2pdf.d.ts
```

Expected: diff only contains the PDF export feature, shared style extraction, and dependency/type changes.

- [ ] **Step 4: Manual app verification**

Run the app:

```bash
npm run dev
```

Manual checks:

- Choose `Markdown` from the export dialog and confirm the existing Markdown export still creates the same export folder.
- Choose `PDF`, cancel the save dialog, and confirm no error appears.
- Edit a note without manually saving, export PDF, and confirm the latest draft appears.
- Export a note with a table and local image and confirm both are visible in the PDF.

- [ ] **Step 5: Commit implementation**

```bash
git add package.json package-lock.json src/types/html2pdf.d.ts src/styles/markdown-content.scss src/main.ts src/components/ViewPane.vue src/utils/pdfExport.ts src/components/ActionBar.vue src/components/ContentPane.vue
git commit -m "feat: add pdf export"
```

## Self-Review

- Spec coverage: covered format selection, Markdown behavior preservation, save-before-export, shared preview rendering and styles, A4 portrait PDF generation, image waiting, error handling, and verification.
- Placeholder scan: no placeholder tasks remain.
- Type consistency: `ExportFormat`, `exportNoteAsPdf()`, `PdfExportResult`, and `Note` usage are consistent across tasks.
