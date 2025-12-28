import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ViewMode } from '@/types'

export const useAppStore = defineStore('app', () => {
  const viewMode = ref<ViewMode>('view')
  const searchQuery = ref('')
  const sidebarWidth = ref(240)
  const notePaneWidth = ref(320)

  function setViewMode(mode: ViewMode) {
    viewMode.value = mode
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function setSidebarWidth(width: number) {
    sidebarWidth.value = width
  }

  function setNotePaneWidth(width: number) {
    notePaneWidth.value = width
  }

  return {
    viewMode,
    searchQuery,
    sidebarWidth,
    notePaneWidth,
    setViewMode,
    setSearchQuery,
    setSidebarWidth,
    setNotePaneWidth,
  }
})