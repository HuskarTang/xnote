import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ViewMode, EditorState } from '@/types';

export const useEditorStore = defineStore('editor', () => {
  const viewMode = ref<ViewMode>('view');
  const isPreviewVisible = ref(false);
  const isDirty = ref(false);
  const lastSaved = ref<Date | null>(null);
  const autoSaveEnabled = ref(true);
  const autoSaveInterval = ref(5000); // 5 seconds

  const isEditMode = computed(() => viewMode.value === 'edit');
  const isSplitMode = computed(() => viewMode.value === 'split');
  const isViewMode = computed(() => viewMode.value === 'view');

  function setViewMode(mode: ViewMode) {
    viewMode.value = mode;
    
    // Auto-enable preview in split mode
    if (mode === 'split') {
      isPreviewVisible.value = true;
    }
  }

  function togglePreview() {
    isPreviewVisible.value = !isPreviewVisible.value;
  }

  function setDirty(dirty: boolean) {
    isDirty.value = dirty;
  }

  function markSaved() {
    isDirty.value = false;
    lastSaved.value = new Date();
  }

  function toggleAutoSave() {
    autoSaveEnabled.value = !autoSaveEnabled.value;
  }

  function setAutoSaveInterval(interval: number) {
    autoSaveInterval.value = interval;
  }

  const editorState = computed<EditorState>(() => ({
    currentNote: null, // This will be managed by the notes store
    viewMode: viewMode.value,
    isPreviewVisible: isPreviewVisible.value,
    isDirty: isDirty.value,
    lastSaved: lastSaved.value,
  }));

  return {
    viewMode,
    isPreviewVisible,
    isDirty,
    lastSaved,
    autoSaveEnabled,
    autoSaveInterval,
    isEditMode,
    isSplitMode,
    isViewMode,
    editorState,
    setViewMode,
    togglePreview,
    setDirty,
    markSaved,
    toggleAutoSave,
    setAutoSaveInterval,
  };
});