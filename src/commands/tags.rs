use tauri::State;
use anyhow::Result;

use crate::{
    AppState,
    models::Tag,
    database::Database,
};

#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let db = state.db.lock().await;
    db.get_all_tags()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag_to_note(
    note_id: String,
    tag_name: String,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let db = state.db.lock().await;
    
    // Create or get existing tag
    let tag = create_or_get_tag(&tag_name, &db)
        .await
        .map_err(|e| e.to_string())?;
    
    // Add tag to note
    db.add_tag_to_note(&note_id, &tag.id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(tag)
}

#[tauri::command]
pub async fn remove_tag_from_note(
    note_id: String,
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.remove_tag_from_note(&note_id, &tag_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_notes_by_tag(
    tag_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::NoteWithTags>, String> {
    let db = state.db.lock().await;
    db.get_notes_by_tag(&tag_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_untagged_notes(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::NoteWithTags>, String> {
    let db = state.db.lock().await;
    
    // Get all notes and filter those without tags
    let all_notes = db.get_all_notes(false)
        .await
        .map_err(|e| e.to_string())?;
    
    let untagged_notes = all_notes
        .into_iter()
        .filter(|note_with_tags| note_with_tags.tags.is_empty())
        .collect();
    
    Ok(untagged_notes)
}

#[tauri::command]
pub async fn get_favorite_notes(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::NoteWithTags>, String> {
    let db = state.db.lock().await;
    
    // Get all notes and filter favorites
    let all_notes = db.get_all_notes(false)
        .await
        .map_err(|e| e.to_string())?;
    
    let favorite_notes = all_notes
        .into_iter()
        .filter(|note_with_tags| note_with_tags.note.is_favorite)
        .collect();
    
    Ok(favorite_notes)
}

#[tauri::command]
pub async fn get_trashed_notes(
    state: State<'_, AppState>,
) -> Result<Vec<crate::models::NoteWithTags>, String> {
    let db = state.db.lock().await;
    
    // Get all notes including trashed and filter only trashed ones
    let all_notes = db.get_all_notes(true)
        .await
        .map_err(|e| e.to_string())?;
    
    let trashed_notes = all_notes
        .into_iter()
        .filter(|note_with_tags| note_with_tags.note.is_trashed)
        .collect();
    
    Ok(trashed_notes)
}

#[tauri::command]
pub async fn create_tag(
    tag_name: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let db = state.db.lock().await;
    
    // Check if tag already exists
    if let Ok(Some(existing_tag)) = db.get_tag_by_name(&tag_name).await {
        return Ok(existing_tag);
    }
    
    // Create new tag
    let mut tag = Tag::new(tag_name);
    tag.color = color;
    
    db.create_tag(&tag)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(tag)
}

#[tauri::command]
pub async fn update_tag_color(
    tag_id: String,
    color: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Note: This would require adding an update_tag method to the database
    // For now, we'll return an error indicating this feature is not yet implemented
    Err("Tag color update not yet implemented".to_string())
}

#[tauri::command]
pub async fn delete_tag(
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Note: This would require adding a delete_tag method to the database
    // For now, we'll return an error indicating this feature is not yet implemented
    Err("Tag deletion not yet implemented".to_string())
}

// Helper function to create or get existing tag
pub async fn create_or_get_tag(tag_name: &str, db: &Database) -> Result<Tag> {
    // Check if tag already exists
    if let Some(existing_tag) = db.get_tag_by_name(tag_name).await? {
        return Ok(existing_tag);
    }
    
    // Create new tag
    let tag = Tag::new(tag_name.to_string());
    db.create_tag(&tag).await?;
    
    Ok(tag)
}