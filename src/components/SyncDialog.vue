<template>
  <div class="sync-dialog-overlay" @click="closeDialog">
    <div class="sync-dialog" @click.stop>
      <div class="sync-header">
        <h3>Git同步管理</h3>
        <div class="repo-info">
          <span>{{ gitConfig?.repository_url || '未配置' }}</span>
          <span class="branch">{{ gitConfig?.branch || 'main' }}</span>
        </div>
        <button class="close-btn" @click="closeDialog">×</button>
      </div>
      
      <div class="sync-content">
        <!-- 使用 Element Plus 表格布局 -->
        <el-row :gutter="1" class="sync-layout">
          <!-- 版本历史 -->
          <el-col :span="8" class="sync-column">
            <el-card class="sync-card" shadow="never">
              <template #header>
                <div class="card-header">
                  <h4>版本历史</h4>
                  <div class="header-spacer"></div>
                </div>
              </template>
              <div class="card-content">
                <div class="history-list">
                  <div 
                    v-for="commit in commitHistory" 
                    :key="commit.id"
                    class="history-item"
                  >
                    <div class="commit-circle"></div>
                    <div class="commit-info">
                      <div class="commit-title">{{ commit.title }}</div>
                      <div class="commit-time">{{ formatTime(commit.time) }}</div>
                    </div>
                  </div>
                  <el-empty v-if="commitHistory.length === 0" description="暂无提交历史" :image-size="60" />
                </div>
              </div>
            </el-card>
          </el-col>

          <!-- 远端变更 -->
          <el-col :span="8" class="sync-column">
            <el-card class="sync-card" shadow="never">
              <template #header>
                <div class="card-header">
                  <h4>远端变更</h4>
                  <el-button 
                    type="primary"
                    size="small"
                    @click="syncFromRemote"
                    :disabled="isSyncing || remoteChanges.length === 0"
                    :loading="isSyncing"
                  >
                    同步
                  </el-button>
                </div>
              </template>
              <div class="card-content">
                <div class="changes-list">
                  <div 
                    v-for="change in remoteChanges" 
                    :key="change.id"
                    class="change-item remote"
                  >
                    <div class="change-circle"></div>
                    <div class="change-info">
                      <div class="change-title">{{ change.title }}</div>
                      <div class="change-time">{{ formatTime(change.time) }}</div>
                    </div>
                  </div>
                  <el-empty v-if="remoteChanges.length === 0" description="无远端变更" :image-size="60" />
                </div>
              </div>
            </el-card>
          </el-col>

          <!-- 本地变更 -->
          <el-col :span="8" class="sync-column">
            <el-card class="sync-card" shadow="never">
              <template #header>
                <div class="card-header">
                  <h4>本地变更</h4>
                  <el-button 
                    type="primary"
                    size="small"
                    @click="commitLocalChanges"
                    :disabled="isSyncing || localChanges.length === 0 || remoteChanges.length > 0"
                    :loading="isSyncing"
                  >
                    同步
                  </el-button>
                </div>
              </template>
              <div class="card-content">
                <div class="changes-list">
                  <div 
                    v-for="file in localChanges" 
                    :key="file.file_path"
                    class="file-item"
                    :class="file.status"
                  >
                    <div class="file-icon">📄</div>
                    <div class="file-info">
                      <div class="file-name">{{ getFileName(file.file_path) }}</div>
                      <div class="file-status">{{ getStatusText(file.status) }}</div>
                    </div>
                  </div>
                  <el-empty v-if="localChanges.length === 0" description="无本地变更" :image-size="60" />
                </div>
              </div>
            </el-card>
          </el-col>
        </el-row>
        
        <!-- 同步进度覆盖层 -->
        <div v-if="isSyncing" class="sync-progress-overlay">
          <div class="progress-content">
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: `${syncProgress}%` }"></div>
            </div>
            <div class="progress-text">{{ syncMessage }}</div>
          </div>
        </div>

        <!-- 提交信息输入对话框 -->
        <div v-if="showCommitDialog" class="commit-dialog-overlay">
          <div class="commit-dialog">
            <h4>输入变更说明</h4>
            <textarea 
              v-model="commitMessage"
              placeholder="请描述本次变更内容..."
              rows="4"
              ref="commitInput"
            ></textarea>
            <div class="commit-actions">
              <button @click="confirmCommit" class="confirm-btn" :disabled="!commitMessage.trim()">
                确认提交
              </button>
              <button @click="cancelCommit" class="cancel-btn">
                取消
              </button>
            </div>
          </div>
        </div>

        <!-- 同步结果提示 -->
        <div v-if="syncResult" class="sync-result-toast" :class="{ success: syncResult.success, error: !syncResult.success }">
          {{ syncResult.message }}
        </div>
      </div>
      
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import type { GitSyncConfig, SyncStatus, SyncDiff, SyncResult } from '@/types'
import { api } from '@/utils/api'

