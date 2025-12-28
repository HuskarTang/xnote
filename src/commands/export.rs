use tauri::State;
use std::path::PathBuf;
use anyhow::Result;

use crate::{
    AppState,
    utils::{markdown_to_html, read_file_content},
};

#[tauri::command]
pub async fn export_note_as_html(
    note_id: String,
    export_path: String,
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
    
    // Read markdown content
    let file_path = data_dir.join(&note.file_path);
    let markdown_content = read_file_content(&file_path)
        .map_err(|e| e.to_string())?;
    
    // Convert to HTML
    let html_content = markdown_to_html(&markdown_content);
    
    // Create full HTML document
    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            color: #333;
        }}
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 24px;
            margin-bottom: 16px;
            font-weight: 600;
            line-height: 1.25;
        }}
        h1 {{
            font-size: 2em;
            border-bottom: 1px solid #eaecef;
            padding-bottom: 10px;
        }}
        h2 {{
            font-size: 1.5em;
            border-bottom: 1px solid #eaecef;
            padding-bottom: 8px;
        }}
        code {{
            background-color: #f6f8fa;
            border-radius: 3px;
            font-size: 85%;
            margin: 0;
            padding: 0.2em 0.4em;
        }}
        pre {{
            background-color: #f6f8fa;
            border-radius: 6px;
            font-size: 85%;
            line-height: 1.45;
            overflow: auto;
            padding: 16px;
        }}
        blockquote {{
            border-left: 4px solid #dfe2e5;
            margin: 0;
            padding: 0 16px;
            color: #6a737d;
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
        }}
        th, td {{
            border: 1px solid #dfe2e5;
            padding: 6px 13px;
            text-align: left;
        }}
        th {{
            background-color: #f6f8fa;
            font-weight: 600;
        }}
        img {{
            max-width: 100%;
            height: auto;
        }}
        .export-info {{
            border-top: 1px solid #eaecef;
            margin-top: 40px;
            padding-top: 20px;
            font-size: 0.9em;
            color: #6a737d;
        }}
    </style>
</head>
<body>
    {}
    <div class="export-info">
        <p>Exported from MDNote on {}</p>
        <p>Original file: {}</p>
    </div>
</body>
</html>"#,
        note.title,
        html_content,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        note.file_path
    );
    
    // Write to export path
    let export_path = PathBuf::from(export_path);
    std::fs::write(&export_path, full_html)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn export_note_as_pdf(
    note_id: String,
    export_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // For PDF export, we would need a PDF generation library
    // For now, we'll return an error indicating this feature is not yet implemented
    Err("PDF export not yet implemented. Please use HTML export instead.".to_string())
}

#[tauri::command]
pub async fn export_note_as_markdown(
    note_id: String,
    export_path: String,
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
    
    // Read markdown content
    let file_path = data_dir.join(&note.file_path);
    let markdown_content = read_file_content(&file_path)
        .map_err(|e| e.to_string())?;
    
    // Write to export path
    let export_path = PathBuf::from(export_path);
    std::fs::write(&export_path, markdown_content)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn export_all_notes(
    export_dir: String,
    format: String, // "html", "markdown"
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let config = state.config.lock().await;
    let data_dir = config.get_data_directory()
        .ok_or("Data directory not configured")?;
    
    let db = state.db.lock().await;
    let notes = db.get_all_notes(false)
        .await
        .map_err(|e| e.to_string())?;
    
    let export_dir = PathBuf::from(export_dir);
    std::fs::create_dir_all(&export_dir)
        .map_err(|e| e.to_string())?;
    
    let mut exported_files = Vec::new();
    
    for note_with_tags in notes {
        let note = note_with_tags.note;
        let file_path = data_dir.join(&note.file_path);
        let markdown_content = read_file_content(&file_path)
            .map_err(|e| e.to_string())?;
        
        let export_filename = match format.as_str() {
            "html" => {
                let html_content = markdown_to_html(&markdown_content);
                let full_html = format!(
                    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body>
    {}
</body>
</html>"#,
                    note.title, html_content
                );
                
                let filename = format!("{}.html", crate::utils::sanitize_filename(&note.title));
                let export_path = export_dir.join(&filename);
                std::fs::write(&export_path, full_html)
                    .map_err(|e| e.to_string())?;
                filename
            }
            "markdown" | _ => {
                let filename = format!("{}.md", crate::utils::sanitize_filename(&note.title));
                let export_path = export_dir.join(&filename);
                std::fs::write(&export_path, markdown_content)
                    .map_err(|e| e.to_string())?;
                filename
            }
        };
        
        exported_files.push(export_filename);
    }
    
    Ok(exported_files)
}