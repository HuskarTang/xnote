use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::{Result, Context};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppConfig {
    #[serde(alias = "dataDirectory")]
    pub data_directory: PathBuf,
    #[serde(alias = "windowWidth")]
    pub window_width: u32,
    #[serde(alias = "windowHeight")]
    pub window_height: u32,
    #[serde(alias = "sidebarWidth")]
    pub sidebar_width: u32,
    #[serde(alias = "noteListWidth")]
    pub note_list_width: u32,
    #[serde(alias = "autoSaveInterval")]
    pub auto_save_interval: u32,
    #[serde(alias = "logConfig")]
    pub log_config: Option<LogConfig>,
    pub theme: String,
    #[serde(alias = "gitSync")]
    pub git_sync: Option<GitSyncConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LogConfig {
    #[serde(alias = "enabled")]
    pub enabled: bool,
    #[serde(alias = "level")]
    pub level: String,
    #[serde(alias = "maxDays")]
    pub max_days: u32,
    #[serde(alias = "consoleOutput")]
    pub console_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GitSyncConfig {
    pub enabled: bool,
    #[serde(alias = "repositoryUrl")]
    pub repository_url: String,
    pub branch: String,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(alias = "sshKeyPath")]
    pub ssh_key_path: Option<String>,
    #[serde(alias = "authType")]
    pub auth_type: String, // "none", "basic", "ssh"
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_directory: get_default_data_dir(),
            window_width: 1200,
            window_height: 800,
            sidebar_width: 240,
            note_list_width: 320,
            auto_save_interval: 5000,
            log_config: Some(LogConfig {
                enabled: true,
                level: "info".to_string(),
                max_days: 7,
                console_output: true,
            }),
            theme: "light".to_string(),
            git_sync: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = get_config_dir()
            .context("Failed to get config directory")?;
        
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        
        let config_path = config_dir.join("config.json");
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            // Create a flag file to indicate that setup is required
            let setup_flag_path = config_dir.join("setup_required");
            fs::write(&setup_flag_path, "")?;
            
            // Return default config - UI will handle directory selection
            AppConfig::default()
        };
        
        // Ensure data directory exists
        fs::create_dir_all(&config.data_directory)
            .context("Failed to create data directory")?;
        
        // No database initialization or scanning needed
        
        Ok(Self {
            config_path,
            config,
        })
    }
    
    pub fn requires_setup(&self) -> bool {
        // Check if setup flag file exists
        let fallback = PathBuf::from(".");
        let config_dir = match self.config_path.parent() {
            Some(parent) => parent,
            None => fallback.as_path(),
        };
        let setup_flag_path = config_dir.join("setup_required");
        setup_flag_path.exists()
    }
    
    pub fn mark_setup_complete(&self) -> Result<()> {
        // Remove setup flag file
        let fallback = PathBuf::from(".");
        let config_dir = match self.config_path.parent() {
            Some(parent) => parent,
            None => fallback.as_path(),
        };
        let setup_flag_path = config_dir.join("setup_required");
        if setup_flag_path.exists() {
            fs::remove_file(&setup_flag_path)?;
        }
        Ok(())
    }
    
    // Removed unused private methods: show_directory_selection_dialog, initialize_database, scan_and_import_existing_files
    
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
    
    pub fn update_config(&mut self, new_config: AppConfig) -> Result<()> {
        self.config = new_config;
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }
    
    pub fn update_data_directory(&mut self, new_path: PathBuf) -> Result<()> {
        self.config.data_directory = new_path;
        Self::save_config(&self.config_path, &self.config)?;
        Ok(())
    }
    
    pub fn get_notes_directory(&self) -> PathBuf {
        self.config.data_directory.clone()
    }
    
    // Removed get_database_path
    
    fn load_config(path: &PathBuf) -> Result<AppConfig> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        serde_json::from_str(&content)
            .context("Failed to parse config file")
    }
    
    fn save_config(path: &PathBuf, config: &AppConfig) -> Result<()> {
        let content = serde_json::to_string_pretty(config)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .context("Failed to write config file")
    }
}

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = if cfg!(target_os = "macos") {
        dirs::home_dir()
            .context("Failed to get home directory")?
            .join("Library")
            .join("Application Support")
            .join("xnote")
    } else if cfg!(target_os = "windows") {
        dirs::config_dir()
            .context("Failed to get config directory")?
            .join("xnote")
    } else {
        dirs::config_dir()
            .context("Failed to get config directory")?
            .join("xnote")
    };
    
    Ok(config_dir)
}

fn get_default_data_dir() -> PathBuf {
    let data_dir = if cfg!(target_os = "macos") {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Documents")
            .join("XNote")
    } else if cfg!(target_os = "windows") {
        dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("XNote")
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Documents")
            .join("xnote")
    };
    
    data_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.window_width, 1200);
        assert_eq!(config.window_height, 800);
        assert_eq!(config.theme, "light");
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.window_width, deserialized.window_width);
        assert_eq!(config.theme, deserialized.theme);
    }
    
    #[test]
    fn test_config_manager_paths() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = AppConfig::default();
        config.data_directory = temp_dir.path().to_path_buf();
        
        // 模拟配置管理器的路径生成逻辑
        assert_eq!(
            config.data_directory.join("attachments"),
            temp_dir.path().join("attachments")
        );
        // Removed xnote.db check
    }
    
    #[test]
    fn test_camel_case_deserialization() {
        let json = r#"{
            "dataDirectory": "/test/path",
            "windowWidth": 1000,
            "windowHeight": 800,
            "sidebarWidth": 200,
            "noteListWidth": 300,
            "autoSaveInterval": 3000,
            "theme": "dark",
            "logConfig": {
                "enabled": true,
                "level": "debug",
                "maxDays": 14,
                "consoleOutput": false
            }
        }"#;
        
        let config: AppConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.data_directory, PathBuf::from("/test/path"));
        assert_eq!(config.window_width, 1000);
        assert_eq!(config.window_height, 800);
        assert_eq!(config.sidebar_width, 200);
        assert_eq!(config.note_list_width, 300);
        assert_eq!(config.auto_save_interval, 3000);
        assert_eq!(config.theme, "dark");
        
        if let Some(log_config) = config.log_config {
            assert_eq!(log_config.enabled, true);
            assert_eq!(log_config.level, "debug");
            assert_eq!(log_config.max_days, 14);
            assert_eq!(log_config.console_output, false);
        }
    }
}