const emit = defineEmits<{
  close: []
}>()

// 数据
const gitConfig = ref<GitSyncConfig | null>(null)
const syncStatus = ref<SyncStatus | null>(null)
const syncResult = ref<SyncResult | null>(null)

// 新的数据结构
const commitHistory = ref<Array<{id: string, title: string, time: string}>>([])
const remoteChanges = ref<Array<{id: string, title: string, time: string}>>([])
const localChanges = ref<SyncDiff[]>([])

// 状态
const isSyncing = ref(false)
const syncProgress = ref(0)
const syncMessage = ref('')
const showCommitDialog = ref(false)
const commitMessage = ref('')

// 计算属性
const canSync = computed(() => {
  return gitConfig.value?.enabled && gitConfig.value?.repository_url && !isSyncing.value
})

// 方法
const closeDialog = () => {
  if (!isSyncing.value) {
    emit('close')
  }
}

const formatTime = (timeStr: string) => {
  try {
    const date = new Date(timeStr)
    if (isNaN(date.getTime())) {
      return '时间解析错误'
    }
    return date.toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch (error) {
    console.error('Failed to format time:', timeStr, error)
    return '无效时间'
  }
}

const getStatusText = (status: string) => {
  const map = {
    'added': '新增',
    'modified': '修改', 
    'deleted': '删除'
  }
  return map[status as keyof typeof map] || status
}

const getFileName = (filePath: string) => {
  return filePath.split('/').pop() || filePath
}

// 加载初始数据
const loadData = async () => {
  try {
    const config = await api.getGitSyncConfig()
    gitConfig.value = config
    
    if (config?.enabled) {
      // 并行加载所有数据
      await Promise.all([
        loadCommitHistory(),
        loadRemoteChanges(),
        loadLocalChanges()
      ])
    }
  } catch (error) {
    console.error('Failed to load sync data:', error)
  }
}

// 加载提交历史
const loadCommitHistory = async () => {
  try {
    // 获取本地提交历史
    const history = await api.getCommitHistory()
    
    // 按时间倒序排序
    commitHistory.value = history.sort((a: any, b: any) => {
      return new Date(b.time).getTime() - new Date(a.time).getTime()
    })
  } catch (error) {
    console.error('Failed to load commit history:', error)
    // 使用模拟数据作为后备
    commitHistory.value = [
      { id: '1', title: '初始化项目', time: new Date().toISOString() },
      { id: '2', title: '添加基础功能', time: new Date(Date.now() - 86400000).toISOString() }
    ].sort((a, b) => new Date(b.time).getTime() - new Date(a.time).getTime())
  }
}

// 加载远端变更
const loadRemoteChanges = async () => {
  try {
    console.log('🔍 Loading remote changes...')
    const commits = await api.getRemoteCommits()
    console.log('📥 Found remote commits:', commits.length)
    
    // 转换为界面需要的格式
    remoteChanges.value = commits.map((commit: any) => ({
      id: commit.id,
      title: commit.title,
      time: commit.time,
      author: commit.author
    }))
  } catch (error) {
    console.error('Failed to load remote changes:', error)
    remoteChanges.value = []
  }
}

// 加载本地变更
const loadLocalChanges = async () => {
  try {
    console.log('🔍 Loading local changes...')
    const localDiffs = await api.getLocalChanges()
    console.log('📝 Found local diffs:', localDiffs.length)
    localChanges.value = localDiffs
  } catch (error) {
    console.error('Failed to load local changes:', error)
  }
}

// 远端同步：stash -> pull -> stash pop
const syncFromRemote = async () => {
  if (isSyncing.value || remoteChanges.value.length === 0) return
  
  isSyncing.value = true
  syncProgress.value = 0
  syncMessage.value = '正在暂存本地变更...'
  
  try {
    // 1. Stash 本地变更
    if (localChanges.value.length > 0) {
      syncProgress.value = 20
      await api.stashChanges()
      syncMessage.value = '本地变更已暂存'
    }
    
    // 2. 拉取远端变更
    syncProgress.value = 50
    syncMessage.value = '正在拉取远端变更...'
    await api.pullFromRemote()
    
    // 3. 恢复本地变更
    if (localChanges.value.length > 0) {
      syncProgress.value = 80
      syncMessage.value = '正在恢复本地变更...'
      await api.stashPop()
    }
    
    syncProgress.value = 100
    syncMessage.value = '远端同步完成'
    
    syncResult.value = {
      success: true,
      message: '远端变更同步成功'
    }
    
    // 刷新数据
    await loadData()
    
  } catch (error) {
    console.error('Remote sync failed:', error)
    syncResult.value = {
      success: false,
      message: `远端同步失败: ${error}`
    }
  } finally {
    isSyncing.value = false
    // 3秒后隐藏结果提示
    setTimeout(() => {
      syncResult.value = null
    }, 3000)
  }
}

// 本地变更提交：检查远端 -> 输入说明 -> 提交 -> pull rebase -> push
const commitLocalChanges = async () => {
  // 检查是否有远端变更
  if (remoteChanges.value.length > 0) {
    syncResult.value = {
      success: false,
      message: '请先同步远端变更'
    }
    setTimeout(() => {
      syncResult.value = null
    }, 3000)
    return
  }
  
  if (localChanges.value.length === 0) return
  
  // 显示提交信息输入对话框
  showCommitDialog.value = true
  commitMessage.value = ''
  
  // 聚焦到输入框
  await nextTick()
  const input = document.querySelector('.commit-dialog textarea') as HTMLTextAreaElement
  if (input) {
    input.focus()
  }
}

// 确认提交
const confirmCommit = async () => {
  if (!commitMessage.value.trim()) return
  
  showCommitDialog.value = false
  isSyncing.value = true
  syncProgress.value = 0
  syncMessage.value = '正在准备提交...'
  
  try {
    // 1. 首先拉取远端最新变更
    syncProgress.value = 20
    syncMessage.value = '正在拉取远端最新变更...'
    await api.pullRebase()
    
    // 2. 提交本地变更（此时基于最新远端）
    syncProgress.value = 50
    syncMessage.value = '正在提交本地变更...'
    await api.commitChanges(commitMessage.value.trim())
    
    // 3. 再次检查远端是否有新变更并 rebase
    syncProgress.value = 70
    syncMessage.value = '正在确保快进式推送...'
    await api.pullRebase()
    
    // 4. 推送到远端（使用智能推送，自动处理非快进情况）
    syncProgress.value = 85
    syncMessage.value = '正在推送到远程仓库...'
    await api.smartPushToRemote()
    
    syncProgress.value = 100
    syncMessage.value = '本地变更提交完成'
    
    syncResult.value = {
      success: true,
      message: '本地变更提交成功'
    }
    
    commitMessage.value = ''
    
    // 刷新数据
    await loadData()
    
  } catch (error) {
    console.error('Local commit failed:', error)
    syncResult.value = {
      success: false,
      message: `本地提交失败: ${error}`
    }
  } finally {
    isSyncing.value = false
    // 3秒后隐藏结果提示
    setTimeout(() => {
      syncResult.value = null
    }, 3000)
  }
}

// 取消提交
const cancelCommit = () => {
  showCommitDialog.value = false
  commitMessage.value = ''
}


onMounted(() => {
  loadData()
})
</script>

<style scoped>
.sync-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.sync-dialog {
  background: #ffffff;
  color: #333333;
  border-radius: 8px;
  width: 1000px;
  height: 85vh;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.sync-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e5e5e5;
}

.sync-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #333333;
}

