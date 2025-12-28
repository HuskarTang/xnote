// Simple test for marked library
import { Marked } from 'marked';
import { markedHighlight } from 'marked-highlight';

const marked = new Marked(
  markedHighlight({
    langPrefix: 'hljs language-',
    highlight(code, lang) {
      return code; // Just return code as is for testing
    }
  })
);

const testContent = `# Hello World

This is a **bold** text and this is *italic* text.

## List

- Item 1
- Item 2
- Item 3
`;

console.log('Input content:');
console.log(testContent);

const result = marked.parse(testContent);
console.log('Parsed result:');
console.log(result);