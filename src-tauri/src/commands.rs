use crate::{NotesManager, TagsManager, ConfigManager};
use uuid;
use crate::models::*;
use crate::{log_debug, log_info, log_warn, log_error};
use std::sync::{Arc, Mutex};
use tauri::{State, Manager};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

// Define a shared application state
pub struct AppState {
    pub notes_manager: Arc<NotesManager>,
    pub tags_manager: Arc<TagsManager>,
    pub config_manager: Arc<Mutex<ConfigManager>>,
}

#[tauri::command]
pub async fn is_setup_required(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Checking if setup is required");
    let requires_setup = state.config_manager.lock().unwrap().requires_setup();
    log_debug!("Setup required: {}", requires_setup);
    Ok(requires_setup)
}

#[tauri::command]
pub async fn get_all_notes(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    log_info!("Getting all notes");
    let result = state.notes_manager.get_all_notes().await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} notes", notes.len());
    }
    
    result
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    log_info!("Getting favorite notes");
    let result = state.notes_manager.get_favorite_notes().await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} favorite notes", notes.len());
    }
    
    result
}

#[tauri::command]
pub async fn get_untagged(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    log_info!("Getting untagged notes");
    let result = state.notes_manager.get_untagged_notes().await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} untagged notes", notes.len());
    }
    
    result
}

#[tauri::command]
pub async fn get_trash(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    log_info!("Getting trash notes");
    let result = state.notes_manager.get_trash().await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} trash notes", notes.len());
    }
    
    result
}

#[tauri::command]
pub async fn get_notes_by_tag(state: State<'_, Arc<AppState>>, tag_name: String) -> Result<Vec<Note>, String> {
    log_info!("Getting notes by tag: {}", tag_name);
    let result = state.notes_manager.get_notes_by_tag(&tag_name).await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} notes with tag '{}'", notes.len(), tag_name);
    }
    
    result
}

#[tauri::command]
pub async fn get_all_tags(state: State<'_, Arc<AppState>>) -> Result<Vec<Tag>, String> {
    log_info!("Getting all tags");
    let result = state.tags_manager.get_all_tags().await
        .map_err(|e| e.to_string());
    
    if let Ok(tags) = &result {
        log_debug!("Found {} tags", tags.len());
    }
    
    result
}

#[tauri::command]
pub async fn get_note(state: State<'_, Arc<AppState>>, id: String) -> Result<Option<Note>, String> {
    log_info!("Getting note with id: {}", id);
    let result = state.notes_manager.get_note(&id).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(Some(note)) => log_debug!("Found note with title: {}", note.title),
        Ok(None) => log_debug!("Note with id '{}' not found", id),
        Err(e) => log_error!("Error getting note {}: {}", id, e),
    }
    
    result
}

#[tauri::command]
pub async fn create_note(state: State<'_, Arc<AppState>>, request: CreateNoteRequest) -> Result<Note, String> {
    log_info!("Creating new note with title: {}", request.title);
    let result = state.notes_manager.create_note(request).await
        .map_err(|e| e.to_string());
    
    if let Ok(note) = &result {
        log_debug!("Successfully created note with id: {}", note.id);
    }
    
    result
}

#[tauri::command]
pub async fn update_note(state: State<'_, Arc<AppState>>, request: UpdateNoteRequest) -> Result<Option<Note>, String> {
    log_info!("Updating note with id: {}", request.id);
    let result = state.notes_manager.update_note(request).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(Some(note)) => log_debug!("Successfully updated note with title: {}", note.title),
        Ok(None) => log_debug!("Note with id not found for update"),
        Err(e) => log_error!("Error updating note: {}", e),
    }
    
    result
}

#[tauri::command]
pub async fn delete_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    log_info!("Deleting note with id: {}", id);
    let result = state.notes_manager.delete_note(&id).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(true) => {
            log_debug!("Successfully deleted note {}", id);
            // Cleanup unused tags after note deletion
            if let Err(e) = state.tags_manager.cleanup_unused_tags().await {
                log_error!("Failed to cleanup unused tags after deleting note {}: {}", id, e);
            }
        },
        Ok(false) => log_debug!("Note {} not found for deletion", id),
        Err(e) => log_error!("Error deleting note {}: {}", id, e),
    }
    
    result
}

