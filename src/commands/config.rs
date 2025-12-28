use tauri::State;
use std::path::PathBuf;
use anyhow::Result;

use crate::{AppState, config::AppConfig};

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn set_data_directory(
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    
    // Update config
    {
        let mut config = state.config.lock().await;
        config.set_data_directory(path_buf.clone())
            .map_err(|e| e.to_string())?;
    }
    
    // Initialize database with new path
    {
        let config = state.config.lock().await;
        if let Some(db_path) = config.get_database_path() {
            let mut db = state.db.lock().await;
            db.initialize(&db_path).await
                .map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn is_first_run(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock().await;
    Ok(config.is_first_run())
}

#[tauri::command]
pub async fn update_config(
    new_config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    *config = new_config;
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}