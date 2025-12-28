use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::models::NoteMetadata;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct FileStorageManager {
    pub notes_directory: PathBuf,
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

    pub fn parse_note(&self, file_name: &str) -> Result<(NoteMetadata, String)> {
        let file_path = self.notes_directory.join(file_name);
        let content = fs::read_to_string(&file_path)?;
        
        if content.starts_with("---") {
            if let Some(end_idx) = content[3..].find("---") {
                let yaml_start = 3;
                let yaml_end = end_idx + 3;
                let yaml_str = content[yaml_start..yaml_end].trim();
                
                let mut body_start = yaml_end + 3;
                if content.len() > body_start {
                    if content[body_start..].starts_with("\r\n") {
                        body_start += 2;
                    } else if content[body_start..].starts_with('\n') {
                        body_start += 1;
                    }
                }
                
                let body = if body_start < content.len() {
                    &content[body_start..]
                } else {
                    ""
                };
                
                match serde_yaml::from_str::<NoteMetadata>(yaml_str) {
                    Ok(mut metadata) => {
                        if metadata.id.is_empty() {
                            metadata.id = uuid::Uuid::new_v4().to_string();
                        }
                        return Ok((metadata, body.to_string()));
                    },
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to parse YAML header in {}: {}", file_name, e));
                    }
                }
            }
        }
        
        // Fallback for files without front matter: generate header and save back
        let file_sys_metadata = fs::metadata(&file_path)?;
        let created_time: DateTime<Utc> = file_sys_metadata.created().unwrap_or(std::time::SystemTime::now()).into();
        let modified_time: DateTime<Utc> = file_sys_metadata.modified().unwrap_or(std::time::SystemTime::now()).into();

        let mut title = self.extract_title_from_file_name(file_name);
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("# ") {
                let h1_title = first_line[2..].trim();
                if !h1_title.is_empty() {
                    title = h1_title.to_string();
                }
            }
        }

        let new_metadata = NoteMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            tags: vec![],
            attachments: vec![],
            created: created_time.to_rfc3339(),
            modified: modified_time.to_rfc3339(),
            favorite: false,
            deleted: false,
        };

        self.save_note(file_name, &new_metadata, &content)?;

        Ok((new_metadata, content))
    }

    pub fn save_note(&self, file_name: &str, metadata: &NoteMetadata, content: &str) -> Result<()> {
        let file_path = self.notes_directory.join(file_name);
        let yaml = serde_yaml::to_string(metadata)?;
        // Ensure there are newlines around YAML markers
        let full_content = format!("---\n{}---\n\n{}", yaml, content);
        
        // Use atomic write: write to temp file then rename
        let temp_path = file_path.with_extension("tmp");
        fs::write(&temp_path, full_content)?;
        fs::rename(&temp_path, &file_path)?;
        
        Ok(())
    }
    
    pub fn rename_note_file(&self, old_file_name: &str, new_title: &str) -> Result<String> {
        let old_path = self.notes_directory.join(old_file_name);
        
        if !old_path.exists() {
            return Err(anyhow::anyhow!("Note file does not exist: {}", old_file_name));
        }
        
        let base_name = self.sanitize_filename(new_title);
        let mut attempts = 0;
        let max_attempts = 1000;
        
        loop {
            let new_file_name = if attempts == 0 {
                format!("{}.md", base_name)
            } else {
                format!("{}({}).md", base_name, attempts)
            };
            
            let new_file_path = self.notes_directory.join(&new_file_name);
            
            // Skip if trying to rename to the same name
            if old_path == new_file_path {
                return Ok(old_file_name.to_string());
            }
            
            // Try to rename to new path (atomic operation, fails if target exists)
            match fs::rename(&old_path, &new_file_path) {
                Ok(_) => return Ok(new_file_name),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Target exists, try next number
                    attempts += 1;
                    if attempts > max_attempts {
                        return Err(anyhow::anyhow!("Failed to generate unique file name after {} attempts", max_attempts));
                    }
                    continue;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to rename file: {}", e));
                }
            }
        }
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
    
    pub fn generate_unique_file_path(&self, title: &str) -> Result<PathBuf> {
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
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub modified: Option<std::time::SystemTime>,
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
    
    fn create_test_note_metadata(title: &str) -> NoteMetadata {
        NoteMetadata {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            tags: vec![],
            attachments: vec![],
            created: Utc::now().to_rfc3339(),
            modified: Utc::now().to_rfc3339(),
            favorite: false,
            deleted: false,
        }
    }

    #[test]
    fn test_note_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let title = "Test Note";
        let content = "# Test Note\n\nThis is a test note.";
        let metadata = create_test_note_metadata(title);
        
        // Save note (create)
        storage.save_note("Test Note.md", &metadata, content).unwrap();
        
        // Read note (parse)
        let (read_metadata, read_content) = storage.parse_note("Test Note.md").unwrap();
        assert_eq!(read_content, content);
        assert_eq!(read_metadata.title, title);
        
        // Scan files
        let files = storage.scan_existing_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "Test Note.md");
    }

    #[test]
    fn test_note_metadata_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorageManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let title = "Metadata Note";
        let content = "# Header\n\nContent.";
        let mut metadata = create_test_note_metadata(title);
        metadata.tags = vec!["tag1".to_string()];
        metadata.favorite = true;
        
        storage.save_note("meta.md", &metadata, content).unwrap();
        
        let (parsed_metadata, parsed_content) = storage.parse_note("meta.md").unwrap();
        
        assert_eq!(parsed_metadata.title, title);
        assert_eq!(parsed_metadata.favorite, true);
        assert_eq!(parsed_metadata.tags[0], "tag1");
        assert_eq!(parsed_content, content);
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
        
        let metadata = create_test_note_metadata("Test");

        // Create first file
        storage.save_note("Test.md", &metadata, "Content 1").unwrap();
        
        // Generate path for same title
        let path = storage.generate_unique_file_path("Test").unwrap();
        assert_eq!(path.file_name().unwrap().to_str().unwrap(), "Test(1).md");
    }
}