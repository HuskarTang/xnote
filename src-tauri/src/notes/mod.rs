use crate::database::{DatabaseManager, NoteRecord};
use crate::storage::FileStorageManager;
use crate::models::{Note, CreateNoteRequest, UpdateNoteRequest};
use anyhow::{Result, Context};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::path::Path;

pub struct NotesManager {
    db: DatabaseManager,
    storage: FileStorageManager,
}

impl NotesManager {
    pub fn new(db: DatabaseManager, storage: FileStorageManager) -> Self {
        Self { db, storage }
    }

    pub fn get_database_manager(&self) -> &DatabaseManager {
        &self.db
    }
    
    pub async fn create_note(&self, request: CreateNoteRequest) -> Result<Note> {
        let id = Uuid::new_v4().to_string();
        let title = if request.title.trim().is_empty() {
            "Untitled".to_string()
        } else {
            request.title.trim().to_string()
        };
        
        let content = request.content.unwrap_or_else(|| "edit your Note here with Markdown...".to_string());
        
        // Create file - this will return the actual filename with potential (n) suffix
        let file_name = self.storage.create_note_file(&title, &content)
            .context("Failed to create note file")?;
        
        // Extract actual title from filename (remove .md extension and use as title)
        let actual_title = file_name.strip_suffix(".md")
            .unwrap_or(&file_name)
            .to_string();
        
        // Create record in database with the actual title
        self.db.create_note(&id, &actual_title, &file_name).await
            .context("Failed to create note in database")?;
        
        let now = Utc::now();
        let note = Note {
            id: id.clone(),
            title: actual_title,  // Use the actual title with (n) suffix
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
        self.get_note_including_deleted(id, false).await
    }
    
    // Helper method to get note including deleted ones
    async fn get_note_including_deleted(&self, id: &str, include_deleted: bool) -> Result<Option<Note>> {
        let query = if include_deleted {
            "SELECT * FROM notes WHERE id = ?1"
        } else {
            "SELECT * FROM notes WHERE id = ?1 AND is_deleted = FALSE"
        };
        
        let row = sqlx::query(query)
            .bind(id)
            .fetch_optional(self.db.get_pool())
            .await?;
            
        if let Some(row) = row {
            let record = crate::database::NoteRecord {
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
            
            // Check for attachments
            let has_attachments = self.db.check_has_attachments(&record.id).await.unwrap_or(false);
            
            let created_at = DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            let modified_at = DateTime::parse_from_rfc3339(&record.modified_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            
            Ok(Some(Note {
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
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_all_notes(&self) -> Result<Vec<Note>> {
        let records = self.db.get_all_notes().await?;
        let mut notes = Vec::new();
        
        for record in records {
            let content = self.storage.read_note_file(&record.file_path)
                .unwrap_or_else(|_| "Failed to load content".to_string());
            
            let tags = self.db.get_note_tags(&record.id).await?;
            
            // Check for attachments
            let has_attachments = self.db.check_has_attachments(&record.id).await.unwrap_or(false);
            
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
            let has_attachments = self.db.check_has_attachments(&record.id).await.unwrap_or(false);
            
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
            let has_attachments = self.db.check_has_attachments(&record.id).await.unwrap_or(false);
            
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
            let has_attachments = self.db.check_has_attachments(&record.id).await.unwrap_or(false);
            
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
                
                // Extract actual title from new filename (remove .md extension)
                let actual_title = new_file_name.strip_suffix(".md")
                    .unwrap_or(&new_file_name)
                    .to_string();
                record.title = actual_title;
            }
        }
        
        // Update file content
        let content_updated = if let Some(ref content) = request.content {
            self.storage.update_note_file(&new_file_name, content)
                .context("Failed to update note content")?;
            true
        } else {
            false
        };
        
        // Update database record
        // Always update the modified time, even if only content changed
        let needs_db_update = request.title.is_some() || request.is_favorite.is_some() || content_updated;
        if needs_db_update {
            // Use the actual title (which may have been modified due to file conflicts)
            let title_to_update = if file_renamed { 
                Some(record.title.as_str())
            } else { 
                request.title.as_deref() 
            };
            
            self.db.update_note(
                &request.id,
                title_to_update,
                request.is_favorite,
            ).await?;
        }
        
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
        println!("DEBUG: Starting permanent deletion for note id: {}", id);
        
        // Get note info - including deleted notes
        if let Some(note) = self.get_note_including_deleted(id, true).await? {
            println!("DEBUG: Found note to delete: {} (file: {}, is_deleted: {})", note.title, note.file_path, note.is_deleted);
            
            // Delete file
            match self.storage.delete_note_file(&note.file_path) {
                Ok(_) => println!("DEBUG: Successfully deleted file: {}", note.file_path),
                Err(e) => {
                    println!("DEBUG: Failed to delete file {}: {}", note.file_path, e);
                    return Err(e);
                }
            }
            
            // Delete record from database (foreign key constraints will automatically delete related tag associations)
            let result = sqlx::query("DELETE FROM notes WHERE id = ?1")
                .bind(id)
                .execute(self.db.get_pool())
                .await?;
            
            println!("DEBUG: Database deletion affected {} rows", result.rows_affected());
            Ok(result.rows_affected() > 0)
        } else {
            println!("DEBUG: Note with id {} not found", id);
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
    
    pub async fn search_notes(&self, query: &str, tag_filter: Option<&str>) -> Result<Vec<Note>> {
        // Get base notes to search from based on tag filter
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
            // Search in title and content
            let title_matches = note.title.to_lowercase().contains(&query_lower);
            let content_matches = note.content.to_lowercase().contains(&query_lower);
            
            if title_matches || content_matches {
                matching_notes.push(note);
            }
        }
        
        // Sort by relevance: title matches first, then by modification date
        matching_notes.sort_by(|a, b| {
            let a_title_match = a.title.to_lowercase().contains(&query_lower);
            let b_title_match = b.title.to_lowercase().contains(&query_lower);
            
            match (a_title_match, b_title_match) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => b.modified_at.cmp(&a.modified_at), // Most recent first
            }
        });
        
        Ok(matching_notes)
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
                
                // Extract title from filename or content
                let title = Path::new(&file_info.name)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
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
    
    #[tokio::test]
    async fn test_sync_external_files() {
        let (manager, temp_dir) = create_test_notes_manager().await;
        
        // Create a test Markdown file manually (simulating external migration)
        let notes_dir = temp_dir.path().join("notes");
        std::fs::create_dir_all(&notes_dir).unwrap();
        let test_file = notes_dir.join("external_note.md");
        std::fs::write(&test_file, "# External Note\n\nThis note was added externally.").unwrap();
        
        // Sync external files
        let new_notes = manager.sync_external_files().await.unwrap();
        
        assert_eq!(new_notes.len(), 1);
        assert_eq!(new_notes[0].title, "external_note");
        assert_eq!(new_notes[0].file_path, "external_note.md");
        
        // Verify the note was added to database
        let all_notes = manager.get_all_notes().await.unwrap();
        assert_eq!(all_notes.len(), 1);
        assert_eq!(all_notes[0].title, "external_note");
    }
}