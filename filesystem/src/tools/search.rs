use anyhow::{Result, anyhow};
use walkdir::WalkDir;
use crate::filesystem::FilesystemService;

pub async fn search_files(
    service: &FilesystemService, 
    path: &str, 
    pattern: &str, 
    exclude_patterns: Option<Vec<String>>
) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    let pattern = pattern.to_lowercase();
    let exclude_patterns = exclude_patterns.unwrap_or_default();
    let mut matches = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path_str = entry.path().to_string_lossy().to_string();
        
        // Skip excluded patterns
        if exclude_patterns.iter().any(|exclude| path_str.contains(exclude)) {
            continue;
        }
        
        let filename = entry.file_name().to_string_lossy().to_lowercase();
        
        if filename.contains(&pattern) {
            matches.push(path_str);
        }
    }

    // Convert to JSON string
    match serde_json::to_string_pretty(&matches) {
        Ok(json) => Ok(json),
        Err(e) => Err(anyhow!("Failed to serialize results: {}", e)),
    }
}
