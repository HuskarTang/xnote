pub mod config;
pub mod logger;
pub mod models;
pub mod notes;
pub mod storage;
pub mod tags;

// Re-export the main types for easier access
pub use config::{AppConfig, ConfigManager};
pub use logger::*;
pub use models::*;
pub use notes::NotesManager;
pub use storage::FileStorageManager;
pub use tags::TagsManager;