.repo-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: #666666;
}

.repo-info .branch {
  background: #007acc;
  color: #ffffff;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666666;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  color: #333333;
  background: #f0f0f0;
  border-radius: 4px;
}

.sync-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* Element Plus 布局样式 */
.sync-layout {
  height: 100%;
  background: #f5f5f5;
}

.sync-column {
  height: 100%;
}

.sync-card {
  height: 100%;
  border: none;
  border-radius: 0;
}

.sync-card :deep(.el-card__header) {
  padding: 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #e5e5e5;
  min-height: 68px; /* 固定高度确保对齐 */
  display: flex;
  align-items: center;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
  min-height: 36px; /* 确保内容区域高度一致 */
}

.card-header h4 {
  margin: 0;
  padding: 0;
  font-size: 14px;
  font-weight: 600;
  color: #333333;
  line-height: 36px; /* 与按钮高度对齐 */
}

.header-spacer {
  width: 60px; /* 与同步按钮宽度相同 */
  height: 32px; /* 与同步按钮高度相同 */
}

.sync-card :deep(.el-card__body) {
  padding: 0;
  height: calc(100% - 68px); /* 减去header高度 */
  overflow: hidden;
}

.card-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.history-list,
.changes-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  min-height: 0;
}

/* 版本历史项 */
.history-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px;
  margin-bottom: 8px;
  border-radius: 4px;
}

