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
}

export interface Tag {
  id: string
  name: string
  note_count: number
}

export interface CreateNoteRequest {
  title: string
  content?: string
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