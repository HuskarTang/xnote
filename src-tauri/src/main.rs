#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod database;
mod logger;
mod models;
mod notes;
mod storage;
mod sync;
mod tags;

use commands::{AppState};
use config::ConfigManager;
use database::DatabaseManager;
use logger::init_logger;
use notes::NotesManager;
use storage::FileStorageManager;
use tags::TagsManager;
use std::sync::{Arc, Mutex};
use tauri::{Menu, MenuItem, Submenu, CustomMenuItem, WindowMenuEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger with console output
    init_logger();
    
    log::info!("Starting XNote application");
    println!("🚀 XNote application starting...");
    
    // Initialize configuration
    let config_manager = ConfigManager::new()
        .map_err(|e| format!("Failed to initialize config: {}", e))?;
    
    // Check if setup is required
    let setup_required = config_manager.requires_setup();
    
    // If setup is required, we'll handle it in the frontend
    if setup_required {
        log_info!("Setup required - will be handled in frontend");
    } else {
        // Change working directory to notes directory for relative path support
        let notes_directory = config_manager.get_notes_directory();
        std::env::set_current_dir(&notes_directory)
            .map_err(|e| format!("Failed to change working directory to {}: {}", notes_directory.display(), e))?;
        log_info!("Changed working directory to: {}", notes_directory.display());
    }
    
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
        config_manager: Arc::new(Mutex::new(config_manager)),
    });

    // Create menu
    let menu = create_menu();

    tauri::Builder::default()
        .menu(menu)
        .on_menu_event(handle_menu_event)
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
            commands::update_data_directory,
            commands::reinitialize_data_directory,
            commands::sync_external_files,
            commands::is_setup_required,
            commands::mark_setup_complete,
            // Tag management commands
            commands::create_tag,
            commands::delete_tag,
            commands::rename_tag,
            commands::add_tag_to_note,
            commands::remove_tag_from_note,
            commands::get_note_tags,
            commands::search_tags,
            commands::cleanup_unused_tags,
            commands::save_image_to_attachments,
            commands::save_image_for_text,
            commands::get_data_directory,
            commands::add_attachment,
            commands::list_attachments,
            commands::delete_attachment,
            commands::open_attachment,
            commands::cleanup_unreferenced_attachments,
            commands::export_note,
            commands::show_export_dialog,
            // Git sync commands
            commands::get_app_config,
            commands::get_git_sync_config,
            commands::update_git_sync_config,
            commands::update_theme,
            commands::get_sync_status,
            commands::get_local_changes,
            commands::perform_sync,
            commands::commit_changes,
            commands::stash_changes,
            commands::stash_pop,
            commands::pull_from_remote,
            commands::pull_rebase,
            commands::push_to_remote,
            commands::smart_push_to_remote,
            commands::get_remote_commits,
            commands::get_commit_history
        ])
        .setup(|_app| {
            // 注释掉自动打开开发者工具的代码
            // #[cfg(debug_assertions)]
            // {
            //     if let Some(window) = app.get_window("main") {
            //         window.open_devtools();
            //     }
            // }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}

fn create_menu() -> Menu {
    let preferences = CustomMenuItem::new("preferences".to_string(), "Preferences...").accelerator("Cmd+,");
    let quit = CustomMenuItem::new("quit".to_string(), "Quit XNote").accelerator("Cmd+Q");
    let about = CustomMenuItem::new("about".to_string(), "About XNote");
    
    let app_menu = Submenu::new(
        "XNote",
        Menu::new()
            .add_item(about)
            .add_native_item(MenuItem::Separator)
            .add_item(preferences)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Services)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Hide)
            .add_native_item(MenuItem::HideOthers)
            .add_native_item(MenuItem::ShowAll)
            .add_native_item(MenuItem::Separator)
            .add_item(quit),
    );

    let file_menu = Submenu::new(
        "File",
        Menu::new()
            .add_native_item(MenuItem::CloseWindow),
    );

    let edit_menu = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );

    let view_menu = Submenu::new(
        "View",
        Menu::new()
            .add_native_item(MenuItem::EnterFullScreen),
    );

    let window_menu = Submenu::new(
        "Window",
        Menu::new()
            .add_native_item(MenuItem::Minimize)
            .add_native_item(MenuItem::Zoom),
    );

    Menu::new()
        .add_submenu(app_menu)
        .add_submenu(file_menu)
        .add_submenu(edit_menu)
        .add_submenu(view_menu)
        .add_submenu(window_menu)
}

fn handle_menu_event(event: WindowMenuEvent) {
    match event.menu_item_id() {
        "preferences" => {
            // 发送事件到前端打开设置
            event.window().emit("open-preferences", {}).unwrap();
        }
        "about" => {
            // 发送事件到前端显示关于信息
            event.window().emit("show-about", {}).unwrap();
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}