.history-item:hover {
  background: #f8f9fa;
}

.commit-circle {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #007acc;
  margin-top: 6px;
  flex-shrink: 0;
}

.commit-info {
  flex: 1;
  min-width: 0;
}

.commit-title {
  font-size: 13px;
  color: #333333;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.commit-time {
  font-size: 11px;
  color: #666666;
}

/* 远端变更项 */
.change-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 8px;
  margin-bottom: 8px;
  border-radius: 4px;
}

.change-item:hover {
  background: #f8f9fa;
}

.change-circle {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ffc107;
  margin-top: 6px;
  flex-shrink: 0;
}

.change-info {
  flex: 1;
  min-width: 0;
}

.change-title {
  font-size: 13px;
  color: #333333;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.change-time {
  font-size: 11px;
  color: #666666;
}

/* 本地文件项 */
.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  margin-bottom: 4px;
  border-radius: 4px;
}

.file-item:hover {
  background: #f8f9fa;
}

.file-item.added {
  border-left: 3px solid #28a745;
}

.file-item.modified {
  border-left: 3px solid #ffc107;
}

.file-item.deleted {
  border-left: 3px solid #dc3545;
}

.file-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  color: #333333;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-status {
  font-size: 11px;
  color: #666666;
}

/* Element Plus empty component styling */
.sync-card :deep(.el-empty) {
  padding: 20px 8px;
}

.sync-card :deep(.el-empty__description) {
  color: #999999;
  font-size: 12px;
  margin-top: 8px;
}

.sync-status {
  margin-bottom: 20px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 6px;
}

.status-item {
  display: flex;
  margin-bottom: 8px;
}

.status-item:last-child {
  margin-bottom: 0;
}

.status-item .label {
  width: 80px;
  font-weight: 500;
  color: #666;
}

.status-item .value {
  flex: 1;
  color: #333;
}

.sync-progress {
  margin-bottom: 20px;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #e5e5e5;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-fill {
  height: 100%;
  background: #007acc;
  transition: width 0.3s ease;
}

.progress-text {
  text-align: center;
  color: #666;
  font-size: 14px;
}

.conflicts-section,
.diffs-section,
.commit-section {
  margin-bottom: 20px;
}