#[tauri::command]
pub async fn search_notes(state: State<'_, Arc<AppState>>, request: SearchRequest) -> Result<Vec<Note>, String> {
    log_info!("Searching notes with query: '{}', tag_filter: {:?}", request.query, request.tag_filter);
    
    let result = state.notes_manager.search_notes(&request.query, request.tag_filter.as_deref()).await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Found {} notes matching search criteria", notes.len());
    }
    
    result
}

#[tauri::command]
pub async fn restore_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    log_info!("Restoring note with id: {}", id);
    let result = state.notes_manager.restore_note(&id).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(true) => log_debug!("Successfully restored note {}", id),
        Ok(false) => log_debug!("Note {} not found for restoration", id),
        Err(e) => log_error!("Error restoring note {}: {}", id, e),
    }
    
    result
}

#[tauri::command]
pub async fn permanently_delete_note(state: State<'_, Arc<AppState>>, id: String) -> Result<bool, String> {
    log_info!("Permanently deleting note with id: {}", id);
    let result = state.notes_manager.permanently_delete_note(&id).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(true) => {
            log_debug!("Successfully permanently deleted note {}", id);
            // Cleanup unused tags after permanent note deletion
            if let Err(e) = state.tags_manager.cleanup_unused_tags().await {
                log_error!("Failed to cleanup unused tags after permanently deleting note {}: {}", id, e);
            }
        },
        Ok(false) => log_debug!("Note {} not found for permanent deletion", id),
        Err(e) => log_error!("Error permanently deleting note {}: {}", id, e),
    }
    
    result
}

#[tauri::command]
pub async fn show_directory_dialog() -> Result<Option<String>, String> {
    log_info!("Showing directory dialog");
    use tauri::api::dialog::FileDialogBuilder;
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    FileDialogBuilder::new()
        .set_title("Select Data Directory")
        .pick_folder(move |path_buf| {
            let _ = tx.send(path_buf.map(|p| p.to_string_lossy().to_string()));
        });
    
    let result = rx.recv().map_err(|e| e.to_string());
    
    if let Ok(Some(path)) = &result {
        log_debug!("Selected directory: {}", path);
    }
    
    result
}

#[tauri::command]
pub async fn update_data_directory(state: State<'_, Arc<AppState>>, path: String) -> Result<bool, String> {
    log_info!("Updating data directory to: {}", path);
    use std::path::PathBuf;
    
    // Create a new config with updated data directory
    let new_path = PathBuf::from(path);
    let mut config_manager = state.config_manager.lock().unwrap();
    config_manager.update_data_directory(new_path)
        .map_err(|e| e.to_string())?;
    
    // Change working directory to new notes directory for relative path support
    let notes_directory = config_manager.get_notes_directory();
    if let Err(e) = std::env::set_current_dir(&notes_directory) {
        log_warn!("Failed to change working directory to {}: {}", notes_directory.display(), e);
    } else {
        log_info!("Changed working directory to: {}", notes_directory.display());
    }
    
    log_debug!("Data directory updated successfully");
    Ok(true)
}

#[tauri::command]
pub async fn reinitialize_data_directory(state: State<'_, Arc<AppState>>, path: String) -> Result<bool, String> {
    log_info!("Reinitializing data directory: {}", path);
    use std::path::PathBuf;
    use crate::database::DatabaseManager;
    
    // Create a new config with updated data directory
    let new_path = PathBuf::from(path);
    let mut config_manager = state.config_manager.lock().unwrap();
    config_manager.update_data_directory(new_path.clone())
        .map_err(|e| e.to_string())?;
    
    // Initialize database in the new directory
    let db_path = new_path.join("xnote.db");
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        DatabaseManager::new(&db_path).await
            .map_err(|e| format!("Failed to initialize database: {}", e))
    })?;
    
    // Change working directory to new notes directory for relative path support
    let notes_directory = config_manager.get_notes_directory();
    if let Err(e) = std::env::set_current_dir(&notes_directory) {
        log_warn!("Failed to change working directory to {}: {}", notes_directory.display(), e);
    } else {
        log_info!("Changed working directory to: {}", notes_directory.display());
    }
    
    log_debug!("Data directory reinitialized successfully");
    Ok(true)
}

