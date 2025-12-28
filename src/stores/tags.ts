import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Tag } from '@/types'
import { api } from '@/utils/api'

export const useTagsStore = defineStore('tags', () => {
  const tags = ref<Tag[]>([])
  const selectedTag = ref<string>('All Notes')
  const loading = ref(false)
  const error = ref<string | null>(null)

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

  function setSelectedTag(tagName: string) {
    selectedTag.value = tagName
  }

  return {
    tags,
    selectedTag,
    loading,
    error,
    loadTags,
    setSelectedTag,
  }
})