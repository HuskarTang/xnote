import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { NoteWithTags, NoteContent, CreateNoteRequest, SaveNoteRequest, SearchRequest } from '@/types';

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<NoteWithTags[]>([]);
  const currentNote = ref<NoteContent | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const searchQuery = ref('');
  const selectedTagFilter = ref<string | null>(null);

  const filteredNotes = computed(() => {
    let filtered = notes.value;

    if (searchQuery.value) {
      const query = searchQuery.value.toLowerCase();
      filtered = filtered.filter(noteWithTags => 
        noteWithTags.note.title.toLowerCase().includes(query) ||
        noteWithTags.tags.some(tag => tag.name.toLowerCase().includes(query))
      );
    }

    if (selectedTagFilter.value) {
      filtered = filtered.filter(noteWithTags =>
        noteWithTags.tags.some(tag => tag.name === selectedTagFilter.value)
      );
    }

    return filtered;
  });

  const favoriteNotes = computed(() => 
    notes.value.filter(noteWithTags => noteWithTags.note.is_favorite && !noteWithTags.note.is_trashed)
  );

  const trashedNotes = computed(() => 
    notes.value.filter(noteWithTags => noteWithTags.note.is_trashed)
  );

  const untaggedNotes = computed(() => 
    notes.value.filter(noteWithTags => 
      noteWithTags.tags.length === 0 && !noteWithTags.note.is_trashed
    )
  );

  async function loadNotes(includeTrash = false) {
    isLoading.value = true;
    error.value = null;

    try {
      notes.value = await invoke<NoteWithTags[]>('get_all_notes', { 
        includeTrash 
      });
    } catch (err) {
      error.value = err as string;
      console.error('Failed to load notes:', err);
    } finally {
      isLoading.value = false;
    }
  }

  async function loadNoteContent(noteId: string) {
    isLoading.value = true;
    error.value = null;

    try {
      currentNote.value = await invoke<NoteContent>('get_note_content', { 
        noteId 
      });
    } catch (err) {
      error.value = err as string;
      console.error('Failed to load note content:', err);
    } finally {
      isLoading.value = false;
    }
  }

  async function createNote(request: CreateNoteRequest) {
    isLoading.value = true;
    error.value = null;

    try {
      const newNote = await invoke<NoteContent>('create_note', { request });
      currentNote.value = newNote;
      
      // Reload notes list
      await loadNotes();
      
      return newNote;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveNote(request: SaveNoteRequest) {
    error.value = null;

    try {
      await invoke('save_note', { request });
      
      // Update current note if it's the same one
      if (currentNote.value && currentNote.value.id === request.id) {
        currentNote.value.title = request.title;
        currentNote.value.content = request.content;
        currentNote.value.modified_at = new Date().toISOString();
      }
      
      // Reload notes list to update the title in the list
      await loadNotes();
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function deleteNote(noteId: string) {
    error.value = null;

    try {
      await invoke('delete_note', { noteId });
      
      // Clear current note if it was deleted
      if (currentNote.value && currentNote.value.id === noteId) {
        currentNote.value = null;
      }
      
      // Reload notes list
      await loadNotes();
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function moveToTrash(noteId: string) {
    error.value = null;

    try {
      await invoke('move_to_trash', { noteId });
      
      // Clear current note if it was trashed
      if (currentNote.value && currentNote.value.id === noteId) {
        currentNote.value = null;
      }
      
      // Reload notes list
      await loadNotes();
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function restoreFromTrash(noteId: string) {
    error.value = null;

    try {
      await invoke('restore_from_trash', { noteId });
      
      // Reload notes list
      await loadNotes();
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function toggleFavorite(noteId: string) {
    error.value = null;

    try {
      const isFavorite = await invoke<boolean>('toggle_favorite', { noteId });
      
      // Update current note if it's the same one
      if (currentNote.value && currentNote.value.id === noteId) {
        currentNote.value.is_favorite = isFavorite;
      }
      
      // Reload notes list
      await loadNotes();
      
      return isFavorite;
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function searchNotes(request: SearchRequest) {
    isLoading.value = true;
    error.value = null;

    try {
      const results = await invoke<NoteWithTags[]>('search_notes', { request });
      return results;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query;
  }

  function setTagFilter(tagName: string | null) {
    selectedTagFilter.value = tagName;
  }

  function clearCurrentNote() {
    currentNote.value = null;
  }

  function clearError() {
    error.value = null;
  }

  return {
    notes,
    currentNote,
    isLoading,
    error,
    searchQuery,
    selectedTagFilter,
    filteredNotes,
    favoriteNotes,
    trashedNotes,
    untaggedNotes,
    loadNotes,
    loadNoteContent,
    createNote,
    saveNote,
    deleteNote,
    moveToTrash,
    restoreFromTrash,
    toggleFavorite,
    searchNotes,
    setSearchQuery,
    setTagFilter,
    clearCurrentNote,
    clearError,
  };
});