#[tauri::command]
pub async fn mark_setup_complete(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Marking setup as complete");
    match state.config_manager.lock().unwrap().mark_setup_complete() {
        Ok(_) => {
            log_debug!("Setup marked as complete");
            
            // Change working directory to notes directory for relative path support
            let notes_directory = state.config_manager.lock().unwrap().get_notes_directory();
            if let Err(e) = std::env::set_current_dir(&notes_directory) {
                log_warn!("Failed to change working directory to {}: {}", notes_directory.display(), e);
            } else {
                log_info!("Changed working directory to: {}", notes_directory.display());
            }
            
            Ok(true)
        }
        Err(e) => {
            log_error!("Failed to mark setup as complete: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn sync_external_files(state: State<'_, Arc<AppState>>) -> Result<Vec<Note>, String> {
    log_info!("Syncing external files");
    let result = state.notes_manager.sync_external_files().await
        .map_err(|e| e.to_string());
    
    if let Ok(notes) = &result {
        log_debug!("Synced {} notes from external files", notes.len());
    }
    
    result
}

// Tag management commands
#[tauri::command]
pub async fn create_tag(state: State<'_, Arc<AppState>>, name: String) -> Result<Tag, String> {
    log_info!("Creating new tag: {}", name);
    let result = state.tags_manager.create_tag(&name).await
        .map_err(|e| e.to_string());
    
    if let Ok(tag) = &result {
        log_debug!("Successfully created tag with id: {}", tag.id);
    }
    
    result
}

#[tauri::command]
pub async fn delete_tag(state: State<'_, Arc<AppState>>, tag_id: String) -> Result<bool, String> {
    log_info!("Deleting tag with id: {}", tag_id);
    let result = state.tags_manager.delete_tag(&tag_id).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(true) => log_debug!("Successfully deleted tag {}", tag_id),
        Ok(false) => log_debug!("Tag {} not found for deletion", tag_id),
        Err(e) => log_error!("Error deleting tag {}: {}", tag_id, e),
    }
    
    result
}

#[tauri::command]
pub async fn rename_tag(state: State<'_, Arc<AppState>>, tag_id: String, new_name: String) -> Result<Option<Tag>, String> {
    log_info!("Renaming tag {} to: {}", tag_id, new_name);
    let result = state.tags_manager.rename_tag(&tag_id, &new_name).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(Some(tag)) => log_debug!("Successfully renamed tag to: {}", tag.name),
        Ok(None) => log_debug!("Tag {} not found for renaming", tag_id),
        Err(e) => log_error!("Error renaming tag {}: {}", tag_id, e),
    }
    
    result
}

#[tauri::command]
pub async fn add_tag_to_note(state: State<'_, Arc<AppState>>, note_id: String, tag_name: String) -> Result<Tag, String> {
    log_info!("Adding tag '{}' to note: {}", tag_name, note_id);
    let result = state.tags_manager.add_tag_to_note(&note_id, &tag_name).await
        .map_err(|e| e.to_string());
    
    if let Ok(tag) = &result {
        log_debug!("Successfully added tag '{}' to note {}", tag.name, note_id);
    }
    
    result
}

#[tauri::command]
pub async fn remove_tag_from_note(state: State<'_, Arc<AppState>>, note_id: String, tag_name: String) -> Result<bool, String> {
    log_info!("Removing tag '{}' from note: {}", tag_name, note_id);
    let result = state.tags_manager.remove_tag_from_note(&note_id, &tag_name).await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(true) => log_debug!("Successfully removed tag '{}' from note {}", tag_name, note_id),
        Ok(false) => log_debug!("Tag '{}' not found on note {}", tag_name, note_id),
        Err(e) => log_error!("Error removing tag '{}' from note {}: {}", tag_name, note_id, e),
    }
    
    result
}

