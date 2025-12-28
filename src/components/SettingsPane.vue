<template>
  <div class="settings-pane">
    <div class="settings-header">
      <h2>系统设置</h2>
      <button class="close-btn" @click="$emit('close')">×</button>
    </div>
    
    <div class="settings-content">
      <!-- Git同步设置 -->
      <div class="settings-section">
        <h3>Git同步设置</h3>
        
        <div class="setting-item">
          <label class="setting-label">
            <input 
              type="checkbox" 
              v-model="gitConfig.enabled"
              @change="onConfigChange"
            />
            启用Git同步
          </label>
        </div>
        
        <div v-if="gitConfig.enabled" class="git-config">
          <div class="setting-item">
            <label>Git仓库地址</label>
            <input 
              type="text" 
              v-model="gitConfig.repository_url"
              placeholder="https://github.com/username/repo.git"
              @input="onConfigChange"
            />
          </div>
          
          <div class="setting-item">
            <label>分支</label>
            <input 
              type="text" 
              v-model="gitConfig.branch"
              placeholder="main"
              @input="onConfigChange"
            />
          </div>
          
          <div class="setting-item">
            <label>认证方式</label>
            <select v-model="gitConfig.auth_type" @change="onConfigChange">
              <option value="none">无认证</option>
              <option value="basic">用户名/密码</option>
              <option value="ssh">SSH密钥</option>
            </select>
          </div>
          
          <!-- 用户名/密码认证 -->
          <div v-if="gitConfig.auth_type === 'basic'" class="auth-config">
            <div class="setting-item">
              <label>用户名</label>
              <input 
                type="text" 
                v-model="gitConfig.username"
                @input="onConfigChange"
              />
            </div>
            <div class="setting-item">
              <label>密码/Token</label>
              <input 
                type="password" 
                v-model="gitConfig.password"
                @input="onConfigChange"
              />
            </div>
          </div>
          
          <!-- SSH认证 -->
          <div v-if="gitConfig.auth_type === 'ssh'" class="auth-config">
            <div class="setting-item">
              <label>SSH密钥路径</label>
              <div class="file-input">
                <input 
                  type="text" 
                  v-model="gitConfig.ssh_key_path"
                  placeholder="~/.ssh/id_rsa"
                  @input="onConfigChange"
                />
                <button @click="selectSSHKey">选择文件</button>
              </div>
            </div>
          </div>
          
          <div class="setting-actions">
            <button 
              class="test-btn" 
              @click="testConnection"
              :disabled="isTestingConnection"
            >
              {{ isTestingConnection ? '测试中...' : '测试连接' }}
            </button>
          </div>
        </div>
      </div>
      
      <!-- 其他设置可以在这里添加 -->
    </div>
    
    <div class="settings-footer">
      <button class="save-btn" @click="saveSettings" :disabled="isSaving">
        {{ isSaving ? '保存中...' : '保存设置' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { GitSyncConfig } from '@/types'
import { api } from '@/utils/api'

const emit = defineEmits<{
  close: []
  saved: []
}>()

// Git配置
const gitConfig = ref<GitSyncConfig>({
  enabled: false,
  repository_url: '',
  branch: 'main',
  auth_type: 'none',
  username: '',
  password: '',
  ssh_key_path: ''
})

const isTestingConnection = ref(false)
const isSaving = ref(false)

// 加载设置
const loadSettings = async () => {
  try {
    const config = await api.getGitSyncConfig()
    if (config) {
      gitConfig.value = { ...gitConfig.value, ...config }
    }
  } catch (error) {
    console.error('Failed to load git sync config:', error)
  }
}

// 配置变更
const onConfigChange = () => {
  // 可以在这里添加实时验证逻辑
}

// 选择SSH密钥文件
const selectSSHKey = async () => {
  try {
    const result = await api.showDirectoryDialog()
    if (result) {
      gitConfig.value.ssh_key_path = result
    }
  } catch (error) {
    console.error('Failed to select SSH key:', error)
  }
}

// 测试连接
const testConnection = async () => {
  if (!gitConfig.value.repository_url) {
    alert('请先填写Git仓库地址')
    return
  }
  
  isTestingConnection.value = true
  try {
    // 暂时简化测试连接逻辑
    alert('连接测试功能开发中...')
  } catch (error) {
    console.error('Connection test failed:', error)
    alert('连接测试失败，请检查配置')
  } finally {
    isTestingConnection.value = false
  }
}

// 保存设置
const saveSettings = async () => {
  isSaving.value = true
  try {
    await api.updateGitSyncConfig(gitConfig.value)
    alert('设置保存成功！')
    emit('saved')
    emit('close')
  } catch (error) {
    console.error('Failed to save settings:', error)
    alert('保存设置失败，请重试')
  } finally {
    isSaving.value = false
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
.settings-pane {
  position: fixed;
  top: 0;
  right: 0;
  width: 500px;
  height: 100vh;
  background: white;
  box-shadow: -2px 0 10px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  display: flex;
  flex-direction: column;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e5e5e5;
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: #333;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.settings-section {
  margin-bottom: 30px;
}

.settings-section h3 {
  margin: 0 0 20px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  cursor: pointer;
}

.setting-item label {
  display: block;
  margin-bottom: 6px;
  font-weight: 500;
  color: #333;
}

.setting-item input,
.setting-item select {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.setting-item input:focus,
.setting-item select:focus {
  outline: none;
  border-color: #007acc;
}

.git-config {
  margin-left: 20px;
  padding-left: 20px;
  border-left: 2px solid #e5e5e5;
}

.auth-config {
  margin-top: 16px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

.file-input {
  display: flex;
  gap: 8px;
}

.file-input input {
  flex: 1;
}

.file-input button {
  padding: 8px 16px;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.file-input button:hover {
  background: #e9e9e9;
}

.setting-actions {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #e5e5e5;
}

.test-btn {
  padding: 8px 16px;
  background: #007acc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.test-btn:hover:not(:disabled) {
  background: #005a9e;
}

.test-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}


.settings-footer {
  padding: 20px;
  border-top: 1px solid #e5e5e5;
  background: #f8f9fa;
}

.save-btn {
  width: 100%;
  padding: 12px;
  background: #28a745;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  font-weight: 500;
}

.save-btn:hover:not(:disabled) {
  background: #218838;
}

.save-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}
</style>