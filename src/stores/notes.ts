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

  async function createNote(title: string, content?: string, tags?: string[]) {
    try {
      const note = await api.createNote({ title, content, tags })
      notes.value.unshift(note)
      currentNote.value = note
      // Emit an event or trigger a refresh to update the note list
      // For now, we'll just emit a custom event
      window.dispatchEvent(new CustomEvent('note-created', { detail: note }))
      return note
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create note'
      throw err
    }
  }

  async function updateNote(id: string, updates: Partial<Note>) {
    try {
      console.log('Updating note:', id, updates);
      const updatedNote = await api.updateNote({ id, ...updates })
      console.log('Updated note from API:', updatedNote);
      if (updatedNote) {
        const index = notes.value.findIndex(n => n.id === id)
        if (index !== -1) {
          notes.value[index] = updatedNote
        }
        if (currentNote.value?.id === id) {
          console.log('Updating currentNote:', currentNote.value, '->', updatedNote);
          currentNote.value = updatedNote
        }
        // 触发更新事件
        window.dispatchEvent(new CustomEvent('note-updated', { detail: updatedNote }))
      } else {
        throw new Error('Note not found or update failed')
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
        const deletedNote = notes.value[index]
        notes.value.splice(index, 1)
        // 触发删除事件
        window.dispatchEvent(new CustomEvent('note-deleted', { detail: deletedNote }))
        // 触发标签更新事件
        window.dispatchEvent(new CustomEvent('tags-updated'))
      }
      if (currentNote.value?.id === id) {
        currentNote.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete note'
      throw err
    }
  }

  async function permanentlyDeleteNote(id: string) {
    try {
      await api.permanentlyDeleteNote(id)
      const index = notes.value.findIndex(n => n.id === id)
      if (index !== -1) {
        const deletedNote = notes.value[index]
        notes.value.splice(index, 1)
        // 触发永久删除事件
        window.dispatchEvent(new CustomEvent('note-permanently-deleted', { detail: deletedNote }))
        // 触发标签更新事件
        window.dispatchEvent(new CustomEvent('tags-updated'))
      }
      if (currentNote.value?.id === id) {
        currentNote.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to permanently delete note'
      throw err
    }
  }

  async function restoreNote(id: string) {
    try {
      await api.restoreNote(id)
      // 重新获取笔记数据
      const restoredNote = await api.getNote(id)
      if (restoredNote) {
        const index = notes.value.findIndex(n => n.id === id)
        if (index !== -1) {
          notes.value[index] = restoredNote
        } else {
          notes.value.unshift(restoredNote)
        }
        // 触发恢复事件
        window.dispatchEvent(new CustomEvent('note-restored', { detail: restoredNote }))
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to restore note'
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

  async function refreshNote(id: string) {
    try {
      const note = await api.getNote(id)
      if (note) {
        const index = notes.value.findIndex(n => n.id === id)
        if (index !== -1) {
          notes.value[index] = note
        }
        if (currentNote.value?.id === id) {
          currentNote.value = note
        }
      }
      return note
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to refresh note'
      throw err
    }
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
    permanentlyDeleteNote,
    restoreNote,
    searchNotes,
    setCurrentNote,
    refreshNote,
  }
})