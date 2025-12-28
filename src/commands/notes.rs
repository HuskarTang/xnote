use tauri::State;
use std::path::PathBuf;
use chrono::Utc;
use anyhow::Result;

use crate::{
    AppState,
    models::{Note, NoteWithTags, NoteContent, CreateNoteRequest, SaveNoteRequest, SearchRequest},
    utils::{extract_title_from_markdown, read_file_content, write_file_content, generate_unique_filename},
};

#[tauri::command]
pub async fn get_all_notes(
    include_trashed: Option<bool>,
    state: State<'_, AppState>,
) -> Result<Vec<NoteWithTags>, String> {
    let db = state.db.lock().await;
    db.get_all_notes(include_trashed.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note_content(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<NoteContent, String> {
    let config = state.config.lock().await;
    let data_dir = config.get_data_directory()
        .ok_or("Data directory not configured")?;
    
    let db = state.db.lock().await;
    let note = db.get_note_by_id(&note_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    let file_path = data_dir.join(&note.file_path);
    let content = read_file_content(&file_path)
        .map_err(|e| e.to_string())?;
    
    let tags = db.get_tags_for_note(&note_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(NoteContent {
        id: note.id,
        title: note.title,
        content,
        tags,
        created_at: note.created_at,
        modified_at: note.modified_at,
        is_favorite: note.is_favorite,
        is_trashed: note.is_trashed,
    })
}

#[tauri::command]
pub async fn create_note(
    request: CreateNoteRequest,
    state: State<'_, AppState>,
) -> Result<NoteContent, String> {
    let config = state.config.lock().await;
    let data_dir = config.get_data_directory()
        .ok_or("Data directory not configured")?;
    
    let content = request.content.unwrap_or_else(|| "# New Note\n\nStart writing...".to_string());
    let title = request.title.unwrap_or_else(|| extract_title_from_markdown(&content));
    
    // Generate unique filename
    let file_path = generate_unique_filename(data_dir, &title, "md");
    let relative_path = file_path.strip_prefix(data_dir)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();
    
    // Create note record
    let note = Note::new(title, relative_path);
    
    // Save to database
    let db = state.db.lock().await;
    db.create_note(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    // Save file content
    write_file_content(&file_path, &content)
        .map_err(|e| e.to_string())?;
    
    // Add tags if provided
    let mut tags = Vec::new();
    if let Some(tag_names) = request.tags {
        for tag_name in tag_names {
            if let Ok(tag) = crate::commands::tags::create_or_get_tag(&tag_name, &db).await {
                db.add_tag_to_note(&note.id, &tag.id)
                    .await
                    .map_err(|e| e.to_string())?;
                tags.push(tag);
            }
        }
    }
    
    Ok(NoteContent {
        id: note.id,
        title: note.title,
        content,
        tags,
        created_at: note.created_at,
        modified_at: note.modified_at,
        is_favorite: note.is_favorite,
        is_trashed: note.is_trashed,
    })
}

#[tauri::command]
pub async fn save_note(
    request: SaveNoteRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().await;
    let data_dir = config.get_data_directory()
        .ok_or("Data directory not configured")?;
    
    let db = state.db.lock().await;
    let mut note = db.get_note_by_id(&request.id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    // Update note metadata
    note.title = request.title;
    note.modified_at = Utc::now();
    
    // Save to database
    db.update_note(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    // Save file content
    let file_path = data_dir.join(&note.file_path);
    write_file_content(&file_path, &request.content)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_note(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().await;
    let data_dir = config.get_data_directory()
        .ok_or("Data directory not configured")?;
    
    let db = state.db.lock().await;
    let note = db.get_note_by_id(&note_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    // Delete file
    let file_path = data_dir.join(&note.file_path);
    if file_path.exists() {
        std::fs::remove_file(&file_path)
            .map_err(|e| e.to_string())?;
    }
    
    // Delete from database
    db.delete_note(&note_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn move_to_trash(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let mut note = db.get_note_by_id(&note_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    note.is_trashed = true;
    note.modified_at = Utc::now();
    
    db.update_note(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn restore_from_trash(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let mut note = db.get_note_by_id(&note_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    note.is_trashed = false;
    note.modified_at = Utc::now();
    
    db.update_note(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn toggle_favorite(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.db.lock().await;
    let mut note = db.get_note_by_id(&note_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Note not found")?;
    
    note.is_favorite = !note.is_favorite;
    note.modified_at = Utc::now();
    
    db.update_note(&note)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(note.is_favorite)
}

#[tauri::command]
pub async fn search_notes(
    request: SearchRequest,
    state: State<'_, AppState>,
) -> Result<Vec<NoteWithTags>, String> {
    let db = state.db.lock().await;
    
    if let Some(tag_filter) = &request.tag_filter {
        db.get_notes_by_tag(tag_filter)
            .await
            .map_err(|e| e.to_string())
    } else {
        db.search_notes(&request.query)
            .await
            .map_err(|e| e.to_string())
    }
}