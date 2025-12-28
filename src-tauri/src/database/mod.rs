use sqlx::{SqlitePool, Row, migrate::MigrateDatabase};
use anyhow::{Result, Context};
use std::path::Path;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: SqlitePool,
}

impl DatabaseManager {
    pub async fn new(database_path: &Path) -> Result<Self> {
        // Ensure database file exists
        if !sqlx::Sqlite::database_exists(database_path.to_str().unwrap()).await.unwrap_or(false) {
            sqlx::Sqlite::create_database(database_path.to_str().unwrap()).await
                .context("Failed to create database")?;
        }
        
        let database_url = format!("sqlite://{}", database_path.to_str().unwrap());
        let pool = SqlitePool::connect(&database_url).await
            .context("Failed to connect to database")?;
        
        let mut manager = Self { pool };
        manager.migrate().await?;
        
        Ok(manager)
    }
    
    async fn migrate(&mut self) -> Result<()> {
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
                is_deleted BOOLEAN NOT NULL DEFAULT FALSE
            )
            "#
        ).execute(&self.pool).await
            .context("Failed to create notes table")?;
        
        // Create tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL
            )
            "#
        ).execute(&self.pool).await
            .context("Failed to create tags table")?;
        
        // Create note-tags association table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS note_tags (
                note_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (note_id, tag_id),
                FOREIGN KEY (note_id) REFERENCES notes (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
            )
            "#
        ).execute(&self.pool).await
            .context("Failed to create note_tags table")?;
        
        // Create attachments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS attachments (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                mime_type TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (note_id) REFERENCES notes (id) ON DELETE CASCADE
            )
            "#
        ).execute(&self.pool).await
            .context("Failed to create attachments table")?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_title ON notes (title)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_notes_modified ON notes (modified_at)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tags_name ON tags (name)")
            .execute(&self.pool).await?;
        
        Ok(())
    }
    
    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    // Note operations
    pub async fn create_note(&self, id: &str, title: &str, file_path: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            "INSERT INTO notes (id, title, file_path, created_at, modified_at) VALUES (?1, ?2, ?3, ?4, ?5)"
        )
        .bind(id)
        .bind(title)
        .bind(file_path)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .context("Failed to create note in database")?;
        
        Ok(())
    }
    
    pub async fn update_note(&self, id: &str, title: Option<&str>, is_favorite: Option<bool>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        if let Some(title) = title {
            sqlx::query("UPDATE notes SET title = ?1, modified_at = ?2 WHERE id = ?3")
                .bind(title)
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        if let Some(is_favorite) = is_favorite {
            sqlx::query("UPDATE notes SET is_favorite = ?1, modified_at = ?2 WHERE id = ?3")
                .bind(is_favorite)
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        
        Ok(())
    }
    
    pub async fn delete_note(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query("UPDATE notes SET is_deleted = TRUE, modified_at = ?1 WHERE id = ?2")
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_all_notes(&self) -> Result<Vec<NoteRecord>> {
        let rows = sqlx::query("SELECT * FROM notes WHERE is_deleted = FALSE ORDER BY modified_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut notes = Vec::new();
        for row in rows {
            let note = NoteRecord {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                is_favorite: row.get("is_favorite"),
                is_deleted: row.get("is_deleted"),
            };
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    pub async fn get_favorite_notes(&self) -> Result<Vec<NoteRecord>> {
        let rows = sqlx::query(
            "SELECT * FROM notes WHERE is_deleted = FALSE AND is_favorite = TRUE ORDER BY modified_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut notes = Vec::new();
        for row in rows {
            let note = NoteRecord {
                id: row.get("id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
                is_favorite: row.get("is_favorite"),
                is_deleted: row.get("is_deleted"),
            };
            notes.push(note);
        }
        
        Ok(notes)
    }
    
    // Tag operations
    pub async fn create_tag(&self, id: &str, name: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query("INSERT INTO tags (id, name, created_at) VALUES (?1, ?2, ?3)")
            .bind(id)
            .bind(name)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_all_tags(&self) -> Result<Vec<TagRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT t.*, COUNT(nt.note_id) as note_count 
            FROM tags t 
            LEFT JOIN note_tags nt ON t.id = nt.tag_id
            LEFT JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            GROUP BY t.id, t.name, t.created_at
            ORDER BY t.name
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut tags = Vec::new();
        for row in rows {
            let tag = TagRecord {
                id: row.get("id"),
                name: row.get("name"),
                created_at: row.get("created_at"),
                note_count: row.get::<i64, _>("note_count") as usize,
            };
            tags.push(tag);
        }
        
        Ok(tags)
    }
    
    pub async fn add_tag_to_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id, created_at) VALUES (?1, ?2, ?3)")
            .bind(note_id)
            .bind(tag_id)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn remove_tag_from_note(&self, note_id: &str, tag_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2")
            .bind(note_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_note_tags(&self, note_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT t.name FROM tags t JOIN note_tags nt ON t.id = nt.tag_id WHERE nt.note_id = ?1"
        )
        .bind(note_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| row.get::<String, _>("name")).collect())
    }
    
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[derive(Debug, Clone)]
pub struct NoteRecord {
    pub id: String,
    pub title: String,
    pub file_path: String,
    pub created_at: String,
    pub modified_at: String,
    pub is_favorite: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Clone)]
pub struct TagRecord {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub note_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::test;
    
    #[test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = DatabaseManager::new(&db_path).await.unwrap();
        assert!(db_path.exists());
        
        db.close().await;
    }
    
    #[test]
    async fn test_note_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseManager::new(&db_path).await.unwrap();
        
        let note_id = uuid::Uuid::new_v4().to_string();
        let title = "Test Note";
        let file_path = "test.md";
        
        // Create note
        db.create_note(&note_id, title, file_path).await.unwrap();
        
        // Get all notes
        let notes = db.get_all_notes().await.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, title);
        
        // Update note
        db.update_note(&note_id, Some("Updated Title"), Some(true)).await.unwrap();
        
        // Get favorite notes
        let favorites = db.get_favorite_notes().await.unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].title, "Updated Title");
        
        db.close().await;
    }
    
    #[test]
    async fn test_tag_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseManager::new(&db_path).await.unwrap();
        
        let note_id = uuid::Uuid::new_v4().to_string();
        let tag_id = uuid::Uuid::new_v4().to_string();
        
        // Create note and tag
        db.create_note(&note_id, "Test Note", "test.md").await.unwrap();
        db.create_tag(&tag_id, "test-tag").await.unwrap();
        
        // Associate tag and note
        db.add_tag_to_note(&note_id, &tag_id).await.unwrap();
        
        // Get note tags
        let tags = db.get_note_tags(&note_id).await.unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0], "test-tag");
        
        // Get all tags
        let all_tags = db.get_all_tags().await.unwrap();
        assert_eq!(all_tags.len(), 1);
        assert_eq!(all_tags[0].note_count, 1);
        
        db.close().await;
    }
}