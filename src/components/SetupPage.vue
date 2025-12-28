<template>
  <div class="setup-page">
    <div class="setup-container">
      <div class="setup-header">
        <h1>Welcome to XNote</h1>
        <p>Let's get started by selecting your data directory</p>
      </div>
      
      <div class="setup-content">
        <div class="directory-selection">
          <label for="data-directory">Data Directory</label>
          <div class="directory-input">
            <input 
              id="data-directory"
              v-model="selectedDirectory"
              type="text" 
              placeholder="Select a directory for your notes"
              readonly
            >
            <button @click="selectDirectory">Browse</button>
          </div>
          <p class="directory-description">
            This is where your notes and attachments will be stored. You can change this later in settings.
          </p>
        </div>
        
        <div class="setup-actions">
          <button 
            class="setup-button"
            :disabled="!selectedDirectory"
            @click="confirmSetup"
          >
            Continue
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/utils/api'

const selectedDirectory = ref('')

const selectDirectory = async () => {
  try {
    const directory = await api.showDirectoryDialog()
    if (directory) {
      selectedDirectory.value = directory
    }
  } catch (error) {
    console.error('Failed to select directory:', error)
  }
}

const confirmSetup = async () => {
  if (!selectedDirectory.value) return
  
  try {
    // Update the data directory
    await api.updateDataDirectory(selectedDirectory.value)
    
    // Reinitialize the data directory
    await api.reinitializeDataDirectory(selectedDirectory.value)
    
    // Mark setup as complete
    await api.markSetupComplete()
    
    // Reload the app or navigate to the main page
    window.location.reload()
  } catch (error) {
    console.error('Failed to setup data directory:', error)
  }
}
</script>

<style scoped>
.setup-page {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  background-color: #f5f5f5;
  padding: 20px;
}

.setup-container {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  padding: 40px;
  max-width: 500px;
  width: 100%;
}

.setup-header {
  text-align: center;
  margin-bottom: 30px;
}

.setup-header h1 {
  font-size: 24px;
  font-weight: 600;
  margin-bottom: 10px;
  color: #333;
}

.setup-header p {
  font-size: 16px;
  color: #666;
}

.setup-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.directory-selection label {
  display: block;
  font-weight: 500;
  margin-bottom: 8px;
  color: #333;
}

.directory-input {
  display: flex;
  gap: 10px;
}

.directory-input input {
  flex: 1;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.directory-input button {
  padding: 10px 20px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  transition: background-color 0.2s;
}

.directory-input button:hover {
  background-color: #0066b3;
}

.directory-description {
  font-size: 12px;
  color: #999;
  margin-top: 5px;
}

.setup-actions {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}

.setup-button {
  padding: 12px 30px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.setup-button:hover:not(:disabled) {
  background-color: #0066b3;
}

.setup-button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}
</style>