#[tauri::command]
pub async fn get_note_tags(state: State<'_, Arc<AppState>>, note_id: String) -> Result<Vec<Tag>, String> {
    log_info!("Getting tags for note: {}", note_id);
    let result = state.tags_manager.get_note_tags(&note_id).await
        .map_err(|e| e.to_string());
    
    if let Ok(tags) = &result {
        log_debug!("Found {} tags for note {}", tags.len(), note_id);
    }
    
    result
}

#[tauri::command]
pub async fn search_tags(state: State<'_, Arc<AppState>>, query: String) -> Result<Vec<Tag>, String> {
    log_info!("Searching tags with query: {}", query);
    let result = state.tags_manager.search_tags(&query).await
        .map_err(|e| e.to_string());
    
    if let Ok(tags) = &result {
        log_debug!("Found {} tags matching '{}'", tags.len(), query);
    }
    
    result
}

#[tauri::command]
pub async fn cleanup_unused_tags(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    log_info!("Cleaning up unused tags");
    let result = state.tags_manager.cleanup_unused_tags().await
        .map_err(|e| e.to_string());
    
    match &result {
        Ok(count) => log_debug!("Successfully cleaned up {} unused tags", count),
        Err(e) => log_error!("Error cleaning up unused tags: {}", e),
    }
    
    result
}

#[tauri::command]
pub async fn save_image_to_attachments(state: State<'_, Arc<AppState>>, filename: String, data: Vec<u8>) -> Result<String, String> {
    log_info!("Saving image to attachments: {}", filename);
    
    use std::fs;
    
    // Get the notes directory from config
    let config_manager = state.config_manager.lock().unwrap();
    let notes_directory = config_manager.get_notes_directory();
    let attachments_dir = notes_directory.join("attachments");
    
    // Create attachments directory if it doesn't exist
    fs::create_dir_all(&attachments_dir)
        .map_err(|e| format!("Failed to create attachments directory: {}", e))?;
    
    // Write the file
    let file_path = attachments_dir.join(&filename);
    fs::write(&file_path, &data)
        .map_err(|e| format!("Failed to write image file: {}", e))?;
    
    // Return relative path for markdown
    let relative_path = format!("attachments/{}", filename);
    
    log_debug!("Successfully saved image to: {}", file_path.display());
    Ok(relative_path)
}

#[tauri::command]
pub async fn save_image_for_text(state: State<'_, Arc<AppState>>, filename: String, data: Vec<u8>) -> Result<String, String> {
    log_info!("Saving image for text content: {}", filename);
    
    use std::fs;
    
    // Get the notes directory from config
    let config_manager = state.config_manager.lock().unwrap();
    let notes_directory = config_manager.get_notes_directory();
    let images_dir = notes_directory.join("images");
    
    // Create images directory if it doesn't exist
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;
    
    // Write the file
    let file_path = images_dir.join(&filename);
    fs::write(&file_path, &data)
        .map_err(|e| format!("Failed to write image file: {}", e))?;
    
    // Return relative path for markdown
    let relative_path = format!("images/{}", filename);
    
    log_debug!("Successfully saved text image to: {}", file_path.display());
    Ok(relative_path)
}

