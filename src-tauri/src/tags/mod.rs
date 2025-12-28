use crate::storage::FileStorageManager;
use crate::models::Tag;
use anyhow::Result;
use std::collections::HashMap;

pub struct TagsManager {
    storage: FileStorageManager,
}

impl TagsManager {
    pub fn new(storage: FileStorageManager) -> Self {
        Self { storage }
    }
    
    pub async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let files = self.storage.scan_existing_files()?;
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        
        for file in files {
             if let Ok((metadata, _)) = self.storage.parse_note(&file.name) {
                 if !metadata.deleted {
                     for tag in metadata.tags {
                         *tag_counts.entry(tag).or_insert(0) += 1;
                     }
                 }
             }
        }
        
        let mut tags: Vec<Tag> = tag_counts.into_iter().map(|(name, count)| Tag {
            id: name.clone(),
            name,
            note_count: count,
        }).collect();
        
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(tags)
    }
    
    pub async fn create_tag(&self, name: &str) -> Result<Tag> {
        Ok(Tag {
            id: name.to_string(),
            name: name.to_string(),
            note_count: 0
        })
    }
    
    pub async fn rename_tag(&self, tag_id: &str, new_name: &str) -> Result<Option<Tag>> {
        let files = self.storage.scan_existing_files()?;
        let mut updated = false;
        
        for file in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file.name) {
                 if metadata.tags.contains(&tag_id.to_string()) {
                     metadata.tags = metadata.tags.into_iter().map(|t| if t == tag_id { new_name.to_string() } else { t }).collect();
                     self.storage.save_note(&file.name, &metadata, &content)?;
                     updated = true;
                 }
             }
        }
        
        if updated {
            Ok(Some(Tag { id: new_name.to_string(), name: new_name.to_string(), note_count: 0 }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn delete_tag(&self, tag_id: &str) -> Result<bool> {
        let files = self.storage.scan_existing_files()?;
        let mut deleted = false;
        
        for file in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file.name) {
                 if metadata.tags.contains(&tag_id.to_string()) {
                     metadata.tags.retain(|t| t != tag_id);
                     self.storage.save_note(&file.name, &metadata, &content)?;
                     deleted = true;
                 }
             }
        }
        Ok(deleted)
    }
    
    pub async fn add_tag_to_note(&self, note_id: &str, tag_name: &str) -> Result<Tag> {
        let files = self.storage.scan_existing_files()?;
        for file in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file.name) {
                 if metadata.id == note_id {
                     if !metadata.tags.contains(&tag_name.to_string()) {
                         metadata.tags.push(tag_name.to_string());
                         self.storage.save_note(&file.name, &metadata, &content)?;
                     }
                     return Ok(Tag { id: tag_name.to_string(), name: tag_name.to_string(), note_count: 1 });
                 }
             }
        }
        Err(anyhow::anyhow!("Note not found"))
    }
    
    pub async fn remove_tag_from_note(&self, note_id: &str, tag_name: &str) -> Result<bool> {
        let files = self.storage.scan_existing_files()?;
        for file in files {
             if let Ok((mut metadata, content)) = self.storage.parse_note(&file.name) {
                 if metadata.id == note_id {
                     if metadata.tags.contains(&tag_name.to_string()) {
                         metadata.tags.retain(|t| t != tag_name);
                         self.storage.save_note(&file.name, &metadata, &content)?;
                         return Ok(true);
                     }
                     return Ok(false);
                 }
             }
        }
        Ok(false)
    }
    
    pub async fn get_note_tags(&self, note_id: &str) -> Result<Vec<Tag>> {
        let files = self.storage.scan_existing_files()?;
        for file in files {
             if let Ok((metadata, _)) = self.storage.parse_note(&file.name) {
                 if metadata.id == note_id {
                     return Ok(metadata.tags.into_iter().map(|t| Tag {
                         id: t.clone(),
                         name: t,
                         note_count: 1
                     }).collect());
                 }
             }
        }
        Ok(vec![])
    }
    
    pub async fn cleanup_unused_tags(&self) -> Result<usize> {
        Ok(0)
    }
    
    pub async fn search_tags(&self, query: &str) -> Result<Vec<Tag>> {
         let all_tags = self.get_all_tags().await?;
         Ok(all_tags.into_iter().filter(|t| t.name.to_lowercase().contains(&query.to_lowercase())).collect())
    }
}
