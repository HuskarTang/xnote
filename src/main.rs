// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod database;
mod models;
mod commands;
mod utils;

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

use config::AppConfig;
use database::Database;

pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub db: Arc<Mutex<Database>>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle();
            
            // Initialize configuration
            let config = Arc::new(Mutex::new(AppConfig::new()?));
            
            // Initialize database
            let db = Arc::new(Mutex::new(Database::new()?));
            
            // Store state
            app_handle.manage(AppState { config, db });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_config,
            commands::config::set_data_directory,
            commands::config::is_first_run,
            commands::notes::get_all_notes,
            commands::notes::get_note_content,
            commands::notes::save_note,
            commands::notes::create_note,
            commands::notes::delete_note,
            commands::notes::move_to_trash,
            commands::notes::restore_from_trash,
            commands::notes::search_notes,
            commands::tags::get_all_tags,
            commands::tags::add_tag_to_note,
            commands::tags::remove_tag_from_note,
            commands::tags::get_notes_by_tag,
            commands::attachments::save_attachment,
            commands::attachments::get_attachment_path,
            commands::export::export_note_as_html,
            commands::export::export_note_as_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}