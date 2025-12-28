use tauri::State;
use std::path::PathBuf;
use chrono::Utc;
use anyhow::Result;

use crate::{
    AppState,
    models::AttachmentInfo,
    utils::{save_image_from_base64, get_file_size, get_mime_type_from_extension, generate_unique_filename},
};

#[tauri::command]
pub async fn save_attachment(
    file_data: String,
    original_name: Option<String>,
    is_base64: bool,
    state: State<'_, AppState>,
) -> Result<AttachmentInfo, String> {
    let config = state.config.lock().await;
    let attachments_dir = config.get_attachments_directory()
        .ok_or("Attachments directory not configured")?;
    
    let file_path = if is_base64 {
        // Handle base64 encoded data (typically images)
        save_image_from_base64(&file_data, &attachments_dir, original_name.as_deref())
            .map_err(|e| e.to_string())?
    } else {
        // Handle file path (for drag & drop files)
        let source_path = PathBuf::from(&file_data);
        if !source_path.exists() {
            return Err("Source file does not exist".to_string());
        }
        
        let filename = original_name.unwrap_or_else(|| {
            source_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });
        
        let target_path = generate_unique_filename(&attachments_dir, &filename, "");
        
        // Copy file to attachments directory
        std::fs::copy(&source_path, &target_path)
            .map_err(|e| e.to_string())?;
        
        target_path
    };
    
    // Get file information
    let file_size = get_file_size(&file_path)
        .map_err(|e| e.to_string())?;
    
    let extension = file_path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let mime_type = get_mime_type_from_extension(&extension);
    
    let relative_path = file_path.strip_prefix(&attachments_dir)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();
    
    let attachment_info = AttachmentInfo {
        id: uuid::Uuid::new_v4().to_string(),
        original_name: original_name.unwrap_or_else(|| {
            file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        }),
        file_path: relative_path,
        file_size,
        mime_type: mime_type.to_string(),
        created_at: Utc::now(),
    };
    
    Ok(attachment_info)
}

#[tauri::command]
pub async fn get_attachment_path(
    relative_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let attachments_dir = config.get_attachments_directory()
        .ok_or("Attachments directory not configured")?;
    
    let full_path = attachments_dir.join(&relative_path);
    
    if !full_path.exists() {
        return Err("Attachment file not found".to_string());
    }
    
    Ok(full_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn save_image_from_clipboard(
    image_data: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let attachments_dir = config.get_attachments_directory()
        .ok_or("Attachments directory not configured")?;
    
    let file_path = save_image_from_base64(&image_data, &attachments_dir, None)
        .map_err(|e| e.to_string())?;
    
    let relative_path = file_path.strip_prefix(&attachments_dir)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .to_string();
    
    // Return markdown image syntax
    let filename = file_path.file_name()
        .unwrap_or_default()
        .to_string_lossy();
    
    Ok(format!("![{}](attachments/{})", filename, relative_path))
}

#[tauri::command]
pub async fn delete_attachment(
    relative_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = state.config.lock().await;
    let attachments_dir = config.get_attachments_directory()
        .ok_or("Attachments directory not configured")?;
    
    let full_path = attachments_dir.join(&relative_path);
    
    if full_path.exists() {
        std::fs::remove_file(&full_path)
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn list_attachments(
    state: State<'_, AppState>,
) -> Result<Vec<AttachmentInfo>, String> {
    let config = state.config.lock().await;
    let attachments_dir = config.get_attachments_directory()
        .ok_or("Attachments directory not configured")?;
    
    if !attachments_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut attachments = Vec::new();
    
    for entry in std::fs::read_dir(&attachments_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let metadata = entry.metadata().map_err(|e| e.to_string())?;
            let file_size = metadata.len();
            
            let filename = path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let extension = path.extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let mime_type = get_mime_type_from_extension(&extension);
            
            let relative_path = path.strip_prefix(&attachments_dir)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string();
            
            let created_at = metadata.created()
                .map(|time| {
                    let duration = time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                        .unwrap_or_else(|| Utc::now())
                })
                .unwrap_or_else(|| Utc::now());
            
            attachments.push(AttachmentInfo {
                id: uuid::Uuid::new_v4().to_string(),
                original_name: filename,
                file_path: relative_path,
                file_size,
                mime_type: mime_type.to_string(),
                created_at,
            });
        }
    }
    
    // Sort by creation time (newest first)
    attachments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    
    Ok(attachments)
}