.conflicts-section h4,
.diffs-section h4,
.commit-section h4 {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.conflicts-list,
.diffs-list {
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  overflow: hidden;
}

.conflict-item,
.diff-item {
  border-bottom: 1px solid #e5e5e5;
}

.conflict-item:last-child,
.diff-item:last-child {
  border-bottom: none;
}

.conflict-header,
.diff-header {
  padding: 12px 16px;
  background: #f8f9fa;
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-path {
  flex: 1;
  font-family: monospace;
  font-size: 14px;
  color: #333;
}

.change-type {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.change-type.added {
  background: #d4edda;
  color: #155724;
}

.change-type.modified {
  background: #fff3cd;
  color: #856404;
}

.change-type.deleted {
  background: #f8d7da;
  color: #721c24;
}

.diff-stats {
  font-family: monospace;
  font-size: 12px;
}

.additions {
  color: #28a745;
  margin-right: 8px;
}

.deletions {
  color: #dc3545;
}

.conflict-content,
.diff-content {
  padding: 16px;
  background: #f8f8f8;
}

.conflict-content pre,
.diff-content pre {
  margin: 0;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.4;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.toggle-diff-btn {
  padding: 8px 16px;
  background: none;
  border: none;
  color: #007acc;
  cursor: pointer;
  font-size: 12px;
  width: 100%;
  text-align: left;
}

.toggle-diff-btn:hover {
  background: #f0f0f0;
}

.conflict-actions {
  margin-top: 16px;
  text-align: center;
}

.resolve-btn {
  padding: 8px 16px;
  background: #28a745;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.resolve-btn:hover {
  background: #218838;
}

.commit-section textarea {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-family: inherit;
  font-size: 14px;
  resize: vertical;
}

.commit-section textarea:focus {
  outline: none;
  border-color: #007acc;
}

.sync-result {
  margin-bottom: 20px;
}

.result-message {
  padding: 12px 16px;
  border-radius: 6px;
  font-weight: 500;
}

.result-message.success {
  background: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.result-message.error {
  background: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

/* 同步进度覆盖层 */
.sync-progress-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.progress-content {
  background: #ffffff;
  padding: 30px;
  border-radius: 8px;
  min-width: 300px;
  text-align: center;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #e5e5e5;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 16px;
}

.progress-fill {
  height: 100%;
  background: #007acc;
  transition: width 0.3s ease;
}

.progress-text {
  color: #333333;
  font-size: 14px;
}

/* 提交对话框覆盖层 */
.commit-dialog-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.commit-dialog {
  background: #ffffff;
  padding: 24px;
  border-radius: 8px;
  min-width: 400px;
  max-width: 500px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.commit-dialog h4 {
  margin: 0 0 16px 0;
  color: #333333;
  font-size: 16px;
}

.commit-dialog textarea {
  width: 100%;
  padding: 12px;
  background: #ffffff;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  color: #333333;
  font-family: inherit;
  font-size: 14px;
  resize: vertical;
  margin-bottom: 16px;
}

.commit-dialog textarea:focus {
  outline: none;
  border-color: #007acc;
}

.commit-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.confirm-btn {
  padding: 8px 16px;
  background: #007acc;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.confirm-btn:hover:not(:disabled) {
  background: #0066b3;
}

.confirm-btn:disabled {
  background: #e5e5e5;
  color: #999999;
  cursor: not-allowed;
}

.cancel-btn {
  padding: 8px 16px;
  background: #f8f9fa;
  color: #333333;
  border: 1px solid #e5e5e5;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.cancel-btn:hover {
  background: #e9ecef;
}

/* 同步结果提示 */
.sync-result-toast {
  position: absolute;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  z-index: 200;
  animation: slideDown 0.3s ease;
}

.sync-result-toast.success {
  background: #28a745;
  color: #ffffff;
}

.sync-result-toast.error {
  background: #dc3545;
  color: #ffffff;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

.sync-footer {
  padding: 16px 20px;
  border-top: 1px solid #3d3d3d;
  display: flex;
  justify-content: center;
}

.close-btn-footer {
  padding: 8px 16px;
  background: #3d3d3d;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.close-btn-footer:hover {
  background: #4d4d4d;
}
</style>