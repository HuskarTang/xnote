<template>
  <div class="main-layout">
    <div class="layout-container" :style="layoutStyles">
      <!-- Sidebar (TagPane) -->
      <div class="sidebar" :style="{ width: sidebarWidth + 'px' }">
        <Sidebar />
      </div>

      <!-- Resize handle for sidebar -->
      <div 
        class="resize-handle sidebar-resize"
        @mousedown="startResize('sidebar', $event)"
      ></div>

      <!-- Note List (NotePane) -->
      <div class="note-list" :style="{ width: noteListWidth + 'px' }">
        <NoteList />
      </div>

      <!-- Resize handle for note list -->
      <div 
        class="resize-handle notelist-resize"
        @mousedown="startResize('notelist', $event)"
      ></div>

      <!-- Editor/Viewer (EditViewPane) -->
      <div class="editor-area">
        <EditorArea />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useAppStore } from '@/stores/app';
import { useNotesStore } from '@/stores/notes';
import { useTagsStore } from '@/stores/tags';
import Sidebar from '@/components/Sidebar.vue';
import NoteList from '@/components/NoteList.vue';
import EditorArea from '@/components/EditorArea.vue';

const appStore = useAppStore();
const notesStore = useNotesStore();
const tagsStore = useTagsStore();

// Layout dimensions
const sidebarWidth = ref(250);
const noteListWidth = ref(350);
const isResizing = ref(false);
const resizeType = ref<'sidebar' | 'notelist' | null>(null);

const layoutStyles = computed(() => ({
  height: '100vh',
  display: 'flex',
}));

// Resize functionality
function startResize(type: 'sidebar' | 'notelist', event: MouseEvent) {
  isResizing.value = true;
  resizeType.value = type;
  
  document.addEventListener('mousemove', handleResize);
  document.addEventListener('mouseup', stopResize);
  
  event.preventDefault();
}

function handleResize(event: MouseEvent) {
  if (!isResizing.value || !resizeType.value) return;
  
  const containerRect = document.querySelector('.layout-container')?.getBoundingClientRect();
  if (!containerRect) return;
  
  const mouseX = event.clientX - containerRect.left;
  
  if (resizeType.value === 'sidebar') {
    const newWidth = Math.max(200, Math.min(400, mouseX));
    sidebarWidth.value = newWidth;
  } else if (resizeType.value === 'notelist') {
    const newWidth = Math.max(250, Math.min(500, mouseX - sidebarWidth.value - 4));
    noteListWidth.value = newWidth;
  }
}

function stopResize() {
  isResizing.value = false;
  resizeType.value = null;
  
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
  
  // Save layout preferences
  if (appStore.config) {
    appStore.updateConfig({
      sidebar_width: sidebarWidth.value,
      note_list_width: noteListWidth.value,
    });
  }
}

onMounted(async () => {
  // Load initial data
  await Promise.all([
    notesStore.loadNotes(),
    tagsStore.loadTags(),
  ]);
  
  // Apply saved layout preferences
  if (appStore.config) {
    sidebarWidth.value = appStore.config.sidebar_width || 250;
    noteListWidth.value = appStore.config.note_list_width || 350;
  }
});

onUnmounted(() => {
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
});
</script>

<style lang="scss" scoped>
.main-layout {
  height: 100vh;
  overflow: hidden;
  background: #f5f5f5;
}

.layout-container {
  display: flex;
  height: 100%;
  position: relative;
}

.sidebar {
  background: #2c3e50;
  color: white;
  border-right: 1px solid #34495e;
  flex-shrink: 0;
  overflow: hidden;
}

.note-list {
  background: white;
  border-right: 1px solid #e1e5e9;
  flex-shrink: 0;
  overflow: hidden;
}

.editor-area {
  flex: 1;
  background: white;
  overflow: hidden;
}

.resize-handle {
  width: 4px;
  background: transparent;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  
  &:hover {
    background: #007acc;
  }
  
  &::after {
    content: '';
    position: absolute;
    top: 0;
    left: -2px;
    right: -2px;
    bottom: 0;
  }
}

.sidebar-resize {
  background: #34495e;
}

.notelist-resize {
  background: #e1e5e9;
}
</style>