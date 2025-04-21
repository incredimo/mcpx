use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use tokio::fs;
use std::path::Path;
use crate::filesystem::FilesystemService;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Edit {
    pub old_text: String,
    pub new_text: String,
}

pub async fn write_file(service: &FilesystemService, path: &str, content: &str) -> Result<String> {
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    // Ensure the parent directory exists
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    fs::write(path, content).await?;
    Ok(format!("Successfully wrote to file: {}", path))
}

pub async fn edit_file(
    service: &FilesystemService, 
    path: &str, 
    edits: &[Edit], 
    dry_run: Option<bool>
) -> Result<String> {
    let dry_run = dry_run.unwrap_or(false);
    
    if !service.is_path_allowed(path) {
        return Err(anyhow!("Access to path '{}' is not allowed", path));
    }

    // Read the original file content
    let original_content = fs::read_to_string(path).await?;
    let mut new_content = original_content.clone();

    // Apply all edits
    for edit in edits {
        new_content = new_content.replace(&edit.old_text, &edit.new_text);
    }

    // Generate a simple diff
    let diff = generate_diff(&original_content, &new_content);

    // If it's not a dry run, write the changes
    if !dry_run {
        fs::write(path, new_content).await?;
        Ok(format!("File edited successfully: {}\n\nChanges:\n{}", path, diff))
    } else {
        Ok(format!("Dry run - no changes made. Diff:\n{}", diff))
    }
}

fn generate_diff(original: &str, modified: &str) -> String {
    let original_lines: Vec<&str> = original.lines().collect();
    let modified_lines: Vec<&str> = modified.lines().collect();
    
    let mut diff = String::new();
    
    // First handle lines that are present in both versions
    for (i, (orig, modified)) in original_lines.iter().zip(modified_lines.iter()).enumerate() {
        if orig != modified {
            diff.push_str(&format!("Line {}: \n- {}\n+ {}\n", i + 1, orig, modified));
        }
    }
    
    // Handle added lines (if modified has more lines than original)
    if original_lines.len() < modified_lines.len() {
        for (i, new) in modified_lines.iter().enumerate().skip(original_lines.len()) {
            diff.push_str(&format!("Line {}: \n+ {}\n", i + 1, new));
        }
    } 
    // Handle removed lines (if original has more lines than modified)
    else if original_lines.len() > modified_lines.len() {
        for (i, orig) in original_lines.iter().enumerate().skip(modified_lines.len()) {
            diff.push_str(&format!("Line {}: \n- {}\n", i + 1, orig));
        }
    }
    
    if diff.is_empty() {
        diff = "No changes detected.".to_string();
    }
    
    diff
}
