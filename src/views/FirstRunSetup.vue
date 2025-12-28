<template>
  <div class="setup-container">
    <div class="setup-card">
      <div class="setup-header">
        <h1>Welcome to XNote</h1>
        <p>Let's set up your note storage location</p>
      </div>

      <div class="setup-content">
        <div class="form-group">
          <label for="data-directory">Data Directory</label>
          <div class="directory-input">
            <input
              id="data-directory"
              v-model="selectedPath"
              type="text"
              placeholder="Choose where to store your notes..."
              readonly
              class="directory-path"
            />
            <button @click="selectDirectory" class="browse-button">
              Browse
            </button>
          </div>
          <p class="help-text">
            This directory will contain all your notes and attachments. 
            You can change this later in settings.
          </p>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="setup-actions">
          <button
            @click="confirmSetup"
            :disabled="!selectedPath || isLoading"
            class="confirm-button"
          >
            <span v-if="isLoading">Setting up...</span>
            <span v-else>Continue</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/api/dialog';
import { useAppStore } from '@/stores/app';

const appStore = useAppStore();

const selectedPath = ref('');
const isLoading = ref(false);
const error = ref('');

async function selectDirectory() {
  try {
    const result = await open({
      directory: true,
      multiple: false,
      title: 'Select Notes Directory',
    });

    if (result && typeof result === 'string') {
      selectedPath.value = result;
      error.value = '';
    }
  } catch (err) {
    error.value = 'Failed to select directory';
    console.error('Directory selection error:', err);
  }
}

async function confirmSetup() {
  if (!selectedPath.value) {
    error.value = 'Please select a directory';
    return;
  }

  isLoading.value = true;
  error.value = '';

  try {
    await appStore.setDataDirectory(selectedPath.value);
  } catch (err) {
    error.value = err as string || 'Failed to set up data directory';
  } finally {
    isLoading.value = false;
  }
}
</script>

<style lang="scss" scoped>
.setup-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 20px;
}

.setup-card {
  background: white;
  border-radius: 12px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
  max-width: 500px;
  width: 100%;
  overflow: hidden;
}

.setup-header {
  background: #f8f9fa;
  padding: 40px 40px 30px;
  text-align: center;
  border-bottom: 1px solid #e9ecef;

  h1 {
    margin: 0 0 10px;
    color: #2c3e50;
    font-size: 28px;
    font-weight: 600;
  }

  p {
    margin: 0;
    color: #6c757d;
    font-size: 16px;
  }
}

.setup-content {
  padding: 40px;
}

.form-group {
  margin-bottom: 24px;

  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #2c3e50;
  }
}

.directory-input {
  display: flex;
  gap: 8px;
}

.directory-path {
  flex: 1;
  padding: 12px 16px;
  border: 2px solid #e9ecef;
  border-radius: 8px;
  font-size: 14px;
  background: #f8f9fa;
  color: #495057;

  &:focus {
    outline: none;
    border-color: #667eea;
  }
}

.browse-button {
  padding: 12px 20px;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;

  &:hover {
    background: #5a6fd8;
  }
}

.help-text {
  margin-top: 8px;
  font-size: 13px;
  color: #6c757d;
  line-height: 1.4;
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 14px;
}

.setup-actions {
  text-align: center;
}

.confirm-button {
  background: #28a745;
  color: white;
  border: none;
  padding: 14px 32px;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;

  &:hover:not(:disabled) {
    background: #218838;
  }

  &:disabled {
    background: #6c757d;
    cursor: not-allowed;
  }
}
</style>