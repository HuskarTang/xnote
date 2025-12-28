use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use pulldown_cmark::{Parser, Options, html};

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

pub fn generate_unique_filename(dir: &Path, base_name: &str, extension: &str) -> PathBuf {
    let sanitized_base = sanitize_filename(base_name);
    let mut filename = format!("{}.{}", sanitized_base, extension);
    let mut path = dir.join(&filename);
    let mut counter = 1;
    
    while path.exists() {
        filename = format!("{}_{}.{}", sanitized_base, counter, extension);
        path = dir.join(&filename);
        counter += 1;
    }
    
    path
}

pub fn extract_title_from_markdown(content: &str) -> String {
    // Try to find the first heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }
    
    // If no heading found, use the first non-empty line
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return trimmed.chars().take(50).collect::<String>() + "...";
        }
    }
    
    "Untitled".to_string()
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn save_image_from_base64(
    base64_data: &str,
    attachments_dir: &Path,
    original_name: Option<&str>,
) -> Result<PathBuf> {
    // Remove data URL prefix if present
    let base64_clean = if base64_data.starts_with("data:") {
        base64_data
            .split(',')
            .nth(1)
            .context("Invalid base64 data URL")?
    } else {
        base64_data
    };
    
    // Decode base64
    let image_data = base64::decode(base64_clean)
        .context("Failed to decode base64 image")?;
    
    // Generate filename
    let timestamp = Utc::now().timestamp();
    let filename = if let Some(name) = original_name {
        let sanitized = sanitize_filename(name);
        if sanitized.contains('.') {
            sanitized
        } else {
            format!("{}.png", sanitized)
        }
    } else {
        format!("image-{}.png", timestamp)
    };
    
    let file_path = generate_unique_filename(attachments_dir, &filename.replace('.', "_"), "png");
    
    // Save file
    fs::write(&file_path, image_data)
        .context("Failed to save image file")?;
    
    Ok(file_path)
}

pub fn get_file_size(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)
        .context("Failed to get file metadata")?;
    Ok(metadata.len())
}

pub fn get_mime_type_from_extension(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "html" => "text/html",
        "json" => "application/json",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))
}

pub fn write_file_content(path: &Path, content: &str) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}