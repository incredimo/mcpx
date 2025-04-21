use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::fs;
use chrono::{DateTime, Utc};
use std::path::Path;
use crate::filesystem::FilesystemService;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub is_file: bool,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub accessed: Option<String>,
    pub permissions: Option<String>,
}

pub async fn get_file_info(service: &FilesystemService, path: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    let metadata = fs::metadata(path).await?;
    let path_obj = Path::new(path);
    
    let created = metadata.created().ok().map(|time| {
        DateTime::<Utc>::from(time).to_rfc3339()
    });
    
    let modified = metadata.modified().ok().map(|time| {
        DateTime::<Utc>::from(time).to_rfc3339()
    });
    
    let accessed = metadata.accessed().ok().map(|time| {
        DateTime::<Utc>::from(time).to_rfc3339()
    });
    
    // Get permissions in a cross-platform way
    let permissions = if metadata.permissions().readonly() {
        Some("read-only".to_string())
    } else {
        Some("read-write".to_string())
    };
    
    let file_info = FileInfo {
        path: path.to_string(),
        name: path_obj.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string()),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        size_bytes: metadata.len(),
        created,
        modified,
        accessed,
        permissions,
    };

    // Convert to JSON string
    match serde_json::to_string_pretty(&file_info) {
        Ok(json) => Ok(json),
        Err(e) => Err(anyhow!("Failed to serialize file info: {}", e)),
    }
}
