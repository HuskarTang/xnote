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

  async searchNotes(request: SearchRequest): Promise<Note[]> {
    return await invoke('search_notes', { request })
  },

  // Tags
  async getAllTags(): Promise<Tag[]> {
    return await invoke('get_all_tags')
  },
}