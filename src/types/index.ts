export interface Note {
  id: string
  title: string
  content: string
  file_path: string
  created_at: string
  modified_at: string
  is_favorite: boolean
  is_deleted: boolean
  tags: string[]
  has_attachments: boolean
  attachments: string[]
}

export interface Tag {
  id: string
  name: string
  note_count: number
}

export interface CreateNoteRequest {
  title: string
  content?: string
  tags?: string[]
}

export interface UpdateNoteRequest {
  id: string
  title?: string
  content?: string
  is_favorite?: boolean
  tags?: string[]
}

export interface SearchRequest {
  query: string
  tag_filter?: string
}

export type ViewMode = 'view' | 'edit' | 'split'

export interface GitSyncConfig {
  enabled: boolean
  repository_url: string
  branch: string
  username?: string
  password?: string
  ssh_key_path?: string
  auth_type: 'none' | 'basic' | 'ssh'
}

export interface SyncStatus {
  is_syncing: boolean
  last_sync: string | null
  has_conflicts: boolean
  local_changes: number
  remote_changes: number
}

export interface SyncDiff {
  file_path: string
  status: 'added' | 'modified' | 'deleted'
  local_content?: string
  remote_content?: string
  diff_content: string
}

export interface SyncResult {
  success: boolean
  message: string
  conflicts?: SyncDiff[]
  changes_pushed: number
  changes_pulled: number
}