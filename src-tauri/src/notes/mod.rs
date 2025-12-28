use crate::storage::FileStorageManager;
use crate::models::{Note, NoteMetadata, CreateNoteRequest, UpdateNoteRequest};
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::fs;

pub struct NotesManager {
    storage: FileStorageManager,
}

impl NotesManager {
    pub fn new(storage: FileStorageManager) -> Self {
        Self { storage }
    }

    fn metadata_to_note(&self, metadata: NoteMetadata, content: String, file_path: String) -> Note {
        let created_at = DateTime::parse_from_rfc3339(&metadata.created)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
            
        let modified_at = DateTime::parse_from_rfc3339(&metadata.modified)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
            
        Note {
            id: metadata.id,
            title: metadata.title,
            content,
            file_path,
            created_at,
            modified_at,
            is_favorite: metadata.favorite,
            is_deleted: metadata.deleted,
            tags: metadata.tags,
            has_attachments: !metadata.attachments.is_empty(),
            attachments: metadata.attachments,
        }
    }

    pub async fn create_note(&self, request: CreateNoteRequest) -> Result<Note> {
        let id = Uuid::new_v4().to_string();
        let title = if request.title.trim().is_empty() {
            "Untitled".to_string()
        } else {
            request.title.trim().to_string()
        };
        
        let content = request.content.unwrap_or_else(|| "edit your Note here with Markdown...".to_string());
        
        let file_path_buf = self.storage.generate_unique_file_path(&title)?;
        let file_name = file_path_buf.file_name().unwrap().to_string_lossy().to_string();
        
        let now = Utc::now();
        let metadata = NoteMetadata {
            id: id.clone(),
            title: title.clone(),
            tags: request.tags.unwrap_or_default(),
            attachments: vec![],
            created: now.to_rfc3339(),
            modified: now.to_rfc3339(),
            favorite: false,
            deleted: false,
        };
        
        self.storage.save_note(&file_name, &metadata, &content)?;
        
        Ok(self.metadata_to_note(metadata, content, file_name))
    }
    
