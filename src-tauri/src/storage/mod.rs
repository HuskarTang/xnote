use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};

#[derive(Clone)]
pub struct FileStorageManager {
    notes_directory: PathBuf,
}

impl FileStorageManager {
    pub fn new(notes_directory: PathBuf) -> Result<Self> {
        // Ensure notes directory exists
        fs::create_dir_all(&notes_directory)
            .context("Failed to create notes directory")?;
        
        Ok(Self {
            notes_directory,
        })
    }
    
    pub fn create_note_file(&self, title: &str, content: &str) -> Result<String> {
        let file_path = self.generate_unique_file_path(title)?;
        
        fs::write(&file_path, content)
            .context("Failed to write note file")?;
        
        Ok(file_path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string())
    }
    
    pub fn read_note_file(&self, file_name: &str) -> Result<String> {
        let file_path = self.notes_directory.join(file_name);
        
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Note file does not exist: {}", file_name));
        }
        
        fs::read_to_string(&file_path)
            .context("Failed to read note file")
    }
    
    pub fn update_note_file(&self, file_name: &str, content: &str) -> Result<()> {
        let file_path = self.notes_directory.join(file_name);
        
        if !file_path.exists() {
            return Err(anyhow::anyhow!("Note file does not exist: {}", file_name));
        }
        
        fs::write(&file_path, content)
            .context("Failed to update note file")
    }
    
    pub fn rename_note_file(&self, old_file_name: &str, new_title: &str) -> Result<String> {
        let old_path = self.notes_directory.join(old_file_name);
        
        if !old_path.exists() {
            return Err(anyhow::anyhow!("Note file does not exist: {}", old_file_name));
        }
        
        let new_file_path = self.generate_unique_file_path(new_title)?;
        
        fs::rename(&old_path, &new_file_path)
            .context("Failed to rename note file")?;
        
        Ok(new_file_path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string())
    }
    
    pub fn delete_note_file(&self, file_name: &str) -> Result<()> {
        let file_path = self.notes_directory.join(file_name);
        
        if file_path.exists() {
            fs::remove_file(&file_path)
                .context("Failed to delete note file")?;
        }
        
        Ok(())
    }
    
    pub fn scan_existing_files(&self) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        
        let entries = fs::read_dir(&self.notes_directory)
            .context("Failed to read notes directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let metadata = fs::metadata(&path)
                    .context("Failed to read file metadata")?;
                
                let file_name = path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                let file_info = FileInfo {
                    name: file_name,
                    size: metadata.len(),
                    created: metadata.created().ok(),
                    modified: metadata.modified().ok(),
                };
                
                files.push(file_info);
            }
        }
        
        files.sort_by(|a, b| {
            b.modified.cmp(&a.modified)
        });
        
        Ok(files)
    }
    
    pub fn get_file_content_preview(&self, file_name: &str, max_chars: usize) -> Result<String> {
        let content = self.read_note_file(file_name)?;
        
        if content.len() <= max_chars {
            Ok(content)
        } else {
            Ok(content.chars().take(max_chars).collect::<String>() + "...")
        }
    }
    
    pub fn search_files_content(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let files = self.scan_existing_files()?;
        
        for file in files {
            let content = match self.read_note_file(&file.name) {
                Ok(content) => content,
                Err(_) => continue,
            };
            
            let title = self.extract_title_from_file_name(&file.name);
            
            // Search in title and content
            if title.to_lowercase().contains(&query.to_lowercase()) ||
               content.to_lowercase().contains(&query.to_lowercase()) {
                
                let preview = self.generate_search_preview(&content, query, 100);
                
                results.push(SearchResult {
                    file_name: file.name,
                    title: title.clone(),
                    preview,
                    relevance_score: self.calculate_relevance(&title, &content, query),
                });
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(results)
    }
    
    fn generate_unique_file_path(&self, title: &str) -> Result<PathBuf> {
        let base_name = self.sanitize_filename(title);
        let mut file_path = self.notes_directory.join(format!("{}.md", base_name));
        
        if !file_path.exists() {
            return Ok(file_path);
        }
        
        // If file already exists, add number suffix
        let mut counter = 1;
        loop {
            file_path = self.notes_directory.join(format!("{}({}).md", base_name, counter));
            if !file_path.exists() {
                return Ok(file_path);
            }
            counter += 1;
            
            if counter > 1000 {
                return Err(anyhow::anyhow!("Failed to generate unique file path"));
            }
        }
    }
    
    fn sanitize_filename(&self, title: &str) -> String {
        let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        let mut sanitized = String::new();
        
        for char in title.chars() {
            if invalid_chars.contains(&char) {
                sanitized.push('_');
            } else if char.is_control() {
                sanitized.push('_');
            } else {
                sanitized.push(char);
            }
        }
        
        if sanitized.trim().is_empty() {
            sanitized = "Untitled".to_string();
        }
        
        // Truncate long filenames
        if sanitized.len() > 100 {
            sanitized.truncate(100);
        }
        
        sanitized.trim().to_string()
    }
    
    fn extract_title_from_file_name(&self, file_name: &str) -> String {
        Path::new(file_name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
    
    fn generate_search_preview(&self, content: &str, query: &str, max_chars: usize) -> String {
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(max_chars / 2);
            let end = (pos + query.len() + max_chars / 2).min(content.len());
            
            let preview = &content[start..end];
            
            if start > 0 {
                format!("...{}", preview)
            } else if end < content.len() {
                format!("{}...", preview)
            } else {
                preview.to_string()
            }
        } else {
            // If query word not found, return beginning part
            if content.len() <= max_chars {
                content.to_string()
            } else {
                format!("{}...", &content[..max_chars])
            }
        }
    }
    
    fn calculate_relevance(&self, title: &str, content: &str, query: &str) -> f64 {
        let query_lower = query.to_lowercase();
        let title_lower = title.to_lowercase();
        let content_lower = content.to_lowercase();
        
        let mut score = 0.0;
        
        // Title match scores higher
        if title_lower.contains(&query_lower) {
            score += 10.0;
            
            // Exact match scores highest
            if title_lower == query_lower {
                score += 50.0;
            }
        }
        
        // Content match
        let content_matches = content_lower.matches(&query_lower).count();
        score += content_matches as f64;
        
        score
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_name: String,
    pub title: String,
    pub preview: String,
    pub relevance_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        assert!(temp_dir.path().exists());
    }
    
    #[test]
    fn test_note_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let title = "Test Note";
        let content = "# Test Note\n\nThis is a test note.";
        
        // Create file
        let file_name = storage.create_note_file(title, content).unwrap();
        assert!(file_name.ends_with(".md"));
        
        // Read file
        let read_content = storage.read_note_file(&file_name).unwrap();
        assert_eq!(read_content, content);
        
        // Update file
        let new_content = "# Updated Test Note\n\nThis is an updated test note.";
        storage.update_note_file(&file_name, new_content).unwrap();
        
        let updated_content = storage.read_note_file(&file_name).unwrap();
        assert_eq!(updated_content, new_content);
        
        // Scan files
        let files = storage.scan_existing_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, file_name);
    }
    
    #[test]
    fn test_filename_sanitization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let sanitized = storage.sanitize_filename("Test: <Invalid> Characters?");
        assert_eq!(sanitized, "Test_ _Invalid_ Characters_");
        
        let empty_sanitized = storage.sanitize_filename("");
        assert_eq!(empty_sanitized, "Untitled");
    }
    
    #[test]
    fn test_unique_file_path_generation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create first file
        let file1 = storage.create_note_file("Test", "Content 1").unwrap();
        assert_eq!(file1, "Test.md");
        
        // Create file with same name, should have number suffix
        let file2 = storage.create_note_file("Test", "Content 2").unwrap();
        assert_eq!(file2, "Test(1).md");
    }
    
    #[test]
    fn test_content_search() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Create test files
        storage.create_note_file("First Note", "This is about programming").unwrap();
        storage.create_note_file("Second Note", "This discusses programming languages").unwrap();
        storage.create_note_file("Third Note", "This is about cooking").unwrap();
        
        // Search
        let results = storage.search_files_content("programming").unwrap();
        assert_eq!(results.len(), 2);
        
        // Verify relevance sorting
        assert!(results[0].relevance_score >= results[1].relevance_score);
    }
}