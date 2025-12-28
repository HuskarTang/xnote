import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import type { Tag, NoteWithTags } from '@/types';

export const useTagsStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  const sortedTags = computed(() => 
    [...tags.value].sort((a, b) => a.name.localeCompare(b.name))
  );

  async function loadTags() {
    isLoading.value = true;
    error.value = null;

    try {
      tags.value = await invoke<Tag[]>('get_all_tags');
    } catch (err) {
      error.value = err as string;
      console.error('Failed to load tags:', err);
    } finally {
      isLoading.value = false;
    }
  }

  async function addTagToNote(noteId: string, tagName: string) {
    error.value = null;

    try {
      const tag = await invoke<Tag>('add_tag_to_note', { 
        noteId, 
        tagName 
      });
      
      // Add tag to list if it's new
      if (!tags.value.find(t => t.id === tag.id)) {
        tags.value.push(tag);
      }
      
      return tag;
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function removeTagFromNote(noteId: string, tagId: string) {
    error.value = null;

    try {
      await invoke('remove_tag_from_note', { 
        noteId, 
        tagId 
      });
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  async function getNotesByTag(tagName: string) {
    isLoading.value = true;
    error.value = null;

    try {
      const notes = await invoke<NoteWithTags[]>('get_notes_by_tag', { 
        tagName 
      });
      return notes;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function getUntaggedNotes() {
    isLoading.value = true;
    error.value = null;

    try {
      const notes = await invoke<NoteWithTags[]>('get_untagged_notes');
      return notes;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function getFavoriteNotes() {
    isLoading.value = true;
    error.value = null;

    try {
      const notes = await invoke<NoteWithTags[]>('get_favorite_notes');
      return notes;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function getTrashedNotes() {
    isLoading.value = true;
    error.value = null;

    try {
      const notes = await invoke<NoteWithTags[]>('get_trashed_notes');
      return notes;
    } catch (err) {
      error.value = err as string;
      throw err;
    } finally {
      isLoading.value = false;
    }
  }

  async function createTag(tagName: string, color?: string) {
    error.value = null;

    try {
      const tag = await invoke<Tag>('create_tag', { 
        tagName, 
        color 
      });
      
      // Add to list if not already present
      if (!tags.value.find(t => t.id === tag.id)) {
        tags.value.push(tag);
      }
      
      return tag;
    } catch (err) {
      error.value = err as string;
      throw err;
    }
  }

  function clearError() {
    error.value = null;
  }

  return {
    tags,
    isLoading,
    error,
    sortedTags,
    loadTags,
    addTagToNote,
    removeTagFromNote,
    getNotesByTag,
    getUntaggedNotes,
    getFavoriteNotes,
    getTrashedNotes,
    createTag,
    clearError,
  };
});