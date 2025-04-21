use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::fs;
use std::path::Path;
use futures::future::BoxFuture;
use crate::filesystem::FilesystemService;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct DirectoryEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<DirectoryEntry>>,
}

pub async fn create_directory(service: &FilesystemService, path: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    fs::create_dir_all(path).await?;
    Ok(format!("Directory created successfully: {}", path))
}

pub async fn list_directory(service: &FilesystemService, path: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    let mut entries = fs::read_dir(path).await?;
    let mut result = format!("Contents of directory: {}\n", path);

    while let Some(entry) = entries.next_entry().await? {
        let file_type = entry.file_type().await?;
        let prefix = if file_type.is_dir() { "[DIR]" } else { "[FILE]" };
        result.push_str(&format!("{} {}\n", prefix, entry.file_name().to_string_lossy()));
    }

    Ok(result)
}

pub async fn directory_tree(service: &FilesystemService, path: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    let tree = build_directory_tree(service, path).await?;
    let json = serde_json::to_string_pretty(&tree)?;
    Ok(json)
}

fn build_directory_tree<'a>(
    service: &'a FilesystemService,
    path: &'a str,
) -> BoxFuture<'a, Result<DirectoryEntry>> {
    Box::pin(async move {
        let path_obj = Path::new(path);
        let name = path_obj.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());

        if !service.is_path_allowed(path) {
            return Err(anyhow!("Access to path '{}' is not allowed", path));
        }

        let metadata = fs::metadata(path).await?;
        
        if metadata.is_dir() {
            let mut children = Vec::new();
            let mut entries = fs::read_dir(path).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let child_path = entry.path().to_string_lossy().into_owned();
                match build_directory_tree(service, &child_path).await {
                    Ok(child_entry) => children.push(child_entry),
                    Err(e) => eprintln!("Error processing {}: {}", child_path, e),
                }
            }
            
            Ok(DirectoryEntry {
                name,
                entry_type: "directory".to_string(),
                children: Some(children),
            })
        } else {
            Ok(DirectoryEntry {
                name,
                entry_type: "file".to_string(),
                children: None,
            })
        }
    })
}

pub async fn move_file(service: &FilesystemService, source: &str, destination: &str) -> Result<String> {
    if !service.is_path_allowed(source) {
        return Err(anyhow!("Access to source path '{}' is not allowed", source));
    }
    
    if !service.is_path_allowed(destination) {
        return Err(anyhow!("Access to destination path '{}' is not allowed", destination));
    }

    // Check if destination exists
    if Path::new(destination).exists() {
        return Err(anyhow!("Destination already exists: {}", destination));
    }

    // Ensure parent directory of destination exists
    if let Some(parent) = Path::new(destination).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    fs::rename(source, destination).await?;
    Ok(format!("Successfully moved '{}' to '{}'", source, destination))
}
