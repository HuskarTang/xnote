<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <h2 class="app-title">XNote</h2>
    </div>

    <div class="sidebar-content">
      <nav class="sidebar-nav">
        <div class="nav-section">
          <button
            v-for="view in navigationViews"
            :key="view.key"
            :class="['nav-item', { active: currentView === view.key }]"
            @click="setCurrentView(view.key)"
          >
            <span class="nav-icon">{{ view.icon }}</span>
            <span class="nav-label">{{ view.label }}</span>
            <span v-if="view.count !== undefined" class="nav-count">
              {{ view.count }}
            </span>
          </button>
        </div>

        <div class="nav-section" v-if="sortedTags.length > 0">
          <div class="section-header">
            <span>Tags</span>
          </div>
          <button
            v-for="tag in sortedTags"
            :key="tag.id"
            :class="['nav-item', 'tag-item', { active: selectedTag === tag.name }]"
            @click="selectTag(tag.name)"
          >
            <span class="tag-color" :style="{ backgroundColor: tag.color || '#6c757d' }"></span>
            <span class="nav-label">{{ tag.name }}</span>
            <span class="nav-count">{{ getTagCount(tag.name) }}</span>
          </button>
        </div>
      </nav>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useNotesStore } from '@/stores/notes';
import { useTagsStore } from '@/stores/tags';
import type { SidebarView } from '@/types';

const notesStore = useNotesStore();
const tagsStore = useTagsStore();

const currentView = ref<SidebarView>('all');
const selectedTag = ref<string | null>(null);

const navigationViews = computed(() => [
  {
    key: 'all' as SidebarView,
    label: 'All Notes',
    icon: '📝',
    count: notesStore.notes.filter(n => !n.note.is_trashed).length,
  },
  {
    key: 'favorites' as SidebarView,
    label: 'Favorites',
    icon: '⭐',
    count: notesStore.favoriteNotes.length,
  },
  {
    key: 'untagged' as SidebarView,
    label: 'Untagged',
    icon: '🏷️',
    count: notesStore.untaggedNotes.length,
  },
  {
    key: 'trash' as SidebarView,
    label: 'Trash',
    icon: '🗑️',
    count: notesStore.trashedNotes.length,
  },
]);

const sortedTags = computed(() => tagsStore.sortedTags);

function setCurrentView(view: SidebarView) {
  currentView.value = view;
  selectedTag.value = null;
  
  // Update notes filter based on view
  switch (view) {
    case 'all':
      notesStore.setTagFilter(null);
      break;
    case 'favorites':
      // This will be handled in the notes store
      break;
    case 'untagged':
      // This will be handled in the notes store
      break;
    case 'trash':
      // This will be handled in the notes store
      break;
  }
  
  // Emit event to parent or use a store to communicate the view change
  emitViewChange(view);
}

function selectTag(tagName: string) {
  currentView.value = 'tags';
  selectedTag.value = tagName;
  notesStore.setTagFilter(tagName);
  
  emitViewChange('tags', tagName);
}

function getTagCount(tagName: string): number {
  return notesStore.notes.filter(noteWithTags => 
    noteWithTags.tags.some(tag => tag.name === tagName) && 
    !noteWithTags.note.is_trashed
  ).length;
}

function emitViewChange(view: SidebarView, tagName?: string) {
  // This could be implemented with a global event bus or store
  // For now, we'll use the notes store to handle filtering
  console.log('View changed:', view, tagName);
}

onMounted(() => {
  // Set initial view
  setCurrentView('all');
});
</script>

<style lang="scss" scoped>
.sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #2c3e50;
  color: white;
}

.sidebar-header {
  padding: 20px;
  border-bottom: 1px solid #34495e;
  
  .app-title {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: white;
  }
}

.sidebar-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px 0;
}

.nav-section {
  margin-bottom: 20px;
  
  &:last-child {
    margin-bottom: 0;
  }
}

.section-header {
  padding: 8px 20px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  color: #bdc3c7;
  letter-spacing: 0.5px;
}

.nav-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 12px 20px;
  border: none;
  background: transparent;
  color: #ecf0f1;
  font-size: 0.9rem;
  cursor: pointer;
  transition: all 0.2s ease;
  
  &:hover {
    background: #34495e;
  }
  
  &.active {
    background: #3498db;
    color: white;
  }
}

.nav-icon {
  margin-right: 12px;
  font-size: 1rem;
}

.nav-label {
  flex: 1;
  text-align: left;
}

.nav-count {
  font-size: 0.75rem;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
}

.tag-item {
  .nav-icon {
    display: none;
  }
}

.tag-color {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 12px;
  flex-shrink: 0;
}
</style>