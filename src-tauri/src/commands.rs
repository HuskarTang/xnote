use crate::{NotesManager, TagsManager, ConfigManager};
use crate::models::*;
use std::sync::Arc;
use tauri::State;
use anyhow::Result;
use tauri::api::dialog::FileDialogBuilder;

// Define a shared application state
pub struct AppState {
    pub notes_manager: Arc<NotesManager>,
    pub tags_manager: Arc<TagsManager>,
    pub config_manager: Arc<ConfigManager>,
}

#[tauri::command]
pub async fn get_all_notes(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    state.notes_manager.get_all_notes().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    state.notes_manager.get_favorite_notes().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_untagged(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    state.notes_manager.get_untagged_notes().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_trash(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    state.notes_manager.get_trash().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notes_by_tag(state: State<'_, Arc<AppState>>, tag_name: String) -> Result<Vec<Note>, String> {
    state.notes_manager.get_notes_by_tag(&tag_name).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_tags(state: State<'_, Arc<AppState>>) -> Result<Vec<Tag>, String> {
    state.tags_manager.get_all_tags().await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note(state: State<'_, Arc<AppState>>, id: String) -> Result<Option<Note>, String> {
    state.notes_manager.get_note(&id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_note(state: State<'_, Arc<AppState>>, request: CreateNoteRequest) -> Result<Note, String> {
    state.notes_manager.create_note(request).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_note(state: State<'_, Arc<AppState>>, request: UpdateNoteRequest) -> Result<Option<Note>, String> {
    state.notes_manager.update_note(request).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    state.notes_manager.delete_note(&id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes(_state: State<'_, Arc<AppState>>, _request: SearchRequest) -> Result<Vec<Note>, String> {
    // TODO: Implement search functionality in NotesManager
    Err("Search not implemented yet".to_string())
}

#[tauri::command]
pub async fn restore_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    state.notes_manager.restore_note(&id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn permanently_delete_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    state.notes_manager.permanently_delete_note(&id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_directory_dialog() -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    
    FileDialogBuilder::new()
        .set_title("Select Data Directory")
        .pick_folder(move |path_buf| {
            let _ = tx.send(path_buf.map(|p| p.to_string_lossy().to_string()));
        });
    
    rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_data_directory(state: State<'_, Arc<AppState>>, path: String) -> Result<bool, String> {
    use std::path::PathBuf;
    
    let mut config = state.config_manager.get_config().clone();
    config.data_directory = PathBuf::from(path);
    
    // Update the config manager
    // Note: This is a simplified approach. In a real implementation, 
    // you would need to properly update the config and reinitialize managers.
    Ok(true)
}