use crate::database::{DatabaseManager, NoteRecord};
use crate::storage::FileStorageManager;
use crate::models::{Note, CreateNoteRequest, UpdateNoteRequest};
use anyhow::{Result, Context};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::Row;

pub struct NotesManager {
    db: DatabaseManager,
    storage: FileStorageManager,
}

impl NotesManager {
    pub fn new(db: DatabaseManager, storage: FileStorageManager) -> Self {
        Self { db, storage }
    }
    
    pub async fn create_note(&self, request: CreateNoteRequest) -> Result<Note> {
        let id = Uuid::new_v4().to_string();
        let title = if request.title.trim().is_empty() {
            "Untitled".to_string()
        } else {
            request.title.trim().to_string()
        };
        
        let content = request.content.unwrap_or_else(|| "edit your Note here with Markdown...".to_string());
        
        // Create file
        let file_name = self.storage.create_note_file(&title, &content)
            .context("Failed to create note file")?;
        
        // Create record in database
        self.db.create_note(&id, &title, &file_name).await
            .context("Failed to create note in database")?;
        
        let now = Utc::now();
        let note = Note {
            id: id.clone(),
            title,
            content,
            file_path: file_name,
            created_at: now,
            modified_at: now,
            is_favorite: false,
            is_deleted: false,
            tags: vec![],
            has_attachments: false,
        };
        
        Ok(note)
    }
    
    pub async fn get_note(&self, id: &str) -> Result<Option<Note>> {
        let notes = self.db.get_all_notes().await?;
        
        for record in notes {
            if record.id == id {
                let content = self.storage.read_note_file(&record.file_path)
                    .unwrap_or_else(|_| "Failed to load content".to_string());
                
                let tags = self.db.get_note_tags(&record.id).await?;
                
                // TODO: Check for attachments
                let has_attachments = false;
                
                let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                
                let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());
                
                return Ok(Some(Note {
                    id: record.id,
                    title: record.title,
                    content,
                    file_path: record.file_path,
                    created_at,
                    modified_at,
                    is_favorite: record.is_favorite,
                    is_deleted: record.is_deleted,
                    tags,
                    has_attachments,
                }));
            }
        }
        
