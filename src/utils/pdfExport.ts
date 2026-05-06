import { save } from '@tauri-apps/api/dialog'
import { writeBinaryFile } from '@tauri-apps/api/fs'
import type { Note } from '@/types'
import { renderMarkdown } from '@/utils/markdown'

export interface PdfExportResult {
  filePath: string
  missingImages: number
}

interface ImageLoadResult {
  loaded: number
  failed: number
}

const IMAGE_LOAD_TIMEOUT_MS = 10000

export const sanitizePdfFileName = (title: string): string => {
  const invalidChars = /[<>:"/\\|?*\x00-\x1f]/g
  const sanitized = title.replace(invalidChars, '_').trim()
  const baseName = sanitized || 'Untitled'
  const truncated = baseName.length > 100 ? baseName.slice(0, 100) : baseName
  return truncated.trim() || 'Untitled'
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
      let settled = false
      const finish = (loadedSuccessfully: boolean) => {
        if (settled) {
          return
        }

        settled = true
        window.clearTimeout(timeoutId)

        if (loadedSuccessfully) {
          loaded += 1
        } else {
          failed += 1
        }

        resolve()
      }

      const timeoutId = window.setTimeout(() => finish(false), IMAGE_LOAD_TIMEOUT_MS)

      image.onload = () => {
        finish(true)
      }
      image.onerror = () => {
        finish(false)
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
  const { default: html2pdf } = await import('html2pdf.js')
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
