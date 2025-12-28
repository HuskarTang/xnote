use anyhow::{Result, Context};
use sqlx::{SqlitePool, Row};
use chrono::{DateTime, Utc};
use std::path::Path;

use crate::models::{Note, Tag, NoteTag, NoteWithTags};

pub struct Database {
    pool: Option<SqlitePool>,
}

impl Database {
    pub fn new() -> Result<Self> {
        Ok(Self { pool: None })
    }
    
    pub async fn initialize(&mut self, db_path: &Path) -> Result<()> {
        let database_url = format!("sqlite:{}", db_path.display());
        
        // Create database file if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let pool = SqlitePool::connect(&database_url).await
            .context("Failed to connect to database")?;
        
        // Run migrations
        self.run_migrations(&pool).await?;
        
        self.pool = Some(pool);
        Ok(())
    }
    
    async fn run_migrations(&self, pool: &SqlitePool) -> Result<()> {
        // Create notes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                is_trashed BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // Create tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // Create note_tags junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (note_id, tag_id),
                FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_is_trashed ON notes(is_trashed)")
            .execute(pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)")
            .execute(pool)
            .await?;
        
        Ok(())
    }
    
    fn get_pool(&self) -> Result<&SqlitePool> {
        self.pool.as_ref().context("Database not initialized")
    }
    
    pub async fn create_note(&self, note: &Note) -> Result<()> {
        let pool = self.get_pool()?;
        
        sqlx::query(
            "INSERT INTO notes (id, title, file_path, created_at, modified_at, is_favorite, is_trashed) 
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&note.id)
        .bind(&note.title)
        .bind(&note.file_path)
        .bind(note.created_at.to_rfc3339())
        .bind(note.modified_at.to_rfc3339())
        .bind(note.is_favorite)
        .bind(note.is_trashed)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_note(&self, note: &Note) -> Result<()> {
        let pool = self.get_pool()?;
        
        sqlx::query(
            "UPDATE notes SET title = ?, modified_at = ?, is_favorite = ?, is_trashed = ? WHERE id = ?"
        )
        .bind(&note.title)
        .bind(note.modified_at.to_rfc3339())
        .bind(note.is_favorite)
        .bind(note.is_trashed)
        .bind(&note.id)
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_note_by_id(&self, id: &str) -> Result<Option<Note>> {
        let pool = self.get_pool()?;
        
        let row = sqlx::query("SELECT * FROM notes WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Note {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                modified_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("modified_at"))?.with_timezone(&Utc),
                is_favorite: row.get("is_favorite"),
                is_trashed: row.get("is_trashed"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_all_notes(&self, include_trashed: bool) -> Result<Vec<NoteWithTags>> {
        let pool = self.get_pool()?;
        
        let query = if include_trashed {
            "SELECT * FROM notes ORDER BY modified_at DESC"
        } else {
            "SELECT * FROM notes WHERE is_trashed = FALSE ORDER BY modified_at DESC"
        };
        
        let rows = sqlx::query(query).fetch_all(pool).await?;
        
        let mut notes = Vec::new();
        for row in rows {
            let note = Note {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                modified_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("modified_at"))?.with_timezone(&Utc),
                is_favorite: row.get("is_favorite"),
                is_trashed: row.get("is_trashed"),
            };
            
            let tags = self.get_tags_for_note(&note.id).await?;
            notes.push(NoteWithTags { note, tags });
        }
        
        Ok(notes)
    }
    
    pub async fn get_notes_by_tag(&self, tag_name: &str) -> Result<Vec<NoteWithTags>> {
        let pool = self.get_pool()?;
        
        let rows = sqlx::query(
            "SELECT n.* FROM notes n 
             JOIN note_tags nt ON n.id = nt.note_id 
             JOIN tags t ON nt.tag_id = t.id 
             WHERE t.name = ? AND n.is_trashed = FALSE 
             ORDER BY n.modified_at DESC"
        )
        .bind(tag_name)
        .fetch_all(pool)
        .await?;
        
        let mut notes = Vec::new();
        for row in rows {
            let note = Note {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                modified_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("modified_at"))?.with_timezone(&Utc),
                is_favorite: row.get("is_favorite"),
                is_trashed: row.get("is_trashed"),
            };
            
            let tags = self.get_tags_for_note(&note.id).await?;
            notes.push(NoteWithTags { note, tags });
        }
        
        Ok(notes)
    }
    
    pub async fn search_notes(&self, query: &str) -> Result<Vec<NoteWithTags>> {
        let pool = self.get_pool()?;
        let search_pattern = format!("%{}%", query);
        
        let rows = sqlx::query(
            "SELECT * FROM notes WHERE (title LIKE ? OR file_path LIKE ?) AND is_trashed = FALSE ORDER BY modified_at DESC"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .fetch_all(pool)
        .await?;
        
        let mut notes = Vec::new();
        for row in rows {
            let note = Note {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
                modified_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("modified_at"))?.with_timezone(&Utc),
                is_favorite: row.get("is_favorite"),
                is_trashed: row.get("is_trashed"),
            };
            
            let tags = self.get_tags_for_note(&note.id).await?;
            notes.push(NoteWithTags { note, tags });
        }
        
        Ok(notes)
    }
    
    pub async fn delete_note(&self, id: &str) -> Result<()> {
        let pool = self.get_pool()?;
        
        // Delete note_tags first (foreign key constraint)
        sqlx::query("DELETE FROM note_tags WHERE note_id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        
        // Delete note
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        
        Ok(())
    }
    
    // Tag operations
    pub async fn create_tag(&self, tag: &Tag) -> Result<()> {
        let pool = self.get_pool()?;
        
        sqlx::query(
            "INSERT OR IGNORE INTO tags (id, name, color, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind(&tag.id)
        .bind(&tag.name)
        .bind(&tag.color)
        .bind(tag.created_at.to_rfc3339())
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let pool = self.get_pool()?;
        
        let row = sqlx::query("SELECT * FROM tags WHERE name = ?")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        
        if let Some(row) = row {
            Ok(Some(Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let pool = self.get_pool()?;
        
        let rows = sqlx::query("SELECT * FROM tags ORDER BY name").fetch_all(pool).await?;
        
        let mut tags = Vec::new();
        for row in rows {
            tags.push(Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
            });
        }
        
        Ok(tags)
    }
    
    pub async fn get_tags_for_note(&self, note_id: &str) -> Result<Vec<Tag>> {
        let pool = self.get_pool()?;
        
        let rows = sqlx::query(
            "SELECT t.* FROM tags t 
             JOIN note_tags nt ON t.id = nt.tag_id 
             WHERE nt.note_id = ? 
             ORDER BY t.name"
        )
        .bind(note_id)
        .fetch_all(pool)
        .await?;
        
        let mut tags = Vec::new();
        for row in rows {
            tags.push(Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))?.with_timezone(&Utc),
            });
        }
        
        Ok(tags)
    }
    
    pub async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let pool = self.get_pool()?;
        
        sqlx::query(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id, created_at) VALUES (?, ?, ?)"
        )
        .bind(note_id)
        .bind(tag_id)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let pool = self.get_pool()?;
        
        sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
            .bind(note_id)
            .bind(tag_id)
            .execute(pool)
            .await?;
        
        Ok(())
    }
}