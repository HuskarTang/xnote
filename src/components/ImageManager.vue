<template>
  <div v-if="visible" class="image-manager-overlay" @click.self="closeDialog">
    <div class="image-manager-dialog">
      <div class="dialog-header">
        <h3>添加图片</h3>
        <button class="close-button" @click="closeDialog">×</button>
      </div>
      
      <div class="dialog-content">
        <div class="tab-buttons">
          <button 
            class="tab-button" 
            :class="{ active: activeTab === 'url' }"
            @click="activeTab = 'url'"
          >
            图片地址
          </button>
          <button 
            class="tab-button" 
            :class="{ active: activeTab === 'upload' }"
            @click="activeTab = 'upload'"
          >
            本地上传
          </button>
        </div>

        <!-- URL Tab -->
        <div v-if="activeTab === 'url'" class="tab-content">
          <div class="form-group">
            <label for="imageUrl">图片地址</label>
            <input
              id="imageUrl"
              v-model="imageUrl"
              type="url"
              placeholder="http://"
              class="form-input"
            />
          </div>
          
          <div class="form-group">
            <label for="imageAlt">图片描述</label>
            <input
              id="imageAlt"
              v-model="imageAlt"
              type="text"
              placeholder="图片描述"
              class="form-input"
            />
          </div>
          
          <div class="form-group">
            <label for="imageTitle">图片链接</label>
            <input
              id="imageTitle"
              v-model="imageTitle"
              type="url"
              placeholder="http://"
              class="form-input"
            />
          </div>
        </div>

        <!-- Upload Tab -->
        <div v-if="activeTab === 'upload'" class="tab-content">
          <div class="upload-area" @drop="handleDrop" @dragover.prevent @dragenter.prevent>
            <div v-if="!selectedFile" class="upload-placeholder">
              <div class="upload-icon">
                <Icons name="folder" :size="48" />
              </div>
              <p>拖拽文件到这里，或点击选择文件</p>
              <input
                ref="fileInput"
                type="file"
                accept="image/*"
                style="display: none"
                @change="handleFileSelect"
              />
              <button class="upload-button" @click="triggerFileSelect">选择文件</button>
            </div>
            
            <div v-else class="file-preview">
              <div class="preview-info">
                <span class="file-name">{{ selectedFile.name }}</span>
                <span class="file-size">({{ formatFileSize(selectedFile.size) }})</span>
                <button class="remove-button" @click="removeFile">×</button>
              </div>
              <img v-if="previewUrl" :src="previewUrl" class="preview-image" />
            </div>
          </div>
          
          <div class="form-group">
            <label for="uploadAlt">图片描述</label>
            <input
              id="uploadAlt"
              v-model="uploadAlt"
              type="text"
              placeholder="图片描述"
              class="form-input"
            />
          </div>
        </div>
      </div>
      
      <div class="dialog-actions">
        <button class="cancel-button" @click="closeDialog">取消</button>
        <button class="confirm-button" @click="confirmInsert" :disabled="!canInsert">确定</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { ElMessage } from 'element-plus'
import Icons from '@/components/Icons.vue'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  close: []
  insert: [imageMarkdown: string]
}>()

// State
const activeTab = ref<'url' | 'upload'>('url')
const imageUrl = ref('')
const imageAlt = ref('')
const imageTitle = ref('')
const uploadAlt = ref('')
const selectedFile = ref<File | null>(null)
const previewUrl = ref('')
const fileInput = ref<HTMLInputElement>()

// Computed
const canInsert = computed(() => {
  if (activeTab.value === 'url') {
    return imageUrl.value.trim() !== ''
  } else {
    return selectedFile.value !== null
  }
})

// Methods
const closeDialog = () => {
  // Reset form
  imageUrl.value = ''
  imageAlt.value = ''
  imageTitle.value = ''
  uploadAlt.value = ''
  removeFile()
  activeTab.value = 'url'
  emit('close')
}

const triggerFileSelect = () => {
  fileInput.value?.click()
}

const handleFileSelect = (event: Event) => {
  const files = (event.target as HTMLInputElement).files
  if (files && files[0]) {
    setSelectedFile(files[0])
  }
}

const handleDrop = (event: DragEvent) => {
  event.preventDefault()
  const files = event.dataTransfer?.files
  if (files && files[0]) {
    setSelectedFile(files[0])
  }
}

