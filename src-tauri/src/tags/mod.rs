use crate::database::{DatabaseManager};
use crate::models::Tag;
use anyhow::Result;
use uuid::Uuid;
use sqlx::Row;

pub struct TagsManager {
    db: DatabaseManager,
}

impl TagsManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
    
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let records = self.db.get_all_tags().await?;
        
        let tags = records.into_iter().map(|record| Tag {
            id: record.id,
            name: record.name,
            note_count: record.note_count,
        }).collect();
        
        Ok(tags)
    }
    
    pub async fn create_tag(&self, name: &str) -> Result<Tag> {
        // Check if tag already exists
        if let Some(existing_tag) = self.find_tag_by_name(name).await? {
            return Ok(existing_tag);
        }
        
        let id = Uuid::new_v4().to_string();
        self.db.create_tag(&id, name).await?;
        
        Ok(Tag {
            id,
            name: name.to_string(),
            note_count: 0,
        })
    }
    
    pub async fn find_tag_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let row = sqlx::query(
            r#"
            SELECT t.*, COUNT(nt.note_id) as note_count 
            FROM tags t 
            LEFT JOIN note_tags nt ON t.id = nt.tag_id
            LEFT JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            WHERE t.name = ?1
            GROUP BY t.id, t.name, t.created_at
            "#
        )
        .bind(name)
        .fetch_optional(self.db.get_pool())
        .await?;
        
        if let Some(row) = row {
            Ok(Some(Tag {
                id: row.get("id"),
                name: row.get("name"),
                note_count: row.get::<i64, _>("note_count") as usize,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn find_tag_by_id(&self, id: &str) -> Result<Option<Tag>> {
        let row = sqlx::query(
            r#"
            SELECT t.*, COUNT(nt.note_id) as note_count 
            FROM tags t 
            LEFT JOIN note_tags nt ON t.id = nt.tag_id
            LEFT JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            WHERE t.id = ?1
            GROUP BY t.id, t.name, t.created_at
            "#
        )
        .bind(id)
        .fetch_optional(self.db.get_pool())
        .await?;
        
        if let Some(row) = row {
            Ok(Some(Tag {
                id: row.get("id"),
                name: row.get("name"),
                note_count: row.get::<i64, _>("note_count") as usize,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn rename_tag(&self, tag_id: &str, new_name: &str) -> Result<Option<Tag>> {
        // Check if new name already exists
        if let Some(_existing) = self.find_tag_by_name(new_name).await? {
            return Err(anyhow::anyhow!("Tag with name '{}' already exists", new_name));
        }
        
        let result = sqlx::query("UPDATE tags SET name = ?1 WHERE id = ?2")
            .bind(new_name)
            .bind(tag_id)
            .execute(self.db.get_pool())
            .await?;
        
        if result.rows_affected() > 0 {
            self.find_tag_by_id(tag_id).await
        } else {
            Ok(None)
        }
    }
    
    pub async fn delete_tag(&self, tag_id: &str) -> Result<bool> {
        // When deleting tag, all note-tag associations will also be deleted (handled by foreign key constraints)
        let result = sqlx::query("DELETE FROM tags WHERE id = ?1")
            .bind(tag_id)
            .execute(self.db.get_pool())
            .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn add_tag_to_note(&self, note_id: &str, tag_name: &str) -> Result<Tag> {
        // Find or create tag
        let tag = if let Some(existing_tag) = self.find_tag_by_name(tag_name).await? {
            existing_tag
        } else {
            self.create_tag(tag_name).await?
        };
        
        // Add tag to note
        self.db.add_tag_to_note(note_id, &tag.id).await?;
        
        // Re-get tag to update count
        Ok(self.find_tag_by_id(&tag.id).await?.unwrap_or(tag))
    }
    
    pub async fn remove_tag_from_note(&self, note_id: &str, tag_name: &str) -> Result<bool> {
        // Find tag ID
        if let Some(tag) = self.find_tag_by_name(tag_name).await? {
            self.db.remove_tag_from_note(note_id, &tag.id).await?;
            
            // Check if tag is still used by other notes, delete if not
            if let Some(updated_tag) = self.find_tag_by_id(&tag.id).await? {
                if updated_tag.note_count == 0 {
                    self.delete_tag(&tag.id).await?;
                }
            }
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub async fn get_note_tags(&self, note_id: &str) -> Result<Vec<Tag>> {
        let rows = sqlx::query(
            r#"
            SELECT t.*, 1 as note_count 
            FROM tags t 
            JOIN note_tags nt ON t.id = nt.tag_id 
            WHERE nt.note_id = ?1
            ORDER BY t.name
            "#
        )
        .bind(note_id)
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut tags = Vec::new();
        for row in rows {
            let tag = Tag {
                id: row.get("id"),
                name: row.get("name"),
                note_count: row.get::<i64, _>("note_count") as usize,
            };
            tags.push(tag);
        }
        
        Ok(tags)
    }
    
    pub async fn get_notes_with_tag(&self, tag_name: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT n.id 
            FROM notes n
            JOIN note_tags nt ON n.id = nt.note_id
            JOIN tags t ON nt.tag_id = t.id
            WHERE t.name = ?1 AND n.is_deleted = FALSE
            ORDER BY n.modified_at DESC
            "#
        )
        .bind(tag_name)
        .fetch_all(self.db.get_pool())
        .await?;
        
        Ok(rows.into_iter().map(|row| row.get::<String, _>("id")).collect())
    }
    
    pub async fn get_tag_statistics(&self) -> Result<TagStatistics> {
        let total_tags = sqlx::query("SELECT COUNT(*) as count FROM tags")
            .fetch_one(self.db.get_pool())
            .await?
            .get::<i64, _>("count") as usize;
        
        let tags_with_notes = sqlx::query(
            r#"
            SELECT COUNT(DISTINCT t.id) as count 
            FROM tags t
            JOIN note_tags nt ON t.id = nt.tag_id
            JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            "#
        )
        .fetch_one(self.db.get_pool())
        .await?
        .get::<i64, _>("count") as usize;
        
        let unused_tags = total_tags - tags_with_notes;
        
        let most_used_tags = self.get_most_used_tags(5).await?;
        
        Ok(TagStatistics {
            total_tags,
            tags_with_notes,
            unused_tags,
            most_used_tags,
        })
    }
    
    pub async fn get_most_used_tags(&self, limit: usize) -> Result<Vec<Tag>> {
        let rows = sqlx::query(
            r#"
            SELECT t.*, COUNT(nt.note_id) as note_count 
            FROM tags t 
            LEFT JOIN note_tags nt ON t.id = nt.tag_id
            LEFT JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            GROUP BY t.id, t.name, t.created_at
            HAVING note_count > 0
            ORDER BY note_count DESC, t.name ASC
            LIMIT ?1
            "#
        )
        .bind(limit as i64)
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut tags = Vec::new();
        for row in rows {
            let tag = Tag {
                id: row.get("id"),
                name: row.get("name"),
                note_count: row.get::<i64, _>("note_count") as usize,
            };
            tags.push(tag);
        }
        
        Ok(tags)
    }
    
    pub async fn search_tags(&self, query: &str) -> Result<Vec<Tag>> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        
        let rows = sqlx::query(
            r#"
            SELECT t.*, COUNT(nt.note_id) as note_count 
            FROM tags t 
            LEFT JOIN note_tags nt ON t.id = nt.tag_id
            LEFT JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            WHERE LOWER(t.name) LIKE ?1
            GROUP BY t.id, t.name, t.created_at
            ORDER BY t.name ASC
            "#
        )
        .bind(&search_pattern)
        .fetch_all(self.db.get_pool())
        .await?;
        
        let mut tags = Vec::new();
        for row in rows {
            let tag = Tag {
                id: row.get("id"),
                name: row.get("name"),
                note_count: row.get::<i64, _>("note_count") as usize,
            };
            tags.push(tag);
        }
        
        Ok(tags)
    }
    
    pub async fn cleanup_unused_tags(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM tags 
            WHERE id NOT IN (
                SELECT DISTINCT nt.tag_id 
                FROM note_tags nt
                JOIN notes n ON nt.note_id = n.id AND n.is_deleted = FALSE
            )
            "#
        )
        .execute(self.db.get_pool())
        .await?;
        
        Ok(result.rows_affected() as usize)
    }
    
    pub async fn merge_tags(&self, source_tag_id: &str, target_tag_id: &str) -> Result<Tag> {
        // Transfer all associations from source tag to target tag
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO note_tags (note_id, tag_id, created_at)
            SELECT nt.note_id, ?2, nt.created_at
            FROM note_tags nt
            WHERE nt.tag_id = ?1
            "#
        )
        .bind(source_tag_id)
        .bind(target_tag_id)
        .execute(self.db.get_pool())
        .await?;
        
        // Delete source tag
        self.delete_tag(source_tag_id).await?;
        
        // Return updated target tag
        self.find_tag_by_id(target_tag_id).await?
            .ok_or_else(|| anyhow::anyhow!("Target tag not found after merge"))
    }
}

#[derive(Debug, Clone)]
pub struct TagStatistics {
    pub total_tags: usize,
    pub tags_with_notes: usize,
    pub unused_tags: usize,
    pub most_used_tags: Vec<Tag>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    async fn create_test_tags_manager() -> (TagsManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseManager::new(&db_path).await.unwrap();
        let manager = TagsManager::new(db);
        
        (manager, temp_dir)
    }
    
    #[tokio::test]
    async fn test_create_and_find_tag() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        let tag = manager.create_tag("test-tag").await.unwrap();
        assert_eq!(tag.name, "test-tag");
        assert_eq!(tag.note_count, 0);
        
        let found_tag = manager.find_tag_by_name("test-tag").await.unwrap().unwrap();
        assert_eq!(found_tag.id, tag.id);
        assert_eq!(found_tag.name, "test-tag");
    }
    
    #[tokio::test]
    async fn test_duplicate_tag_creation() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        let tag1 = manager.create_tag("duplicate").await.unwrap();
        let tag2 = manager.create_tag("duplicate").await.unwrap();
        
        // Should return the same tag
        assert_eq!(tag1.id, tag2.id);
        assert_eq!(tag1.name, tag2.name);
    }
    
    #[tokio::test]
    async fn test_rename_tag() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        let tag = manager.create_tag("original-name").await.unwrap();
        let renamed_tag = manager.rename_tag(&tag.id, "new-name").await.unwrap().unwrap();
        
        assert_eq!(renamed_tag.id, tag.id);
        assert_eq!(renamed_tag.name, "new-name");
        
        // Original name should not be found
        let not_found = manager.find_tag_by_name("original-name").await.unwrap();
        assert!(not_found.is_none());
    }
    
    #[tokio::test]
    async fn test_delete_tag() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        let tag = manager.create_tag("to-delete").await.unwrap();
        let deleted = manager.delete_tag(&tag.id).await.unwrap();
        
        assert!(deleted);
        
        let not_found = manager.find_tag_by_id(&tag.id).await.unwrap();
        assert!(not_found.is_none());
    }
    
    #[tokio::test]
    async fn test_search_tags() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        manager.create_tag("programming").await.unwrap();
        manager.create_tag("programming-rust").await.unwrap();
        manager.create_tag("design").await.unwrap();
        
        let results = manager.search_tags("program").await.unwrap();
        assert_eq!(results.len(), 2);
        
        let names: Vec<String> = results.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"programming".to_string()));
        assert!(names.contains(&"programming-rust".to_string()));
    }
    
    #[tokio::test]
    async fn test_get_all_tags() {
        let (manager, _temp_dir) = create_test_tags_manager().await;
        
        manager.create_tag("tag1").await.unwrap();
        manager.create_tag("tag2").await.unwrap();
        manager.create_tag("tag3").await.unwrap();
        
        let all_tags = manager.get_all_tags().await.unwrap();
        assert_eq!(all_tags.len(), 3);
    }
}