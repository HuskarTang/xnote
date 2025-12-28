import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Note } from '@/types'
import { api } from '@/utils/api'

export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([])
  const currentNote = ref<Note | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const sortedNotes = computed(() => {
    return [...notes.value].sort((a, b) => 
      new Date(b.modified_at).getTime() - new Date(a.modified_at).getTime()
    )
  })

  async function loadNotes() {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.getAllNotes()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load notes'
    } finally {
      loading.value = false
    }
  }

  async function loadFavorites() {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.getFavorites()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load favorites'
    } finally {
      loading.value = false
    }
  }

  async function loadUntagged() {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.getUntagged()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load untagged notes'
    } finally {
      loading.value = false
    }
  }

  async function loadTrash() {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.getTrash()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load trash'
    } finally {
      loading.value = false
    }
  }

  async function loadNotesByTag(tagName: string) {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.getNotesByTag(tagName)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load notes by tag'
    } finally {
      loading.value = false
    }
  }

  async function createNote(title: string, content?: string) {
    try {
      const note = await api.createNote({ title, content })
      notes.value.unshift(note)
      currentNote.value = note
      return note
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create note'
      throw err
    }
  }

  async function updateNote(id: string, updates: Partial<Note>) {
    try {
      const updatedNote = await api.updateNote({ id, ...updates })
      if (updatedNote) {
        const index = notes.value.findIndex(n => n.id === id)
        if (index !== -1) {
          notes.value[index] = updatedNote
        }
        if (currentNote.value?.id === id) {
          currentNote.value = updatedNote
        }
      }
      return updatedNote
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update note'
      throw err
    }
  }

  async function deleteNote(id: string) {
    try {
      await api.deleteNote(id)
      const index = notes.value.findIndex(n => n.id === id)
      if (index !== -1) {
        notes.value.splice(index, 1)
      }
      if (currentNote.value?.id === id) {
        currentNote.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete note'
      throw err
    }
  }

  async function searchNotes(query: string, tagFilter?: string) {
    try {
      loading.value = true
      error.value = null
      notes.value = await api.searchNotes({ query, tag_filter: tagFilter })
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to search notes'
    } finally {
      loading.value = false
    }
  }

  function setCurrentNote(note: Note | null) {
    currentNote.value = note
  }

  return {
    notes,
    currentNote,
    loading,
    error,
    sortedNotes,
    loadNotes,
    loadFavorites,
    loadUntagged,
    loadTrash,
    loadNotesByTag,
    createNote,
    updateNote,
    deleteNote,
    searchNotes,
    setCurrentNote,
  }
})