import { invoke } from '@tauri-apps/api/tauri'
import type { Note, Tag, CreateNoteRequest, UpdateNoteRequest, SearchRequest } from '@/types'

export const api = {
  // Notes
  async getAllNotes(): Promise<Note[]> {
    return await invoke('get_all_notes')
  },

  async getFavorites(): Promise<Note[]> {
    return await invoke('get_favorites')
  },

  async getUntagged(): Promise<Note[]> {
    return await invoke('get_untagged')
  },

  async getTrash(): Promise<Note[]> {
    return await invoke('get_trash')
  },

  async getNotesByTag(tagName: string): Promise<Note[]> {
    return await invoke('get_notes_by_tag', { tagName })
  },

  async getNote(id: string): Promise<Note | null> {
    return await invoke('get_note', { id })
  },

  async createNote(request: CreateNoteRequest): Promise<Note> {
    return await invoke('create_note', { request })
  },

  async updateNote(request: UpdateNoteRequest): Promise<Note | null> {
    return await invoke('update_note', { request })
  },

  async deleteNote(id: string): Promise<boolean> {
    return await invoke('delete_note', { id })
  },

  async permanentlyDeleteNote(id: string): Promise<boolean> {
    return await invoke('permanently_delete_note', { id })
  },

  async restoreNote(id: string): Promise<boolean> {
    return await invoke('restore_note', { id })
  },

  async searchNotes(request: SearchRequest): Promise<Note[]> {
    return await invoke('search_notes', { request })
  },

  // Tags
  async getAllTags(): Promise<Tag[]> {
    return await invoke('get_all_tags')
  },

  async createTag(name: string): Promise<Tag> {
    return await invoke('create_tag', { name })
  },

  async deleteTag(tagId: string): Promise<boolean> {
    return await invoke('delete_tag', { tagId })
  },

  async renameTag(tagId: string, newName: string): Promise<Tag | null> {
    return await invoke('rename_tag', { tagId, newName })
  },

  async addTagToNote(noteId: string, tagName: string): Promise<Tag> {
    return await invoke('add_tag_to_note', { noteId, tagName })
  },

  async removeTagFromNote(noteId: string, tagName: string): Promise<boolean> {
    return await invoke('remove_tag_from_note', { noteId, tagName })
  },

  async getNoteTags(noteId: string): Promise<Tag[]> {
    return await invoke('get_note_tags', { noteId })
  },

  async searchTags(query: string): Promise<Tag[]> {
    return await invoke('search_tags', { query })
  },

  async cleanupUnusedTags(): Promise<number> {
    return await invoke('cleanup_unused_tags')
  },
  
  // Configuration
  async showDirectoryDialog(): Promise<string | null> {
    const result = await invoke('show_directory_dialog')
    return result as string | null
  },
  
  async updateDataDirectory(path: string): Promise<boolean> {
    return await invoke('update_data_directory', { path })
  },
  
  async reinitializeDataDirectory(path: string): Promise<boolean> {
    return await invoke('reinitialize_data_directory', { path })
  },
  
  async isSetupRequired(): Promise<boolean> {
    return await invoke('is_setup_required')
  },
  
  async markSetupComplete(): Promise<boolean> {
    return await invoke('mark_setup_complete')
  },
  
  async syncExternalFiles(): Promise<Note[]> {
    return await invoke('sync_external_files')
  },

  // Git Sync APIs
  async getAppConfig(): Promise<any> {
    return await invoke('get_app_config')
  },

  async getGitSyncConfig(): Promise<any> {
    return await invoke('get_git_sync_config')
  },

  async updateGitSyncConfig(config: any): Promise<boolean> {
    return await invoke('update_git_sync_config', { config })
  },

  async updateTheme(theme: string): Promise<boolean> {
    return await invoke('update_theme', { theme })
  },

  async getSyncStatus(): Promise<any> {
    return await invoke('get_sync_status')
  },

  async getLocalChanges(): Promise<any[]> {
    return await invoke('get_local_changes')
  },

  async performSync(): Promise<any> {
    return await invoke('perform_sync')
  },

  async commitChanges(message: string): Promise<boolean> {
    return await invoke('commit_changes', { message })
  },

  // 新的Git操作API
  async stashChanges(): Promise<boolean> {
    return await invoke('stash_changes')
  },

  async stashPop(): Promise<boolean> {
    return await invoke('stash_pop')
  },

  async pullFromRemote(): Promise<boolean> {
    return await invoke('pull_from_remote')
  },

  async pullRebase(): Promise<boolean> {
    return await invoke('pull_rebase')
  },

  async pushToRemote(): Promise<boolean> {
    return await invoke('push_to_remote')
  },

  async smartPushToRemote(): Promise<boolean> {
    return await invoke('smart_push_to_remote')
  },

  async getCommitHistory(): Promise<any[]> {
    return await invoke('get_commit_history')
  },

  async getRemoteChanges(): Promise<any[]> {
    return await invoke('get_remote_changes')
  },

  async getRemoteCommits(): Promise<any[]> {
    return await invoke('get_remote_commits')
  }
}