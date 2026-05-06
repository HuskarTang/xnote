declare module 'html2pdf.js' {
  interface Html2PdfWorker {
    from(element: HTMLElement): Html2PdfWorker
    set(options: Record<string, unknown>): Html2PdfWorker
    outputPdf(type: 'arraybuffer'): Promise<ArrayBuffer>
  }

  function html2pdf(): Html2PdfWorker

  export default html2pdf
}
