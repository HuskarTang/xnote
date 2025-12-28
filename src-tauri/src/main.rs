#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod database;
mod models;
mod notes;
mod storage;
mod tags;

use commands::{AppState};
use config::ConfigManager;
use database::DatabaseManager;
use notes::NotesManager;
use storage::FileStorageManager;
use tags::TagsManager;
use std::sync::Arc;
use tauri::Manager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration
    let config_manager = ConfigManager::new()
        .map_err(|e| format!("Failed to initialize config: {}", e))?;
    
    // Initialize database
    let db_path = config_manager.get_database_path();
    let database_manager = DatabaseManager::new(&db_path).await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    
    // Initialize file storage
    let notes_directory = config_manager.get_notes_directory();
    let storage_manager = FileStorageManager::new(notes_directory)
        .map_err(|e| format!("Failed to initialize storage: {}", e))?;
    
    // Initialize managers
    let notes_manager = NotesManager::new(database_manager.clone(), storage_manager);
    let tags_manager = TagsManager::new(database_manager);
    
    // Create shared application state
    let app_state = Arc::new(AppState {
        notes_manager: Arc::new(notes_manager),
        tags_manager: Arc::new(tags_manager),
        config_manager: Arc::new(config_manager),
    });

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_all_notes,
            commands::get_favorites,
            commands::get_untagged,
            commands::get_trash,
            commands::get_notes_by_tag,
            commands::get_all_tags,
            commands::get_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::search_notes,
            commands::restore_note,
            commands::permanently_delete_note,
            commands::show_directory_dialog,
            commands::update_data_directory
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}