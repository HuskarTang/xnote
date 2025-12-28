<template>
  <div class="editor-area">
    <div v-if="!currentNote" class="welcome-screen">
      <div class="welcome-content">
        <div class="welcome-icon">📝</div>
        <h2>Welcome to XNote</h2>
        <p>Select a note from the list or create a new one to get started.</p>
      </div>
    </div>

    <div v-else class="editor-container">
      <!-- Toolbar -->
      <div class="editor-toolbar">
        <div class="toolbar-left">
          <button
            :class="['toolbar-btn', { active: viewMode === 'view' }]"
            @click="setViewMode('view')"
            title="View Mode"
          >
            👁️
          </button>
          <button
            :class="['toolbar-btn', { active: viewMode === 'edit' }]"
            @click="setViewMode('edit')"
            title="Edit Mode"
          >
            ✏️
          </button>
          <button
            v-if="viewMode === 'edit'"
            :class="['toolbar-btn', { active: viewMode === 'split' }]"
            @click="toggleSplitMode"
            title="Split View"
          >
            📱
          </button>
        </div>

        <div class="toolbar-center">
          <input
            v-if="isEditMode"
            v-model="noteTitle"
            @blur="saveNote"
            class="note-title-input"
            placeholder="Note title..."
          />
          <h1 v-else class="note-title-display">{{ currentNote.title }}</h1>
        </div>

        <div class="toolbar-right">
          <button
            @click="toggleFavorite"
            :class="['toolbar-btn', { active: currentNote.is_favorite }]"
            title="Toggle Favorite"
          >
            ⭐
          </button>
          <button
            @click="showTagManager = !showTagManager"
            class="toolbar-btn"
            title="Manage Tags"
          >
            🏷️
          </button>
          <button
            @click="showAttachments = !showAttachments"
            class="toolbar-btn"
            title="Attachments"
          >
            📎
          </button>
          <button
            @click="moveToTrash"
            class="toolbar-btn danger"
            title="Move to Trash"
          >
            🗑️
          </button>
          <button
            @click="showExportMenu = !showExportMenu"
            class="toolbar-btn"
            title="Export"
          >
            📤
          </button>
        </div>
      </div>

      <!-- Tag Manager -->
      <TagManager
        v-if="showTagManager"
        :note-id="currentNote.id"
        :current-tags="currentNote.tags"
        @close="showTagManager = false"
        @tags-updated="onTagsUpdated"
      />

      <!-- Editor Content -->
      <div class="editor-content" :class="{ 'split-view': isSplitMode }">
        <!-- Edit View -->
        <div v-if="isEditMode || isSplitMode" class="editor-pane">
          <MarkdownEditor
            v-model="noteContent"
            @change="onContentChange"
            @save="saveNote"
          />
        </div>

        <!-- Preview View -->
        <div v-if="isViewMode || isSplitMode" class="preview-pane">
          <MarkdownPreview :content="noteContent" />
        </div>
      </div>

      <!-- Status Bar -->
      <div class="status-bar">
        <div class="status-left">
          <span v-if="isDirty" class="status-indicator unsaved">Unsaved changes</span>
          <span v-else-if="lastSaved" class="status-indicator saved">
            Saved {{ formatDistanceToNow(lastSaved, { addSuffix: true }) }}
          </span>
        </div>
        <div class="status-right">
          <span class="word-count">{{ wordCount }} words</span>
          <span class="char-count">{{ charCount }} characters</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useNotesStore } from '@/stores/notes';
import { useEditorStore } from '@/stores/editor';
import { formatDistanceToNow } from 'date-fns';
import MarkdownEditor from './MarkdownEditor.vue';
import MarkdownPreview from './MarkdownPreview.vue';
import TagManager from './TagManager.vue';

const notesStore = useNotesStore();
const editorStore = useEditorStore();

const showTagManager = ref(false);
const showAttachments = ref(false);
const showExportMenu = ref(false);
const noteTitle = ref('');
const noteContent = ref('');
const autoSaveTimer = ref<number | null>(null);

const currentNote = computed(() => notesStore.currentNote);
const viewMode = computed(() => editorStore.viewMode);
const isEditMode = computed(() => editorStore.isEditMode);
const isViewMode = computed(() => editorStore.isViewMode);
const isSplitMode = computed(() => editorStore.isSplitMode);
const isDirty = computed(() => editorStore.isDirty);
const lastSaved = computed(() => editorStore.lastSaved);

