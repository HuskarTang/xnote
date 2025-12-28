<template>
  <div class="markdown-editor">
    <div ref="editorContainer" class="editor-container"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';
import { markdown } from '@codemirror/lang-markdown';
import { oneDark } from '@codemirror/theme-one-dark';

interface Props {
  modelValue: string;
}

interface Emits {
  (e: 'update:modelValue', value: string): void;
  (e: 'change'): void;
  (e: 'save'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const editorContainer = ref<HTMLElement>();
let editorView: EditorView | null = null;

function createEditor() {
  if (!editorContainer.value) return;

  const state = EditorState.create({
    doc: props.modelValue,
    extensions: [
      basicSetup,
      markdown(),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          const newValue = update.state.doc.toString();
          emit('update:modelValue', newValue);
          emit('change');
        }
      }),
      EditorView.theme({
        '&': {
          height: '100%',
          fontSize: '14px',
        },
        '.cm-content': {
          padding: '20px',
          fontFamily: 'SF Mono, Monaco, Cascadia Code, Roboto Mono, Consolas, Courier New, monospace',
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
        },
      }),
      // Keyboard shortcuts
      EditorView.domEventHandlers({
        keydown: (event) => {
          if ((event.ctrlKey || event.metaKey) && event.key === 's') {
            event.preventDefault();
            emit('save');
            return true;
          }
          return false;
        },
      }),
    ],
  });

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  });
}

function updateEditor() {
  if (!editorView) return;
  
  const currentValue = editorView.state.doc.toString();
  if (currentValue !== props.modelValue) {
    editorView.dispatch({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: props.modelValue,
      },
    });
  }
}

watch(() => props.modelValue, updateEditor);

onMounted(() => {
  createEditor();
});

onUnmounted(() => {
  if (editorView) {
    editorView.destroy();
  }
});
</script>

<style lang="scss" scoped>
.markdown-editor {
  height: 100%;
  overflow: hidden;
}

.editor-container {
  height: 100%;
  
  :deep(.cm-editor) {
    height: 100%;
  }
  
  :deep(.cm-content) {
    min-height: 100%;
  }
}
</style>