const setSelectedFile = (file: File) => {
  selectedFile.value = file
  
  // Create preview URL
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
  }
  previewUrl.value = URL.createObjectURL(file)
  
  // Auto-set alt text from filename
  if (!uploadAlt.value) {
    uploadAlt.value = file.name.replace(/\.[^/.]+$/, "") // Remove extension
  }
}

const removeFile = () => {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
  }
  selectedFile.value = null
  previewUrl.value = ''
  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const confirmInsert = async () => {
  try {
    if (activeTab.value === 'url') {
      // URL mode
      const alt = imageAlt.value || 'image'
      const url = imageUrl.value
      const title = imageTitle.value
      
      let markdown: string
      if (title) {
        markdown = `![${alt}](${url} "${title}")`
      } else {
        markdown = `![${alt}](${url})`
      }
      
      emit('insert', markdown)
    } else {
      // Upload mode
      if (!selectedFile.value) return
      
      const alt = uploadAlt.value || 'image'
      
      // Save file and get path
      const savedPath = await saveImageToAttachments(selectedFile.value)
      
      // Create markdown with relative path
      const markdown = `![${alt}](${savedPath})`
      
      emit('insert', markdown)
    }
    
    closeDialog()
  } catch (error) {
    console.error('Failed to insert image:', error)
    ElMessage.error('插入图片失败')
  }
}

const saveImageToAttachments = async (file: File): Promise<string> => {
  // Convert file to base64
  const arrayBuffer = await file.arrayBuffer()
  const uint8Array = new Uint8Array(arrayBuffer)
  
  // Generate filename
  const now = new Date()
  const timestamp = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, '0')}${String(now.getDate()).padStart(2, '0')}${String(now.getHours()).padStart(2, '0')}${String(now.getMinutes()).padStart(2, '0')}${String(now.getSeconds()).padStart(2, '0')}`
  const extension = file.name.split('.').pop() || 'png'
  const filename = `Upload_${timestamp}.${extension}`
  
  // Call Tauri command to save file to images directory for text content
  const savedPath = await invoke<string>('save_image_for_text', {
    filename,
    data: Array.from(uint8Array)
  })
  
  return savedPath
}

// Cleanup on unmount
watch(() => props.visible, (newVisible) => {
  if (!newVisible) {
    removeFile()
  }
})
</script>

<style scoped>
.image-manager-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.image-manager-dialog {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e5e5e5;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-button {
  width: 32px;
  height: 32px;
  border: none;
  background: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
  border-radius: 4px;
}

.close-button:hover {
  background-color: #f0f0f0;
}

.dialog-content {
  padding: 24px;
}

.tab-buttons {
  display: flex;
  gap: 4px;
  margin-bottom: 24px;
  border-bottom: 1px solid #e5e5e5;
}

.tab-button {
  padding: 8px 16px;
  border: none;
  background: none;
  cursor: pointer;
  color: #666;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-button.active {
  color: #007acc;
  border-bottom-color: #007acc;
}

.tab-button:hover {
  background-color: #f0f0f0;
}

.tab-content {
  min-height: 200px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 4px;
  font-weight: 500;
  color: #333;
}

.form-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.form-input:focus {
  outline: none;
  border-color: #007acc;
}

.upload-area {
  border: 2px dashed #d1d5db;
  border-radius: 8px;
  padding: 40px 20px;
  text-align: center;
  transition: border-color 0.2s;
  margin-bottom: 16px;
}

.upload-area:hover {
  border-color: #007acc;
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.upload-icon {
  font-size: 48px;
  opacity: 0.5;
}

.upload-placeholder p {
  margin: 0;
  color: #666;
}

.upload-button {
  padding: 8px 16px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.upload-button:hover {
  background-color: #005fa3;
}

.file-preview {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.preview-info {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.file-name {
  font-weight: 500;
}

.file-size {
  color: #666;
}

.remove-button {
  width: 20px;
  height: 20px;
  border: none;
  background-color: #ef4444;
  color: white;
  border-radius: 50%;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.remove-button:hover {
  background-color: #dc2626;
}

.preview-image {
  max-width: 200px;
  max-height: 200px;
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 24px;
  border-top: 1px solid #e5e5e5;
}

.cancel-button {
  padding: 8px 16px;
  border: 1px solid #d1d5db;
  background: white;
  color: #374151;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-button:hover {
  background-color: #f9fafb;
}

.confirm-button {
  padding: 8px 16px;
  background-color: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.confirm-button:hover:not(:disabled) {
  background-color: #005fa3;
}

.confirm-button:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}
</style>