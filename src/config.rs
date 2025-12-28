use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub data_directory: Option<PathBuf>,
    pub window_width: u32,
    pub window_height: u32,
    pub sidebar_width: u32,
    pub note_list_width: u32,
    pub auto_save_interval: u32,
    pub theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_directory: None,
            window_width: 1200,
            window_height: 800,
            sidebar_width: 200,
            note_list_width: 300,
            auto_save_interval: 5000, // 5 seconds
            theme: "light".to_string(),
        }
    }
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: AppConfig = serde_json::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            let config = AppConfig::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        Ok(config_dir.join("mdnote").join("config.json"))
    }
    
    pub fn is_first_run(&self) -> bool {
        self.data_directory.is_none()
    }
    
    pub fn set_data_directory(&mut self, path: PathBuf) -> Result<()> {
        // Create data directory if it doesn't exist
        fs::create_dir_all(&path)
            .context("Failed to create data directory")?;
        
        // Create attachments subdirectory
        let attachments_dir = path.join("attachments");
        fs::create_dir_all(&attachments_dir)
            .context("Failed to create attachments directory")?;
        
        self.data_directory = Some(path);
        self.save()?;
        Ok(())
    }
    
    pub fn get_data_directory(&self) -> Option<&PathBuf> {
        self.data_directory.as_ref()
    }
    
    pub fn get_attachments_directory(&self) -> Option<PathBuf> {
        self.data_directory.as_ref().map(|d| d.join("attachments"))
    }
    
    pub fn get_database_path(&self) -> Option<PathBuf> {
        self.data_directory.as_ref().map(|d| d.join("mdnote.db"))
    }
}