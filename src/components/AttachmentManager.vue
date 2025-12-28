<template>
  <div v-if="visible" class="attachment-manager-overlay" @click.self="closeDialog">
    <div class="attachment-manager-dialog">
      <div class="dialog-header">
        <h3>附件管理</h3>
        <button class="close-button" @click="closeDialog">×</button>
      </div>
      
      <div class="dialog-content">
        <!-- 添加附件按钮 -->
        <div class="add-attachment-section">
          <button class="add-attachment-button" @click="selectFiles">
            <Icons name="attachment" :size="16" class="icon" />
            选择文件添加附件
          </button>
          <input
            ref="fileInput"
            type="file"
            multiple
            @change="handleFileSelect"
            style="display: none;"
          />
        </div>

        <!-- 附件列表 -->
        <div class="attachment-list">
          <h4>已添加的附件 ({{ attachments.length }})</h4>
          
          <div v-if="attachments.length === 0" class="no-attachments">
            暂无附件
          </div>
          
          <div v-else class="attachment-items">
            <div
              v-for="attachment in attachments"
              :key="attachment"
              class="attachment-item"
            >
              <div class="attachment-info">
                <Icons name="file" :size="16" class="attachment-icon" />
                <span class="attachment-name">{{ getFileName(attachment) }}</span>
              </div>
              <div class="attachment-actions">
                <button
                  class="action-button open-button"
                  @click="openAttachment(attachment)"
                  title="打开附件"
                >
                  <Icons name="open" :size="16" />
                </button>
                <button
                  class="action-button delete-button"
                  @click="deleteAttachment(attachment)"
                  title="删除附件"
                >
                  <Icons name="delete" :size="16" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <div class="dialog-footer">
        <button class="secondary-button" @click="closeDialog">关闭</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { ElMessage } from 'element-plus'
import Icons from '@/components/Icons.vue'

const props = defineProps<{
  visible: boolean
  noteId?: string
}>()

const emit = defineEmits<{
  close: []
}>()

const fileInput = ref<HTMLInputElement>()
const attachments = ref<string[]>([])
const loading = ref(false)

// 监听对话框打开，加载附件列表
watch(() => props.visible, (visible) => {
  if (visible) {
    loadAttachments()
  }
})

// 加载附件列表
async function loadAttachments() {
  if (!props.noteId) {
    attachments.value = []
    return
  }
  
  try {
    loading.value = true
    const result = await invoke<string[]>('list_attachments', {
      noteId: props.noteId
    })
    attachments.value = result
  } catch (error) {
    console.error('Failed to load attachments:', error)
  } finally {
    loading.value = false
  }
}

// 选择文件
async function selectFiles() {
  try {
    const selected = await open({
      multiple: true,
      title: '选择附件文件'
    })
    
    if (selected && Array.isArray(selected)) {
      for (const filePath of selected) {
        await addAttachment(filePath)
      }
    } else if (selected && typeof selected === 'string') {
      await addAttachment(selected)
    }
  } catch (error) {
    console.error('Failed to select files:', error)
  }
}

// 处理文件输入变化
async function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files) {
    for (const file of Array.from(target.files)) {
      // 这里需要将File对象转换为路径，但在Web环境中比较复杂
      // 建议使用open API替代
    }
  }
}

// 添加附件
async function addAttachment(filePath: string) {
  if (!props.noteId) {
    ElMessage.error('无法添加附件：未指定笔记ID')
    return
  }
  
  try {
    const result = await invoke<string>('add_attachment', {
      noteId: props.noteId,
      filePath
    })
    
    console.log('Attachment added:', result)
    
    // 重新加载附件列表
    await loadAttachments()
  } catch (error) {
    console.error('Failed to add attachment:', error)
    ElMessage.error(`添加附件失败: ${error}`)
  }
}

// 删除附件
async function deleteAttachment(attachment: string) {
  if (!props.noteId) {
    ElMessage.error('无法删除附件：未指定笔记ID')
    return
  }
  
  if (!confirm(`确定要删除附件 "${getFileName(attachment)}" 吗？`)) {
    return
  }
  
  try {
    const result = await invoke<boolean>('delete_attachment', {
      noteId: props.noteId,
      relativePath: attachment
    })
    
    if (result) {
      console.log('Attachment deleted:', attachment)
      // 重新加载附件列表
      await loadAttachments()
    } else {
      ElMessage.warning('附件不存在或已被删除')
    }
  } catch (error) {
    console.error('Failed to delete attachment:', error)
    ElMessage.error(`删除附件失败: ${error}`)
  }
}

// 打开附件
async function openAttachment(attachment: string) {
  try {
    await invoke('open_attachment', {
      relativePath: attachment
    })
  } catch (error) {
    console.error('Failed to open attachment:', error)
    ElMessage.error(`打开附件失败: ${error}`)
  }
}

// 获取文件名
function getFileName(attachment: string): string {
  return attachment.split('/').pop() || attachment
}

// 关闭对话框
function closeDialog() {
  emit('close')
}
</script>

<style scoped>
.attachment-manager-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.attachment-manager-dialog {
  background: white;
  border-radius: 8px;
  width: 600px;
  max-width: 90vw;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #eee;
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  color: #333;
}

.close-button {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #999;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-button:hover {
  color: #333;
}

.dialog-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.add-attachment-section {
  margin-bottom: 20px;
}

.add-attachment-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: #007bff;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background-color 0.2s;
}

.add-attachment-button:hover {
  background: #0056b3;
}

.add-attachment-button .icon {
  font-size: 16px;
}

.attachment-list h4 {
  margin: 0 0 15px 0;
  font-size: 16px;
  color: #333;
}

.no-attachments {
  text-align: center;
  color: #999;
  padding: 40px 20px;
  font-style: italic;
}

.attachment-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.attachment-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.attachment-item:hover {
  background: #e9ecef;
}

.attachment-info {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.attachment-icon {
  font-size: 16px;
}

.attachment-name {
  font-size: 14px;
  color: #333;
  word-break: break-all;
}

.attachment-actions {
  display: flex;
  gap: 8px;
}

.action-button {
  background: none;
  border: 1px solid #ddd;
  border-radius: 4px;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.open-button:hover {
  background: #007bff;
  border-color: #007bff;
  color: white;
}

.delete-button:hover {
  background: #dc3545;
  border-color: #dc3545;
  color: white;
}

.dialog-footer {
  padding: 20px;
  border-top: 1px solid #eee;
  display: flex;
  justify-content: flex-end;
}

.secondary-button {
  padding: 8px 16px;
  background: #6c757d;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.secondary-button:hover {
  background: #545b62;
}
</style>