#[tauri::command]
pub async fn get_data_directory(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    let config_manager = state.config_manager.lock().unwrap();
    let data_directory = config_manager.get_notes_directory();
    Ok(data_directory.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn add_attachment(state: State<'_, Arc<AppState>>, note_id: String, file_path: String) -> Result<String, String> {
    log_info!("Adding attachment {} to note {}", file_path, note_id);
    
    use std::path::Path;
    use std::fs;
    
    let source_path = Path::new(&file_path);
    
    // 验证源文件存在
    if !source_path.exists() {
        return Err(format!("Source file does not exist: {}", file_path));
    }
    
    // 获取文件信息
    let file_name = source_path.file_name()
        .ok_or("Invalid file path")?
        .to_string_lossy();
    
    let file_size = source_path.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len() as i64;
    
    // 获取MIME类型
    let mime_type = mime_guess::from_path(&source_path)
        .first()
        .map(|m| m.to_string());
    
    // 获取notes目录
    let notes_directory = {
        let config_manager = state.config_manager.lock().unwrap();
        config_manager.get_notes_directory()
    };
    let attachments_dir = notes_directory.join("attachments");
    
    // 创建attachments目录（如果不存在）
    fs::create_dir_all(&attachments_dir)
        .map_err(|e| format!("Failed to create attachments directory: {}", e))?;
    
    // 生成目标文件名（避免重名）
    let mut target_filename = file_name.to_string();
    let mut counter = 1;
    let target_path = loop {
        let test_path = attachments_dir.join(&target_filename);
        if !test_path.exists() {
            break test_path;
        }
        
        // 处理文件重名
        if let Some(stem) = source_path.file_stem() {
            if let Some(ext) = source_path.extension() {
                target_filename = format!("{}_{}.{}", 
                    stem.to_string_lossy(), 
                    counter, 
                    ext.to_string_lossy()
                );
            } else {
                target_filename = format!("{}_{}", stem.to_string_lossy(), counter);
            }
        } else {
            target_filename = format!("{}_{}", file_name, counter);
        }
        counter += 1;
    };
    
    let relative_path = format!("attachments/{}", target_filename);
    
    // 检查数据库中是否已存在此附件
    let db_manager = state.notes_manager.get_database_manager();
    let attachment_record = db_manager.get_attachment_by_path(&relative_path).await
        .map_err(|e| format!("Failed to check existing attachment: {}", e))?;
    
    let attachment_id = if let Some(existing) = attachment_record {
        // 附件已存在，直接使用
        existing.id
    } else {
        // 复制文件
        fs::copy(&source_path, &target_path)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
        
        // 创建新的附件记录
        let attachment_id = uuid::Uuid::new_v4().to_string();
        db_manager.create_attachment(
            &attachment_id,
            &target_filename,
            &relative_path,
            file_size,
            mime_type.as_deref()
        ).await
        .map_err(|e| format!("Failed to create attachment record: {}", e))?;
        
        attachment_id
    };
    
    // 将附件关联到笔记
    db_manager.add_attachment_to_note(&note_id, &attachment_id).await
        .map_err(|e| format!("Failed to link attachment to note: {}", e))?;
    
    log_debug!("Successfully added attachment: {}", relative_path);
    
    Ok(relative_path)
}

#[tauri::command]
pub async fn list_attachments(state: State<'_, Arc<AppState>>, note_id: String) -> Result<Vec<String>, String> {
    log_info!("Listing attachments for note {}", note_id);
    
    let db_manager = state.notes_manager.get_database_manager();
    let attachments = db_manager.get_note_attachments(&note_id).await
        .map_err(|e| format!("Failed to get note attachments: {}", e))?;
    
    let attachment_paths: Vec<String> = attachments.into_iter()
        .map(|a| a.file_path)
        .collect();
    
    log_debug!("Found {} attachments for note {}", attachment_paths.len(), note_id);
    
    Ok(attachment_paths)
}

#[tauri::command]
pub async fn delete_attachment(state: State<'_, Arc<AppState>>, note_id: String, relative_path: String) -> Result<bool, String> {
    log_info!("Deleting attachment {} from note {}", relative_path, note_id);
    
    use std::fs;
    
    let db_manager = state.notes_manager.get_database_manager();
    
    // 获取附件记录
    let attachment = db_manager.get_attachment_by_path(&relative_path).await
        .map_err(|e| format!("Failed to get attachment: {}", e))?;
    
    let attachment = match attachment {
        Some(a) => a,
        None => return Err("Attachment not found in database".to_string()),
    };
    
    // 从笔记中移除附件引用
    db_manager.remove_attachment_from_note(&note_id, &attachment.id).await
        .map_err(|e| format!("Failed to remove attachment from note: {}", e))?;
    
    // 检查是否还有其他引用
    let updated_attachment = db_manager.get_attachment_by_path(&relative_path).await
        .map_err(|e| format!("Failed to check attachment references: {}", e))?;
    
    if let Some(att) = updated_attachment {
        if att.reference_count <= 0 {
            // 没有引用了，删除文件和数据库记录
            let notes_directory = {
                let config_manager = state.config_manager.lock().unwrap();
                config_manager.get_notes_directory()
            };
            let file_path = notes_directory.join(&relative_path);
            
            if file_path.exists() {
                fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete attachment file: {}", e))?;
            }
            
            db_manager.delete_attachment(&attachment.id).await
                .map_err(|e| format!("Failed to delete attachment record: {}", e))?;
            
            log_debug!("Successfully deleted attachment file and record: {}", relative_path);
        } else {
            log_debug!("Attachment {} still has {} references, keeping file", relative_path, att.reference_count);
        }
    }
    
    Ok(true)
}

#[tauri::command]
pub async fn open_attachment(app_handle: tauri::AppHandle, state: State<'_, Arc<AppState>>, relative_path: String) -> Result<(), String> {
    log_info!("Opening attachment: {}", relative_path);
    
    let config_manager = state.config_manager.lock().unwrap();
    let notes_directory = config_manager.get_notes_directory();
    let file_path = notes_directory.join(&relative_path);
    
    if !file_path.exists() {
        return Err("Attachment not found".to_string());
    }
    
    // 使用系统默认程序打开文件
    use tauri::api::shell;
    shell::open(&app_handle.shell_scope(), file_path.to_string_lossy().as_ref(), None)
        .map_err(|e| format!("Failed to open attachment: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn export_note(state: State<'_, Arc<AppState>>, note_id: String, export_path: String) -> Result<String, String> {
    log_info!("Exporting note {} to {}", note_id, export_path);
    
    // Get note data
    let note = match state.notes_manager.get_note(&note_id).await {
        Ok(Some(note)) => note,
        Ok(None) => return Err("Note not found".to_string()),
        Err(e) => return Err(format!("Failed to get note: {}", e)),
    };
    
    // Create export directory
    let export_dir = PathBuf::from(&export_path).join(format!("{}-xnote-export", sanitize_filename(&note.title)));
    
    if let Err(e) = fs::create_dir_all(&export_dir) {
        return Err(format!("Failed to create export directory: {}", e));
    }
    
    // Export markdown file
    let md_file_path = export_dir.join(format!("{}.md", sanitize_filename(&note.title)));
    if let Err(e) = fs::write(&md_file_path, &note.content) {
        return Err(format!("Failed to write markdown file: {}", e));
    }
    
    // Create attachments directory
    let attachments_dir = export_dir.join("attachments");
    if let Err(e) = fs::create_dir_all(&attachments_dir) {
        return Err(format!("Failed to create attachments directory: {}", e));
    }
    
    // Get source data directory
    let config_manager = state.config_manager.lock().unwrap();
    let notes_directory = config_manager.get_notes_directory();
    let _source_attachments_dir = notes_directory.join("attachments");
    
    // Extract attachment references from markdown content
    let attachment_refs = extract_attachment_references(&note.content);
    let mut exported_count = 0;
    
    // Copy referenced attachments
    for attachment_ref in attachment_refs {
        let source_path = notes_directory.join(&attachment_ref);
        
        if source_path.exists() {
            // Extract filename from the reference path
            if let Some(filename) = Path::new(&attachment_ref).file_name() {
                let dest_path = attachments_dir.join(filename);
                
                if let Err(e) = fs::copy(&source_path, &dest_path) {
                    log_warn!("Failed to copy attachment {}: {}", attachment_ref, e);
                } else {
                    exported_count += 1;
                    log_debug!("Copied attachment: {} -> {}", source_path.display(), dest_path.display());
                }
            }
        } else {
            log_warn!("Attachment not found: {}", source_path.display());
        }
    }
    
    let export_summary = format!(
        "Note exported successfully to: {}\nMarkdown file: {}.md\nAttachments exported: {}",
        export_dir.display(),
        sanitize_filename(&note.title),
        exported_count
    );
    
    log_info!("Export completed: {}", export_summary);
    Ok(export_summary)
}

#[tauri::command]
pub async fn show_export_dialog() -> Result<Option<String>, String> {
    log_info!("Showing export directory dialog");
    use tauri::api::dialog::FileDialogBuilder;
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    FileDialogBuilder::new()
        .set_title("Select Export Directory")
        .pick_folder(move |path_buf| {
            let _ = tx.send(path_buf.map(|p| p.to_string_lossy().to_string()));
        });
    
    let result = rx.recv().map_err(|e| e.to_string());
    
    if let Ok(Some(path)) = &result {
        log_debug!("Selected export directory: {}", path);
    }
    
    result
}

// Helper function to sanitize filename
fn sanitize_filename(title: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized = String::new();
    
    for char in title.chars() {
        if invalid_chars.contains(&char) {
            sanitized.push('_');
        } else if char.is_control() {
            sanitized.push('_');
        } else {
            sanitized.push(char);
        }
    }
    
    if sanitized.trim().is_empty() {
        sanitized = "Untitled".to_string();
    }
    
    // Truncate long filenames
    if sanitized.len() > 100 {
        sanitized.truncate(100);
    }
    
    sanitized.trim().to_string()
}

// Helper function to extract attachment references from markdown content
fn extract_attachment_references(content: &str) -> Vec<String> {
    let mut attachments = Vec::new();
    
    // Regex patterns for different markdown image/link formats
    let patterns = [
        // ![alt](attachments/filename)
        r"!\[.*?\]\((attachments/[^)]+)\)",
        // [text](attachments/filename)
        r"\[.*?\]\((attachments/[^)]+)\)",
        // ![alt](images/filename) - for images saved via save_image_for_text
        r"!\[.*?\]\((images/[^)]+)\)",
        // [text](images/filename)
        r"\[.*?\]\((images/[^)]+)\)",
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(content) {
                if let Some(path) = cap.get(1) {
                    attachments.push(path.as_str().to_string());
                }
            }
        }
    }
    
    // Remove duplicates
    attachments.sort();
    attachments.dedup();
    
    attachments
}

#[tauri::command]
pub async fn cleanup_unreferenced_attachments(state: State<'_, Arc<AppState>>) -> Result<usize, String> {
    log_info!("Cleaning up unreferenced attachments");
    
    use std::fs;
    
    let db_manager = state.notes_manager.get_database_manager();
    let unreferenced = db_manager.get_unreferenced_attachments().await
        .map_err(|e| format!("Failed to get unreferenced attachments: {}", e))?;
    
    let notes_directory = {
        let config_manager = state.config_manager.lock().unwrap();
        config_manager.get_notes_directory()
    };
    
    let mut cleaned_count = 0;
    
    for attachment in unreferenced {
        let file_path = notes_directory.join(&attachment.file_path);
        
        // Delete file if it exists
        if file_path.exists() {
            if let Err(e) = fs::remove_file(&file_path) {
                log_warn!("Failed to delete unreferenced attachment file {}: {}", attachment.file_path, e);
                continue;
            }
        }
        
        // Delete database record
        if let Err(e) = db_manager.delete_attachment(&attachment.id).await {
            log_warn!("Failed to delete unreferenced attachment record {}: {}", attachment.id, e);
            continue;
        }
        
        cleaned_count += 1;
        log_debug!("Cleaned up unreferenced attachment: {}", attachment.file_path);
    }
    
    log_info!("Cleaned up {} unreferenced attachments", cleaned_count);
    Ok(cleaned_count)
}

// Git Sync Commands
#[tauri::command]
pub async fn get_app_config(state: State<'_, Arc<AppState>>) -> Result<crate::config::AppConfig, String> {
    log_info!("Getting app config");
    let config_manager = state.config_manager.lock().unwrap();
    Ok(config_manager.get_config().clone())
}

#[tauri::command]
pub async fn get_git_sync_config(state: State<'_, Arc<AppState>>) -> Result<Option<crate::config::GitSyncConfig>, String> {
    log_info!("Getting git sync config");
    let config_manager = state.config_manager.lock().unwrap();
    Ok(config_manager.get_config().git_sync.clone())
}

#[tauri::command]
pub async fn update_git_sync_config(state: State<'_, Arc<AppState>>, config: crate::config::GitSyncConfig) -> Result<bool, String> {
    log_info!("Updating git sync config");
    let mut config_manager = state.config_manager.lock().unwrap();
    let mut app_config = config_manager.get_config().clone();
    app_config.git_sync = Some(config);
    
    config_manager.update_config(app_config)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn update_theme(state: State<'_, Arc<AppState>>, theme: String) -> Result<bool, String> {
    log_info!("Updating theme to: {}", theme);
    let mut config_manager = state.config_manager.lock().unwrap();
    let mut app_config = config_manager.get_config().clone();
    app_config.theme = theme;
    
    config_manager.update_config(app_config)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, Arc<AppState>>) -> Result<crate::sync::SyncStatus, String> {
    log_info!("Getting sync status");
    println!("🔄 Getting sync status...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    let status = sync_manager.get_sync_status()
        .map_err(|e| e.to_string())?;
    
    println!("📊 Sync status: {} local changes, {} remote changes", 
             status.local_changes, status.remote_changes);
    
    Ok(status)
}

#[tauri::command]
pub async fn get_local_changes(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::sync::SyncDiff>, String> {
    log_info!("Getting local changes");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.get_local_changes()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn perform_sync(state: State<'_, Arc<AppState>>) -> Result<crate::sync::SyncResult, String> {
    log_info!("Performing sync");
    println!("🚀 Starting sync operation...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    // Initialize repository if needed
    sync_manager.initialize_repository()
        .map_err(|e| format!("Failed to initialize repository: {}", e))?;
    
    sync_manager.perform_sync()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn commit_changes(state: State<'_, Arc<AppState>>, message: String) -> Result<bool, String> {
    log_info!("Committing changes with message: {}", message);
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.commit_changes(&message)
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

// 新的Git操作命令
#[tauri::command]
pub async fn stash_changes(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Stashing local changes");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.stash_changes()
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn stash_pop(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Popping stashed changes");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.stash_pop()
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn pull_from_remote(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Pulling from remote");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.pull_from_remote()
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn pull_rebase(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Pulling with rebase");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.pull_rebase()
        .map_err(|e| e.to_string())?;
    
    Ok(true)
}

#[tauri::command]
pub async fn push_to_remote(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Pushing to remote");
    println!("🚀 Pushing changes to remote repository...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.push_to_remote()
        .map_err(|e| e.to_string())?;
    
    println!("✅ Successfully pushed to remote repository");
    Ok(true)
}

#[tauri::command]
pub async fn smart_push_to_remote(state: State<'_, Arc<AppState>>) -> Result<bool, String> {
    log_info!("Smart pushing to remote with auto-rebase");
    println!("🚀 Smart pushing changes to remote repository...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    // 使用智能推送，包含自动 rebase 重试
    sync_manager.push_to_remote()
        .map_err(|e| e.to_string())?;
    
    println!("✅ Successfully smart pushed to remote repository");
    Ok(true)
}

#[tauri::command]
pub async fn get_remote_commits(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::sync::RemoteCommit>, String> {
    log_info!("Getting remote commits");
    println!("🔄 Getting remote commits...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.get_remote_commits()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_commit_history(state: State<'_, Arc<AppState>>) -> Result<Vec<crate::sync::RemoteCommit>, String> {
    log_info!("Getting commit history");
    println!("🔄 Getting commit history...");
    
    let config_manager = state.config_manager.lock().unwrap();
    let git_config = match &config_manager.get_config().git_sync {
        Some(config) => config.clone(),
        None => return Err("Git sync not configured".to_string()),
    };
    
    let notes_directory = config_manager.get_notes_directory();
    let sync_manager = crate::sync::GitSyncManager::new(notes_directory, git_config);
    
    sync_manager.get_commit_history(10) // Get last 10 commits
        .map_err(|e| e.to_string())
}