const wordCount = computed(() => {
  if (!noteContent.value) return 0;
  return noteContent.value.trim().split(/\s+/).filter(word => word.length > 0).length;
});

const charCount = computed(() => {
  return noteContent.value.length;
});

function setViewMode(mode: 'view' | 'edit') {
  editorStore.setViewMode(mode);
}

function toggleSplitMode() {
  if (isSplitMode.value) {
    editorStore.setViewMode('edit');
  } else {
    editorStore.setViewMode('split');
  }
}

function onContentChange() {
  editorStore.setDirty(true);
  scheduleAutoSave();
}

function scheduleAutoSave() {
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }
  
  autoSaveTimer.value = setTimeout(() => {
    saveNote();
  }, 2000); // Auto-save after 2 seconds of inactivity
}

async function saveNote() {
  if (!currentNote.value || !isDirty.value) return;
  
  try {
    await notesStore.saveNote({
      id: currentNote.value.id,
      title: noteTitle.value || 'Untitled',
      content: noteContent.value,
    });
    
    editorStore.markSaved();
    
    if (autoSaveTimer.value) {
      clearTimeout(autoSaveTimer.value);
      autoSaveTimer.value = null;
    }
  } catch (error) {
    console.error('Failed to save note:', error);
  }
}

async function toggleFavorite() {
  if (!currentNote.value) return;
  
  try {
    await notesStore.toggleFavorite(currentNote.value.id);
  } catch (error) {
    console.error('Failed to toggle favorite:', error);
  }
}

async function moveToTrash() {
  if (!currentNote.value) return;
  
  if (confirm('Are you sure you want to move this note to trash?')) {
    try {
      await notesStore.moveToTrash(currentNote.value.id);
    } catch (error) {
      console.error('Failed to move note to trash:', error);
    }
  }
}

function onTagsUpdated() {
  // Reload the current note to get updated tags
  if (currentNote.value) {
    notesStore.loadNoteContent(currentNote.value.id);
  }
}

// Watch for note changes
watch(currentNote, (newNote) => {
  if (newNote) {
    noteTitle.value = newNote.title;
    noteContent.value = newNote.content;
    editorStore.setDirty(false);
  } else {
    noteTitle.value = '';
    noteContent.value = '';
  }
}, { immediate: true });

// Keyboard shortcuts
function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    saveNote();
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
  if (autoSaveTimer.value) {
    clearTimeout(autoSaveTimer.value);
  }
});
</script>

<style lang="scss" scoped>
.editor-area {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: white;
}

.welcome-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: #f8f9fa;
}

.welcome-content {
  text-align: center;
  color: #6c757d;
}

.welcome-icon {
  font-size: 4rem;
  margin-bottom: 20px;
}

.editor-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e1e5e9;
  background: #f8f9fa;
  gap: 16px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  gap: 4px;
}

.toolbar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  transition: all 0.2s;
  
  &:hover {
    background: #e9ecef;
  }
  
  &.active {
    background: #007acc;
    color: white;
  }
  
  &.danger:hover {
    background: #dc3545;
    color: white;
  }
}

.note-title-input {
  font-size: 1.25rem;
  font-weight: 600;
  border: none;
  background: transparent;
  text-align: center;
  min-width: 200px;
  
  &:focus {
    outline: none;
    background: white;
    border-radius: 4px;
    padding: 4px 8px;
  }
}

.note-title-display {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  color: #2c3e50;
}

.editor-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  
  &.split-view {
    .editor-pane,
    .preview-pane {
      flex: 1;
      border-right: 1px solid #e1e5e9;
      
      &:last-child {
        border-right: none;
      }
    }
  }
}

.editor-pane,
.preview-pane {
  height: 100%;
  overflow: hidden;
}

.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  border-top: 1px solid #e1e5e9;
  background: #f8f9fa;
  font-size: 12px;
  color: #6c757d;
}

.status-left,
.status-right {
  display: flex;
  gap: 16px;
}

.status-indicator {
  &.unsaved {
    color: #ffc107;
  }
  
  &.saved {
    color: #28a745;
  }
}
</style>