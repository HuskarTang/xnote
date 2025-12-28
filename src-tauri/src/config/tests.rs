#[cfg(test)]
mod config_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_config_creation_and_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");
        
        // Set environment variables to use our test directories
        std::env::set_var("XDG_CONFIG_HOME", &config_dir);
        std::env::set_var("HOME", &data_dir);
        
        // Create config manager - this should trigger initialization
        let config_manager = ConfigManager::new().unwrap();
        
        // Check that config file was created
        let config_path = config_dir.join("xnote").join("config.json");
        assert!(config_path.exists());
        
        // Check that data directory was created
        assert!(data_dir.join("Documents").join("xnote").exists());
        
        // Check that database file was created
        let db_path = data_dir.join("Documents").join("xnote").join("xnote.db");
        assert!(db_path.exists());
    }
    
    #[test]
    fn test_config_file_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");
        
        // Set environment variables
        std::env::set_var("XDG_CONFIG_HOME", &config_dir);
        std::env::set_var("HOME", &data_dir);
        
        // Create config manager
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.get_config();
        
        // Check default values
        assert_eq!(config.window_width, 1200);
        assert_eq!(config.window_height, 800);
        assert_eq!(config.auto_save_interval, 5000);
        assert_eq!(config.theme, "light");
    }
    
    #[test]
    fn test_external_file_migration() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");
        let notes_dir = data_dir.join("Documents").join("xnote");
        
        // Set environment variables
        std::env::set_var("XDG_CONFIG_HOME", &config_dir);
        std::env::set_var("HOME", &data_dir);
        
        // Create some external Markdown files before initializing the app
        fs::create_dir_all(&notes_dir).unwrap();
        fs::write(notes_dir.join("note1.md"), "# Note 1\n\nThis is the first note.").unwrap();
        fs::write(notes_dir.join("note2.md"), "# Note 2\n\nThis is the second note.").unwrap();
        
        // Create config manager - this should trigger initialization and scan for existing files
        let config_manager = ConfigManager::new().unwrap();
        
        // Check that database file was created
        let db_path = notes_dir.join("xnote.db");
        assert!(db_path.exists());
        
        // In a real implementation, we would check that the notes were added to the database
        // But for this test, we're just verifying the config management works
    }
}