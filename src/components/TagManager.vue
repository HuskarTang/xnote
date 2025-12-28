<template>
  <div class="tag-manager">
    <div class="tag-manager-header">
      <h3>Manage Tags</h3>
      <button @click="$emit('close')" class="close-btn">×</button>
    </div>
    
    <div class="tag-manager-content">
      <div class="current-tags">
        <div class="section-title">Current Tags</div>
        <div v-if="currentTags.length === 0" class="empty-state">
          No tags assigned
        </div>
        <div v-else class="tag-list">
          <div
            v-for="tag in currentTags"
            :key="tag.id"
            class="tag-item"
          >
            <span class="tag-color" :style="{ backgroundColor: tag.color || '#6c757d' }"></span>
            <span class="tag-name">{{ tag.name }}</span>
            <button
              @click="removeTag(tag.id)"
              class="remove-tag-btn"
              title="Remove tag"
            >
              ×
            </button>
          </div>
        </div>
      </div>
      
      <div class="add-tag-section">
        <div class="section-title">Add Tag</div>
        <div class="add-tag-input">
          <input
            v-model="newTagName"
            @keyup.enter="addTag"
            type="text"
            placeholder="Enter tag name..."
            class="tag-input"
          />
          <button
            @click="addTag"
            :disabled="!newTagName.trim()"
            class="add-tag-btn"
          >
            Add
          </button>
        </div>
        
        <div v-if="availableTags.length > 0" class="available-tags">
          <div class="section-subtitle">Or select from existing:</div>
          <div class="tag-suggestions">
            <button
              v-for="tag in availableTags"
              :key="tag.id"
              @click="addExistingTag(tag.name)"
              class="tag-suggestion"
            >
              <span class="tag-color" :style="{ backgroundColor: tag.color || '#6c757d' }"></span>
              {{ tag.name }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useTagsStore } from '@/stores/tags';
import type { Tag } from '@/types';

interface Props {
  noteId: string;
  currentTags: Tag[];
}

interface Emits {
  (e: 'close'): void;
  (e: 'tags-updated'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const tagsStore = useTagsStore();
const newTagName = ref('');

const availableTags = computed(() => {
  const currentTagIds = new Set(props.currentTags.map(tag => tag.id));
  return tagsStore.tags.filter(tag => !currentTagIds.has(tag.id));
});

async function addTag() {
  const tagName = newTagName.value.trim();
  if (!tagName) return;
  
  try {
    await tagsStore.addTagToNote(props.noteId, tagName);
    newTagName.value = '';
    emit('tags-updated');
  } catch (error) {
    console.error('Failed to add tag:', error);
  }
}

async function addExistingTag(tagName: string) {
  try {
    await tagsStore.addTagToNote(props.noteId, tagName);
    emit('tags-updated');
  } catch (error) {
    console.error('Failed to add tag:', error);
  }
}

async function removeTag(tagId: string) {
  try {
    await tagsStore.removeTagFromNote(props.noteId, tagId);
    emit('tags-updated');
  } catch (error) {
    console.error('Failed to remove tag:', error);
  }
}

onMounted(() => {
  // Load all tags to show suggestions
  tagsStore.loadTags();
});
</script>

<style lang="scss" scoped>
.tag-manager {
  position: absolute;
  top: 60px;
  right: 16px;
  width: 300px;
  background: white;
  border: 1px solid #e1e5e9;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 1000;
}

.tag-manager-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border-bottom: 1px solid #e1e5e9;
  
  h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #6c757d;
  
  &:hover {
    color: #2c3e50;
  }
}

.tag-manager-content {
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #2c3e50;
}

.section-subtitle {
  font-size: 12px;
  color: #6c757d;
  margin-bottom: 8px;
}

.current-tags {
  margin-bottom: 20px;
}

.empty-state {
  color: #6c757d;
  font-size: 14px;
  font-style: italic;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tag-item {
  display: flex;
  align-items: center;
  padding: 6px 8px;
  background: #f8f9fa;
  border-radius: 6px;
  gap: 8px;
}

.tag-color {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.tag-name {
  flex: 1;
  font-size: 14px;
}

.remove-tag-btn {
  background: none;
  border: none;
  color: #dc3545;
  cursor: pointer;
  font-size: 16px;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  
  &:hover {
    background: #dc3545;
    color: white;
  }
}

.add-tag-input {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.tag-input {
  flex: 1;
  padding: 6px 8px;
  border: 1px solid #e1e5e9;
  border-radius: 4px;
  font-size: 14px;
  
  &:focus {
    outline: none;
    border-color: #007acc;
  }
}

.add-tag-btn {
  padding: 6px 12px;
  background: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  
  &:hover:not(:disabled) {
    background: #005a9e;
  }
  
  &:disabled {
    background: #6c757d;
    cursor: not-allowed;
  }
}

.tag-suggestions {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag-suggestion {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  background: #f8f9fa;
  border: 1px solid #e1e5e9;
  border-radius: 12px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  
  &:hover {
    background: #e9ecef;
    border-color: #007acc;
  }
}
</style>