        Ok(None)
    }
    
    pub async fn get_all_notes(&self) -> Result<Vec<Note>> {
        let records = self.db.get_all_notes().await?;
        let mut notes = Vec::new();
        
        for record in records {
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Failed to load content".to_string());
            
            let tags = self.db.get_note_tags(&record.id).await?;
            
            // TODO: Check for attachments
            let has_attachments = false;
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let note = Note {
                id: record.id,
                title: record.title,
                content,
                file_path: record.file_path,
                created_at,
                modified_at,
                is_favorite: record.is_favorite,
                is_deleted: record.is_deleted,
                tags,
                has_attachments,
            };
            
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn get_favorite_notes(&self) -> Result<Vec<Note>> {
        let records = self.db.get_favorite_notes().await?;
        let mut notes = Vec::new();
        
        for record in records {
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Failed to load content".to_string());
            
            let tags = self.db.get_note_tags(&record.id).await?;
            let has_attachments = false; // TODO: Implement attachment check
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let note = Note {
                id: record.id,
                title: record.title,
                content,
                file_path: record.file_path,
                created_at,
                modified_at,
                is_favorite: record.is_favorite,
                is_deleted: record.is_deleted,
                tags,
                has_attachments,
            };
            
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn get_notes_by_tag(&self, tag_name: &str) -> Result<Vec<Note>> {
        // Get notes with specified tag name
        let rows = sqlx::query(
            r#"
            SELECT n.* FROM notes n
            JOIN note_tags nt ON n.id = nt.note_id
            JOIN tags t ON nt.tag_id = t.id
            WHERE t.name = ?1 AND n.is_deleted = FALSE
            ORDER BY n.modified_at DESC
            "#
        )
        .bind(tag_name)
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut notes = Vec::new();
        
        for row in rows {
            let record = NoteRecord {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                is_favorite: row.get("is_favorite"),
                is_deleted: row.get("is_deleted"),
            };
            
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Failed to load content".to_string());
            
            let tags = self.db.get_note_tags(&record.id).await?;
            let has_attachments = false;
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let note = Note {
                id: record.id,
                title: record.title,
                content,
                file_path: record.file_path,
                created_at,
                modified_at,
                is_favorite: record.is_favorite,
                is_deleted: record.is_deleted,
                tags,
                has_attachments,
            };
            
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn get_untagged_notes(&self) -> Result<Vec<Note>> {
        // Get notes without tags
        let rows = sqlx::query(
            r#"
            SELECT n.* FROM notes n
            LEFT JOIN note_tags nt ON n.id = nt.note_id
            WHERE nt.note_id IS NULL AND n.is_deleted = FALSE
            ORDER BY n.modified_at DESC
            "#
        )
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut notes = Vec::new();
        
        for row in rows {
            let record = NoteRecord {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                is_favorite: row.get("is_favorite"),
                is_deleted: row.get("is_deleted"),
            };
            
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Failed to load content".to_string());
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let note = Note {
                id: record.id,
                title: record.title,
                content,
                file_path: record.file_path,
                created_at,
                modified_at,
                is_favorite: record.is_favorite,
                is_deleted: record.is_deleted,
                tags: vec![],
                has_attachments: false,
            };
            
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn get_trash(&self) -> Result<Vec<Note>> {
        // Get deleted notes
        let rows = sqlx::query(
            "SELECT * FROM notes WHERE is_deleted = TRUE ORDER BY modified_at DESC"
        )
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut notes = Vec::new();
        
        for row in rows {
            let record = NoteRecord {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                is_favorite: row.get("is_favorite"),
                is_deleted: row.get("is_deleted"),
            };
            
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Content unavailable".to_string());
            
            let tags = self.db.get_note_tags(&record.id).await?;
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let note = Note {
                id: record.id,
                title: record.title,
                content,
                file_path: record.file_path,
                created_at,
                modified_at,
                is_favorite: record.is_favorite,
                is_deleted: record.is_deleted,
                tags,
                has_attachments: false,
            };
            
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn update_note(&self, request: UpdateNoteRequest) -> Result<Option<Note>> {
        // Get existing note record
        let notes = self.db.get_all_notes().await?;
        let current_record = notes.into_iter()
            .find(|n| n.id == request.id && !n.is_deleted);
        
        let mut record = match current_record {
            Some(record) => record,
            None => return Ok(None),
        };
        
        let mut file_renamed = false;
        let mut new_file_name = record.file_path.clone();
        
        // If title changes, rename file
        if let Some(ref new_title) = request.title {
            if new_title != &record.title {
                new_file_name = self.storage.rename_note_file(&record.file_path, new_title)
                    .context("Failed to rename note file")?;
                file_renamed = true;
                record.title = new_title.clone();
            }
        }
        
        // Update file content
        if let Some(ref content) = request.content {
            self.storage.update_note_file(&new_file_name, content)
                .context("Failed to update note content")?;
        }
        
        // Update database record
        self.db.update_note(
            &request.id,
            request.title.as_deref(),
            request.is_favorite,
        ).await?;
        
        // If file was renamed, update file path in database
        if file_renamed {
            sqlx::query("UPDATE notes SET file_path = ?1, modified_at = ?2 WHERE id = ?3")
                .bind(&new_file_name)
                .bind(Utc::now().to_rfc3339())
                .bind(&request.id)
                .execute(self.db.get_pool())
                .await?;
        }
        
        // Update tags
        if let Some(ref new_tags) = request.tags {
            // First remove all existing tags
            sqlx::query("DELETE FROM note_tags WHERE note_id = ?1")
                .bind(&request.id)
                .execute(self.db.get_pool())
                .await?;
            
            // Add new tags
            for tag_name in new_tags {
                // Find or create tag
                let tag_id = self.find_or_create_tag(tag_name).await?;
                self.db.add_tag_to_note(&request.id, &tag_id).await?;
            }
        }
        
        // Get updated note
        self.get_note(&request.id).await
    }
    
    pub async fn delete_note(&self, id: &str) -> Result<bool> {
        // Soft delete: mark as deleted but don't delete file
        self.db.delete_note(id).await?;
        Ok(true)
    }
    
    pub async fn restore_note(&self, id: &str) -> Result<bool> {
        // Restore from trash
        let result = sqlx::query("UPDATE notes SET is_deleted = FALSE, modified_at = ?1 WHERE id = ?2")
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(self.db.get_pool())
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn permanently_delete_note(&self, id: &str) -> Result<bool> {
        // Get note info
        if let Some(note) = self.get_note(id).await? {
            // Delete file
            self.storage.delete_note_file(&note.file_path)?;
            
            // Delete record from database (foreign key constraints will automatically delete related tag associations)
            let result = sqlx::query("DELETE FROM notes WHERE id = ?1")
                .bind(id)
                .execute(self.db.get_pool())
                .await?;
            
            Ok(result.rows_affected() > 0)
        } else {
            Ok(false)
        }
    }
    
    async fn find_or_create_tag(&self, tag_name: &str) -> Result<String> {
        // Find existing tag
        let row = sqlx::query("SELECT id FROM tags WHERE name = ?1")
            .bind(tag_name)
            .fetch_optional(self.db.get_pool())
            .await?;
        
        if let Some(row) = row {
            Ok(row.get("id"))
        } else {
            // Create new tag
            let tag_id = Uuid::new_v4().to_string();
            self.db.create_tag(&tag_id, tag_name).await?;
            Ok(tag_id)
        }
    }
    
    pub async fn sync_external_files(&self) -> Result<Vec<Note>> {
        // Scan all Markdown files in filesystem
        let file_infos = self.storage.scan_existing_files()?;
        let existing_notes = self.db.get_all_notes().await?;
        
        let mut new_notes = Vec::new();
        
        for file_info in file_infos {
            // Check if file is already in database
            let exists = existing_notes.iter()
                .any(|note| note.file_path == file_info.name);
            
            if !exists {
                // Create new database record
                let id = Uuid::new_v4().to_string();
                let title = self.extract_title_from_content(&file_info.name).unwrap_or_else(|_| {
                    // Extract title from filename
                    std::path::Path::new(&file_info.name)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });
                
                self.db.create_note(&id, &title, &file_info.name).await?;
                
                // Create Note object
                let content = self.storage.read_note_file(&file_info.name)
                    .unwrap_or_default();
                
                let now = Utc::now();
                let note = Note {
                    id,
                    title,
                    content,
                    file_path: file_info.name,
                    created_at: now,
                    modified_at: now,
                    is_favorite: false,
                    is_deleted: false,
                    tags: vec![],
                    has_attachments: false,
                };
                
                new_notes.push(note);
            }
        }
        
        Ok(new_notes)
    }
    
    fn extract_title_from_content(&self, file_name: &str) -> Result<String> {
        let content = self.storage.read_note_file(file_name)?;
        
        // Try to extract first heading from content
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                let title = line.trim_start_matches('#').trim();
                if !title.is_empty() {
                    return Ok(title.to_string());
                }
            }
        }
        
        // If no heading found, return error
        Err(anyhow::anyhow!("No title found in content"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::sync::Arc;
    
    async fn create_test_notes_manager() -> (NotesManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let notes_path = temp_dir.path().join("notes");
        
        let db = DatabaseManager::new(&db_path).await.unwrap();
        let storage = FileStorageManager::new(notes_path).unwrap();
        let manager = NotesManager::new(db, storage);
        
        (manager, temp_dir)
    }
    
    #[tokio::test]
    async fn test_create_note() {
        let (manager, _temp_dir) = create_test_notes_manager().await;
        
        let request = CreateNoteRequest {
            title: "Test Note".to_string(),
            content: Some("# Test Note\n\nThis is a test.".to_string()),
        };
        
        let note = manager.create_note(request).await.unwrap();
        
        assert_eq!(note.title, "Test Note");
        assert!(note.content.contains("This is a test"));
        assert!(!note.is_favorite);
        assert!(!note.is_deleted);
    }
    
    #[tokio::test]
    async fn test_get_all_notes() {
        let (manager, _temp_dir) = create_test_notes_manager().await;
        
        // Create a few test notes
        let request1 = CreateNoteRequest {
            title: "Note 1".to_string(),
            content: Some("Content 1".to_string()),
        };
        let request2 = CreateNoteRequest {
            title: "Note 2".to_string(),
            content: Some("Content 2".to_string()),
        };
        
        manager.create_note(request1).await.unwrap();
        manager.create_note(request2).await.unwrap();
        
        let notes = manager.get_all_notes().await.unwrap();
        assert_eq!(notes.len(), 2);
    }
    
    #[tokio::test]
    async fn test_update_note() {
        let (manager, _temp_dir) = create_test_notes_manager().await;
        
        let create_request = CreateNoteRequest {
            title: "Original Title".to_string(),
            content: Some("Original content".to_string()),
        };
        
        let note = manager.create_note(create_request).await.unwrap();
        
        let update_request = UpdateNoteRequest {
            id: note.id.clone(),
            title: Some("Updated Title".to_string()),
            content: Some("Updated content".to_string()),
            is_favorite: Some(true),
            tags: Some(vec!["test".to_string()]),
        };
        
        let updated_note = manager.update_note(update_request).await.unwrap().unwrap();
        
        assert_eq!(updated_note.title, "Updated Title");
        assert_eq!(updated_note.content, "Updated content");
        assert!(updated_note.is_favorite);
        assert_eq!(updated_note.tags.len(), 1);
        assert_eq!(updated_note.tags[0], "test");
    }
    
    #[tokio::test]
    async fn test_delete_and_restore_note() {
        let (manager, _temp_dir) = create_test_notes_manager().await;
        
        let request = CreateNoteRequest {
            title: "Test Note".to_string(),
            content: Some("Test content".to_string()),
        };
        
        let note = manager.create_note(request).await.unwrap();
        
        // Delete note
        let deleted = manager.delete_note(&note.id).await.unwrap();
        assert!(deleted);
        
        // Check note is in trash
        let trash = manager.get_trash().await.unwrap();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].id, note.id);
        
        // Restore note
        let restored = manager.restore_note(&note.id).await.unwrap();
        assert!(restored);
        
        // Check note is restored
        let notes = manager.get_all_notes().await.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note.id);
    }
}