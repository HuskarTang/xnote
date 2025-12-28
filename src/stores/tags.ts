import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Tag } from '@/types'
import { api } from '@/utils/api'

export const useTagsStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([])
  const selectedTag = ref<string>('All Notes')
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Listen for tag update events
  if (typeof window !== 'undefined') {
    window.addEventListener('tags-updated', () => {
      loadTags()
    })
  }

  async function loadTags() {
    try {
      loading.value = true
      error.value = null
      tags.value = await api.getAllTags()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to load tags'
    } finally {
      loading.value = false
    }
  }

  async function createTag(name: string): Promise<Tag | null> {
    try {
      error.value = null
      const tag = await api.createTag(name)
      // Reload tags to get updated counts
      await loadTags()
      return tag
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create tag'
      return null
    }
  }

  async function deleteTag(tagId: string): Promise<boolean> {
    try {
      error.value = null
      const success = await api.deleteTag(tagId)
      if (success) {
        // Reload tags to get updated list
        await loadTags()
      }
      return success
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete tag'
      return false
    }
  }

  async function renameTag(tagId: string, newName: string): Promise<Tag | null> {
    try {
      error.value = null
      const tag = await api.renameTag(tagId, newName)
      if (tag) {
        // Reload tags to get updated list
        await loadTags()
      }
      return tag
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to rename tag'
      return null
    }
  }

  async function addTagToNote(noteId: string, tagName: string): Promise<Tag | null> {
    try {
      error.value = null
      const tag = await api.addTagToNote(noteId, tagName)
      // Reload tags to get updated counts
      await loadTags()
      return tag
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to add tag to note'
      return null
    }
  }

  async function removeTagFromNote(noteId: string, tagName: string): Promise<boolean> {
    try {
      error.value = null
      const success = await api.removeTagFromNote(noteId, tagName)
      if (success) {
        // Reload tags to get updated counts
        await loadTags()
      }
      return success
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to remove tag from note'
      return false
    }
  }

  async function getNoteTags(noteId: string): Promise<Tag[]> {
    try {
      error.value = null
      return await api.getNoteTags(noteId)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to get note tags'
      return []
    }
  }

  async function searchTags(query: string): Promise<Tag[]> {
    try {
      error.value = null
      return await api.searchTags(query)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to search tags'
      return []
    }
  }

  async function cleanupUnusedTags(): Promise<number> {
    try {
      error.value = null
      const deletedCount = await api.cleanupUnusedTags()
      // Reload tags to get updated list
      await loadTags()
      return deletedCount
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to cleanup unused tags'
      return 0
    }
  }

  function setSelectedTag(tagName: string) {
    selectedTag.value = tagName
  }

  return {
    tags,
    selectedTag,
    loading,
    error,
    loadTags,
    createTag,
    deleteTag,
    renameTag,
    addTagToNote,
    removeTagFromNote,
    getNoteTags,
    searchTags,
    cleanupUnusedTags,
    setSelectedTag,
  }
})