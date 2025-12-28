<template>
  <div class="note-list">
    <div class="note-list-header">
      <div class="search-container">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search notes..."
          class="search-input"
        />
        <button class="search-icon">🔍</button>
      </div>
      
      <button @click="createNewNote" class="new-note-btn">
        <span class="btn-icon">+</span>
        New Note
      </button>
    </div>

    <div class="note-list-content">
      <div v-if="isLoading" class="loading-state">
        Loading notes...
      </div>
      
      <div v-else-if="displayedNotes.length === 0" class="empty-state">
        <div class="empty-icon">📝</div>
        <p>No notes found</p>
        <button @click="createNewNote" class="btn btn-primary">
          Create your first note
        </button>
      </div>
      
      <div v-else class="notes-container">
        <div
          v-for="noteWithTags in displayedNotes"
          :key="noteWithTags.note.id"
          :class="['note-item', { active: isNoteSelected(noteWithTags.note.id) }]"
          @click="selectNote(noteWithTags.note.id)"
        >
          <div class="note-header">
            <h3 class="note-title">{{ noteWithTags.note.title }}</h3>
            <div class="note-meta">
              <span v-if="noteWithTags.note.is_favorite" class="favorite-icon">⭐</span>
              <span class="note-date">{{ formatDate(noteWithTags.note.modified_at) }}</span>
            </div>
          </div>
          
          <div v-if="noteWithTags.tags.length > 0" class="note-tags">
            <span
              v-for="tag in noteWithTags.tags.slice(0, 3)"
              :key="tag.id"
              class="note-tag"
              :style="{ backgroundColor: tag.color || '#6c757d' }"
            >
              {{ tag.name }}
            </span>
            <span v-if="noteWithTags.tags.length > 3" class="more-tags">
              +{{ noteWithTags.tags.length - 3 }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useNotesStore } from '@/stores/notes';
import { formatDistanceToNow } from 'date-fns';

const notesStore = useNotesStore();

const searchQuery = ref('');
const selectedNoteId = ref<string | null>(null);

const isLoading = computed(() => notesStore.isLoading);

const displayedNotes = computed(() => {
  let notes = notesStore.filteredNotes;
  
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    notes = notes.filter(noteWithTags =>
      noteWithTags.note.title.toLowerCase().includes(query) ||
      noteWithTags.tags.some(tag => tag.name.toLowerCase().includes(query))
    );
  }
  
  return notes;
});

function createNewNote() {
  notesStore.createNote({
    title: 'New Note',
    content: '# New Note\n\nStart writing...',
  }).then(newNote => {
    selectedNoteId.value = newNote.id;
  });
}

function selectNote(noteId: string) {
  selectedNoteId.value = noteId;
  notesStore.loadNoteContent(noteId);
}

function isNoteSelected(noteId: string): boolean {
  return selectedNoteId.value === noteId;
}

function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    return formatDistanceToNow(date, { addSuffix: true });
  } catch {
    return 'Unknown';
  }
}

// Watch for search query changes
watch(searchQuery, (newQuery) => {
  notesStore.setSearchQuery(newQuery);
});

// Auto-select first note if none selected
watch(() => displayedNotes.value, (notes) => {
  if (notes.length > 0 && !selectedNoteId.value) {
    selectNote(notes[0].note.id);
  }
}, { immediate: true });
</script>

<style lang="scss" scoped>
.note-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: white;
}

.note-list-header {
  padding: 16px;
  border-bottom: 1px solid #e1e5e9;
  background: #f8f9fa;
}

.search-container {
  position: relative;
  margin-bottom: 12px;
}

.search-input {
  width: 100%;
  padding: 8px 12px 8px 36px;
  border: 1px solid #e1e5e9;
  border-radius: 6px;
  font-size: 14px;
  
  &:focus {
    outline: none;
    border-color: #007acc;
    box-shadow: 0 0 0 2px rgba(0, 122, 204, 0.2);
  }
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  color: #6c757d;
  cursor: pointer;
}

.new-note-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 10px 16px;
  background: #007acc;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
  
  &:hover {
    background: #005a9e;
  }
}

.btn-icon {
  margin-right: 8px;
  font-size: 16px;
}

.note-list-content {
  flex: 1;
  overflow-y: auto;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #6c757d;
  text-align: center;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 16px;
}

.notes-container {
  padding: 8px 0;
}

.note-item {
  padding: 16px;
  border-bottom: 1px solid #f1f3f4;
  cursor: pointer;
  transition: background 0.2s;
  
  &:hover {
    background: #f8f9fa;
  }
  
  &.active {
    background: #e3f2fd;
    border-right: 3px solid #007acc;
  }
  
  &:last-child {
    border-bottom: none;
  }
}

.note-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.note-title {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
  color: #2c3e50;
  line-height: 1.3;
  flex: 1;
  margin-right: 8px;
  
  // Truncate long titles
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.favorite-icon {
  font-size: 14px;
}

.note-date {
  font-size: 12px;
  color: #6c757d;
  white-space: nowrap;
}

.note-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.note-tag {
  display: inline-block;
  padding: 2px 6px;
  background: #6c757d;
  color: white;
  font-size: 11px;
  border-radius: 10px;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.more-tags {
  font-size: 11px;
  color: #6c757d;
  font-weight: 500;
}
</style>