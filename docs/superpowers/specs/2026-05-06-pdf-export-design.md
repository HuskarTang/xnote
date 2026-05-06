# PDF Export Design

## Context

XNote currently supports Markdown export from the toolbar export button. The existing flow in `ActionBar.vue` asks the user to choose an export directory, then calls the Tauri `export_note` command. The Rust command writes the Markdown file and copies referenced attachments into an export folder.

The rendered note preview is produced on the frontend. `ViewPane.vue` calls `renderMarkdown()` from `src/utils/markdown.ts`, which handles Markdown parsing, code highlighting, table wrapping, and local image path conversion through Tauri asset URLs. To make PDF output match the preview, PDF export should reuse this frontend rendering path instead of recreating Markdown rendering in Rust.

## Goals

- Add PDF as an export format without removing or changing the existing Markdown export behavior.
- Export the current note's rendered Markdown content as a PDF.
- Preserve preview styling for headings, paragraphs, lists, code blocks, tables, links, and images.
- Include local images, HTML image tags, data URLs, and remote HTTPS images where they can be loaded by the current app security policy.
- Use A4 portrait output with 12 mm margins and automatic pagination.
- Save the PDF to a user-selected `.pdf` path.
- Save the current note before exporting so the PDF matches the persisted note content.

## Non-Goals

- No PDF export settings in the first version.
- No page headers, footers, page numbers, table of contents, or metadata editor.
- No backend Markdown-to-PDF renderer.
- No change to the existing Markdown export directory structure.
- No new frontend test runner as a prerequisite for this feature.

## User Flow

1. The user clicks the existing export button.
2. XNote shows a format choice with `Markdown` and `PDF`.
3. If the user chooses `Markdown`, XNote runs the current Markdown export flow unchanged.
4. If the user chooses `PDF`, XNote saves the current note first.
5. XNote shows a save-file dialog filtered to PDF files, using `<note-title>.pdf` as the default filename.
6. If the user chooses a path, XNote renders the note content and generates a PDF.
7. XNote writes the PDF to the selected path and shows a success message.

Canceling either the format choice or save-file dialog exits quietly.

## Architecture

### Shared Markdown Rendering

`src/utils/markdown.ts` remains the single Markdown-to-HTML rendering entry point for preview and PDF export. PDF export should call `renderMarkdown(content)` so code highlighting, table HTML, and image URL conversion stay aligned with the preview.

### Shared Preview Styles

Move the Markdown content styles currently scoped inside `ViewPane.vue` into a reusable stylesheet, for example:

- `src/styles/markdown-content.scss`

Both `ViewPane.vue` and the PDF export DOM should use the same root class:

```html
<div class="markdown-content">...</div>
```

This avoids drift between preview styling and PDF styling. `ViewPane.vue` can keep container-specific layout and scrollbar styles locally.

### Export Coordination

`ContentPane.vue` should coordinate export requests because it owns the editable title and draft content state. `ActionBar.vue` should emit an export request instead of directly reading and exporting `currentNote` for the PDF path.

The export coordinator should derive the latest content from the active mode:

- View mode: `currentNote.content`.
- Edit mode: the latest draft emitted by `EditPane` through `update:content`.
- Split mode: `splitContent`.

Before PDF export starts, `ContentPane.vue` should save the latest title and content through `notesStore.updateNote()`. The export should use the saved note returned by the store update, not a stale `currentNote` snapshot.

### PDF Export Utility

Add a frontend utility such as `src/utils/pdfExport.ts`. Its responsibilities:

- Sanitize the note title for a default PDF filename.
- Open the Tauri `save()` dialog with a PDF filter.
- Render Markdown to HTML through `renderMarkdown()`.
- Create an offscreen export container using the shared `.markdown-content` class.
- Wait for images in the export container to load or fail.
- Generate an A4 portrait PDF from the DOM.
- Write the PDF bytes to the selected path through Tauri filesystem APIs.
- Remove temporary DOM nodes after export succeeds or fails.

`ActionBar.vue` should only present the export action and emit the request. `ContentPane.vue` should handle save-before-export and call this utility with the saved note data.

### PDF Generation

Use a frontend DOM-to-PDF approach so the generated output follows browser-rendered HTML and CSS. The implementation should use A4 portrait settings with 12 mm margins. A library such as `html2pdf.js` or an equivalent `html2canvas` + `jsPDF` wrapper is suitable because it can render the same DOM and CSS used by the preview. The generated PDF should preserve rendered images, tables, code block styling, colors, and typography as closely as practical.

The first implementation should prefer visual consistency over adding advanced PDF semantics. Text selection quality is secondary to matching the preview.

## Data Flow

1. `ActionBar.vue` receives the export button click.
2. Element Plus asks the user to choose `Markdown` or `PDF`.
3. `ActionBar.vue` emits the selected export format to `ContentPane.vue`.
4. For `Markdown`, `ContentPane.vue` calls the existing Tauri commands unchanged.
5. For `PDF`, `ContentPane.vue` saves the latest title and content first.
6. `pdfExport.ts` asks the user for a target PDF path.
7. `pdfExport.ts` calls `renderMarkdown(savedNote.content)`.
8. The rendered HTML is mounted into a temporary `.markdown-content` export container.
9. The utility waits for image loading to settle.
10. The PDF library renders the export container to A4 portrait PDF output.
11. The utility writes the resulting PDF bytes to the selected path.
12. `ContentPane.vue` shows success or error feedback.

## Error Handling

- No current note: do nothing.
- User cancels format choice: do nothing.
- User cancels the save-file dialog: do nothing.
- Auto-save fails: stop export and show that the note could not be saved, so PDF export was canceled.
- Markdown rendering fails: use the existing `renderMarkdown()` fallback output and log the failure.
- Some images fail to load: continue export, but report that some images may be missing.
- PDF generation fails: show an export error and do not show a success message.
- File writing fails: show an export error and do not show a success message.

Temporary export DOM nodes must be removed in a `finally` path.

## Layout And Pagination

- Default paper: A4.
- Orientation: portrait.
- Margins: 12 mm on all sides.
- Content width should fit the printable page.
- Images should use `max-width: 100%` and `height: auto`.
- Tables and code blocks should avoid overflowing the page where possible. Wide content may be scaled or wrapped according to the chosen frontend PDF library's capabilities.
- Long content should paginate automatically.

## Testing And Verification

Automated verification:

- Run `npm run build` for frontend TypeScript and Vue validation.
- Run `cd src-tauri && cargo test` to confirm backend behavior still passes.

Manual verification:

- Export Markdown and confirm the existing behavior is unchanged.
- Export PDF for a note containing headings, paragraphs, lists, blockquotes, code blocks, tables, local Markdown images, HTML `<img>` tags, data URL images, and long content.
- Compare the PDF against the preview for visual consistency.
- Confirm local images appear in the PDF.
- Confirm table borders, code block styles, and heading hierarchy match the preview.
- Confirm canceling the format choice and save dialog shows no error.
- Edit a note without manually saving, export PDF, and confirm the latest content appears in the PDF.

## Implementation Notes

- Keep the backend `export_note` command focused on Markdown export.
- Prefer a small, focused `pdfExport.ts` utility over expanding `ActionBar.vue`.
- Keep save-before-export in `ContentPane.vue`, where the latest draft content is available.
- Keep filename sanitization consistent with Rust's `sanitize_filename()` behavior for invalid filesystem characters.
- Do not commit generated PDFs or local note data.
