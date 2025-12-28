export interface AppConfig {
  data_directory?: string;
  window_width: number;
  window_height: number;
  sidebar_width: number;
  note_list_width: number;
  auto_save_interval: number;
  theme: string;
}

export interface Note {
  id: string;
  title: string;
  file_path: string;
  created_at: string;
  modified_at: string;
  is_favorite: boolean;
  is_trashed: boolean;
}

export interface Tag {
  id: string;
  name: string;
  color?: string;
  created_at: string;
}

export interface NoteWithTags {
  note: Note;
  tags: Tag[];
}

export interface NoteContent {
  id: string;
  title: string;
  content: string;
  tags: Tag[];
  created_at: string;
  modified_at: string;
  is_favorite: boolean;
  is_trashed: boolean;
}

export interface CreateNoteRequest {
  title?: string;
  content?: string;
  tags?: string[];
}

export interface SaveNoteRequest {
  id: string;
  title: string;
  content: string;
}

export interface SearchRequest {
  query: string;
  tag_filter?: string;
  include_content: boolean;
}

export interface AttachmentInfo {
  id: string;
  original_name: string;
  file_path: string;
  file_size: number;
  mime_type: string;
  created_at: string;
}

export type ViewMode = 'view' | 'edit' | 'split';

export type SidebarView = 'all' | 'favorites' | 'tags' | 'untagged' | 'trash';

export interface EditorState {
  currentNote: NoteContent | null;
  viewMode: ViewMode;
  isPreviewVisible: boolean;
  isDirty: boolean;
  lastSaved: Date | null;
}