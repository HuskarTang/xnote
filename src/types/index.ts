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

export interface EditorState {
  currentNote: Note | null
  viewMode: ViewMode
  isPreviewVisible: boolean
  isDirty: boolean
  lastSaved: Date | null
}

export interface GitSyncConfig {
  enabled: boolean
  repository_url: string
  branch: string
  username?: string
  password?: string
  ssh_key_path?: string
  auth_type: 'none' | 'basic' | 'ssh'
}

export interface LogConfig {
  enabled: boolean
  level: string
  max_days: number
  console_output: boolean
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

export type GitSyncOutcome = 'success' | 'conflicted' | 'blocked'
export type SyncTransactionType = 'setup' | 'sync'
export type SyncPhase =
  | 'testing_connection'
  | 'preparing_local_snapshot'
  | 'fetching_remote'
  | 'updating_tracking_branch'
  | 'applying_local_snapshot'
  | 'conflicted'
  | 'pushing'
  | 'completed'
  | 'aborted'

export type GitConflictStatus =
  | 'both_modified'
  | 'local_deleted'
  | 'remote_deleted'
  | 'both_added'
  | 'unsupported'

export interface GitConflictFile {
  file_path: string
  status: GitConflictStatus
  local_content?: string
  remote_content?: string
  is_binary: boolean
  diff_summary?: string
}

export interface PendingSyncState {
  transaction_id: string
  transaction_type: SyncTransactionType
  phase: SyncPhase
  target_branch: string
  temporary_branch: string
  remote_url: string
  pre_transaction_head?: string
  conflicts: GitConflictFile[]
}

export interface GitConnectionTestResult {
  success: boolean
  repository_reachable: boolean
  auth_success: boolean
  default_branch?: string
  target_branch: string
  target_branch_exists: boolean
  will_create_branch: boolean
  data_dir_has_git: boolean
  data_dir_has_uncommitted_content: boolean
  remote_mismatch: boolean
  pending_transaction?: PendingSyncState
  actions: string[]
  message: string
}

export interface GitSyncTransactionResult {
  outcome: GitSyncOutcome
  message: string
  target_branch?: string
  temporary_branch?: string
  pushed: boolean
  conflicts: GitConflictFile[]
  pending?: PendingSyncState
}

export interface ResolvedConflictFile {
  file_path: string
  final_content?: string
}
