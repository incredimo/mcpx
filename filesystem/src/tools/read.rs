use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::fs;
use crate::filesystem::FilesystemService;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct FileContent {
    pub path: String,
    pub content: Option<String>,
    pub error: Option<String>,
}

pub async fn read_file(service: &FilesystemService, path: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    match fs::read_to_string(path).await {
        Ok(content) => Ok(content),
        Err(e) => Err(anyhow!("Failed to read file '{}': {}", path, e)),
    }
}

pub async fn read_multiple_files(service: &FilesystemService, paths: Vec<String>) -> Result<String> {
    let mut results = Vec::new();

    for path in paths {
        if !service.is_path_allowed(&path) {
            results.push(FileContent {
                path: path.clone(),
                content: None,
                error: Some(format!("Access to path '{}' is not allowed", path)),
            });
            continue;
        }

        match fs::read_to_string(&path).await {
            Ok(content) => {
                results.push(FileContent {
                    path: path.clone(),
                    content: Some(content),
                    error: None,
                });
            }
            Err(e) => {
                results.push(FileContent {
                    path: path.clone(),
                    content: None,
                    error: Some(format!("Failed to read file: {}", e)),
                });
            }
        }
    }

    // Convert to JSON string
    match serde_json::to_string_pretty(&results) {
        Ok(json) => Ok(json),
        Err(e) => Err(anyhow!("Failed to serialize results: {}", e)),
    }
}