    pub async fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let files = self.storage.scan_existing_files()?;
        for file_info in files {
            if let Ok((metadata, content)) = self.storage.parse_note(&file_info.name) {
                if metadata.id == id {
                    if !metadata.deleted {
                        return Ok(Some(self.metadata_to_note(metadata, content, file_info.name)));
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn get_all_notes(&self) -> Result<Vec<Note>> {
        let files = self.storage.scan_existing_files()?;
        let mut notes = Vec::new();
        
        for file_info in files {
            match self.storage.parse_note(&file_info.name) {
                Ok((metadata, content)) => {
                     if !metadata.deleted {
                         notes.push(self.metadata_to_note(metadata, content, file_info.name));
                     }
                },
                Err(e) => {
                    log::error!("Failed to parse note {}: {}", file_info.name, e);
                }
            }
        }
        notes.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(notes)
    }
    
    pub async fn get_favorite_notes(&self) -> Result<Vec<Note>> {
        let notes = self.get_all_notes().await?;
        Ok(notes.into_iter().filter(|n| n.is_favorite).collect())
    }
    
    pub async fn get_notes_by_tag(&self, tag_name: &str) -> Result<Vec<Note>> {
        let notes = self.get_all_notes().await?;
        Ok(notes.into_iter().filter(|n| n.tags.contains(&tag_name.to_string())).collect())
    }
    
    pub async fn get_untagged_notes(&self) -> Result<Vec<Note>> {
        let notes = self.get_all_notes().await?;
        Ok(notes.into_iter().filter(|n| n.tags.is_empty()).collect())
    }
    
    pub async fn get_trash(&self) -> Result<Vec<Note>> {
        let files = self.storage.scan_existing_files()?;
        let mut notes = Vec::new();
        
        for file_info in files {
            match self.storage.parse_note(&file_info.name) {
                Ok((metadata, content)) => {
                     if metadata.deleted {
                         notes.push(self.metadata_to_note(metadata, content, file_info.name));
                     }
                },
                Err(e) => {
                    log::error!("Failed to parse note {}: {}", file_info.name, e);
                }
            }
        }
        notes.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(notes)
    }
    
    pub async fn update_note(&self, request: UpdateNoteRequest) -> Result<Option<Note>> {
        // Find note file by ID
        let mut target_file: Option<String> = None;
        let mut target_metadata: Option<NoteMetadata> = None;
        let mut target_content: Option<String> = None;
        
        let files = self.storage.scan_existing_files()?;
        for file_info in &files {
            match self.storage.parse_note(&file_info.name) {
                Ok((metadata, content)) => {
                    if metadata.id == request.id {
                        target_file = Some(file_info.name.clone());
                        target_metadata = Some(metadata);
                        target_content = Some(content);
                        break;
                    }
                },
                Err(e) => {
                    log::warn!("Failed to parse note during update scan: {} - {}", file_info.name, e);
                }
            }
        }
        
        if let (Some(file_name), Some(mut metadata), Some(mut content)) = (target_file, target_metadata, target_content) {
            let mut new_file_name = file_name.clone();
            
            // Update metadata
            if let Some(title) = request.title {
                if title != metadata.title {
                    metadata.title = title.clone();
                    // Rename file
                     match self.storage.rename_note_file(&file_name, &title) {
                        Ok(new_name) => {
                            new_file_name = new_name;
                        },
                        Err(e) => return Err(e),
                     }
                }
            }
            
            if let Some(content_update) = request.content {
                content = content_update;
            }
            
            if let Some(fav) = request.is_favorite {
                metadata.favorite = fav;
            }
            
            if let Some(tags) = request.tags {
                metadata.tags = tags;
            }
            
            metadata.modified = Utc::now().to_rfc3339();
            
            self.storage.save_note(&new_file_name, &metadata, &content)?;
            
            Ok(Some(self.metadata_to_note(metadata, content, new_file_name)))
        } else {
            log::warn!("Note with id {} not found during update. Scanned {} files.", request.id, files.len());
            Ok(None)
        }
    }
    
    pub async fn delete_note(&self, id: &str) -> Result<bool> {
        self.set_note_deleted(id, true).await
    }
    
    pub async fn restore_note(&self, id: &str) -> Result<bool> {
        self.set_note_deleted(id, false).await
    }
    
    async fn set_note_deleted(&self, id: &str, deleted: bool) -> Result<bool> {
        let files = self.storage.scan_existing_files()?;
        for file_info in files {
            if let Ok((mut metadata, content)) = self.storage.parse_note(&file_info.name) {
                if metadata.id == id {
                    metadata.deleted = deleted;
                    metadata.modified = Utc::now().to_rfc3339();
                    self.storage.save_note(&file_info.name, &metadata, &content)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    
    pub async fn permanently_delete_note(&self, id: &str) -> Result<bool> {
        let files = self.storage.scan_existing_files()?;
        for file_info in files {
            if let Ok((metadata, _)) = self.storage.parse_note(&file_info.name) {
                if metadata.id == id {
                    self.storage.delete_note_file(&file_info.name)?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    
    pub async fn search_notes(&self, query: &str, tag_filter: Option<&str>) -> Result<Vec<Note>> {
        let base_notes = match tag_filter {
            Some("All Notes") | None => self.get_all_notes().await?,
            Some("Favorites") => self.get_favorite_notes().await?,
            Some("Untagged") => self.get_untagged_notes().await?,
            Some("Trash") => self.get_trash().await?,
            Some(tag_name) => self.get_notes_by_tag(tag_name).await?,
        };
        
        let query_lower = query.to_lowercase();
        let mut matching_notes = Vec::new();
        
        for note in base_notes {
            let title_matches = note.title.to_lowercase().contains(&query_lower);
            let content_matches = note.content.to_lowercase().contains(&query_lower);
            
            if title_matches || content_matches {
                matching_notes.push(note);
            }
        }
        
        matching_notes.sort_by(|a, b| {
            let a_title_match = a.title.to_lowercase().contains(&query_lower);
            let b_title_match = b.title.to_lowercase().contains(&query_lower);
            
            match (a_title_match, b_title_match) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.modified_at.cmp(&a.modified_at),
            }
        });
        
        Ok(matching_notes)
    }

    pub async fn sync_external_files(&self) -> Result<Vec<Note>> {
        self.get_all_notes().await
    }
    
    pub async fn add_attachment_to_note(&self, note_id: &str, attachment_path: &str) -> Result<()> {
        let files = self.storage.scan_existing_files()?;
        for file_info in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file_info.name) {
                if metadata.id == note_id {
                    if !metadata.attachments.contains(&attachment_path.to_string()) {
                        metadata.attachments.push(attachment_path.to_string());
                        self.storage.save_note(&file_info.name, &metadata, &content)?;
                    }
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("Note not found"))
    }
    
    pub async fn remove_attachment_from_note(&self, note_id: &str, attachment_path: &str) -> Result<()> {
        let files = self.storage.scan_existing_files()?;
        for file_info in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file_info.name) {
                if metadata.id == note_id {
                    if let Some(pos) = metadata.attachments.iter().position(|x| x == attachment_path) {
                        metadata.attachments.remove(pos);
                        self.storage.save_note(&file_info.name, &metadata, &content)?;
                    }
                    return Ok(());
                }
            }
        }
        Err(anyhow::anyhow!("Note not found"))
    }

    pub async fn cleanup_unused_attachments(&self) -> Result<usize> {
        let files = self.storage.scan_existing_files()?;
        let mut used_attachments = HashSet::new();
        
        for file_info in files {
            if let Ok((metadata, _)) = self.storage.parse_note(&file_info.name) {
                for att in metadata.attachments {
                    used_attachments.insert(att);
                }
            }
        }
        
        let attachments_dir = self.storage.notes_directory.join("attachments");
        if !attachments_dir.exists() {
            return Ok(0);
        }
        
        let mut deleted_count = 0;
        let entries = fs::read_dir(&attachments_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name() {
                    let relative_path = format!("attachments/{}", name.to_string_lossy());
                    if !used_attachments.contains(&relative_path) {
                        fs::remove_file(&path)?;
                        deleted_count += 1;
                    }
                }
            }
        }
        
        Ok(deleted_count)
    }
}