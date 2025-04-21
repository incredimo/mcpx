use rmcp::{model::ServerInfo, ServerHandler, tool};
use std::path::Path;

use crate::tools;

#[derive(Debug, Clone)]
pub struct FilesystemService {
    allowed_dirs: Vec<String>,
}

impl FilesystemService {
    pub fn new(allowed_dirs: Vec<String>) -> Self {
        Self { allowed_dirs }
    }

    pub fn is_path_allowed(&self, path: &str) -> bool {
        let path = Path::new(path);
        
        // Check if the path is within any of the allowed directories
        self.allowed_dirs.iter().any(|allowed_dir| {
            let allowed_path = Path::new(allowed_dir);
            path.starts_with(allowed_path)
        })
    }
}

#[tool(tool_box)]
impl FilesystemService {
    // Read operations
    #[tool(description = "Read the complete contents of a file from the file system. Handles various text encodings and provides detailed error messages if the file cannot be read. Use this tool when you need to examine the contents of a single file. Only works within allowed directories.")]
    async fn read_file(&self, #[tool(param)] path: String) -> String {
        match tools::read::read_file(self, &path).await {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Read the contents of multiple files simultaneously. This is more efficient than reading files one by one when you need to analyze or compare multiple files. Each file's content is returned with its path as a reference. Failed reads for individual files won't stop the entire operation. Only works within allowed directories.")]
    async fn read_multiple_files(&self, #[tool(param)] paths: Vec<String>) -> String {
        match tools::read::read_multiple_files(self, paths).await {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        }
    }

    // Write operations
    #[tool(description = "Create a new file or completely overwrite an existing file with new content. Use with caution as it will overwrite existing files without warning. Handles text content with proper encoding. Only works within allowed directories.")]
    async fn write_file(&self, #[tool(param)] path: String, #[tool(param)] content: String) -> String {
        match tools::write::write_file(self, &path, &content).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Make line-based edits to a text file. Each edit replaces exact line sequences with new content. Returns a git-style diff showing the changes made. Only works within allowed directories.")]
    async fn edit_file(
        &self, 
        #[tool(param)] path: String, 
        #[tool(param)] edits: Vec<tools::write::Edit>, 
        #[tool(param)] dry_run: Option<bool>
    ) -> String {
        match tools::write::edit_file(self, &path, &edits, dry_run).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    // Directory operations
    #[tool(description = "Create a new directory or ensure a directory exists. Can create multiple nested directories in one operation. If the directory already exists, this operation will succeed silently. Perfect for setting up directory structures for projects or ensuring required paths exist. Only works within allowed directories.")]
    async fn create_directory(&self, #[tool(param)] path: String) -> String {
        match tools::directory::create_directory(self, &path).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get a detailed listing of all files and directories in a specified path. Results clearly distinguish between files and directories with [FILE] and [DIR] prefixes. This tool is essential for understanding directory structure and finding specific files within a directory. Only works within allowed directories.")]
    async fn list_directory(&self, #[tool(param)] path: String) -> String {
        match tools::directory::list_directory(self, &path).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get a recursive tree view of files and directories as a JSON structure. Each entry includes 'name', 'type' (file/directory), and 'children' for directories. Files have no children array, while directories always have a children array (which may be empty). The output is formatted with 2-space indentation for readability. Only works within allowed directories.")]
    async fn directory_tree(&self, #[tool(param)] path: String) -> String {
        match tools::directory::directory_tree(self, &path).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Move or rename files and directories. Can move files between directories and rename them in a single operation. If the destination exists, the operation will fail. Works across different directories and can be used for simple renaming within the same directory. Both source and destination must be within allowed directories.")]
    async fn move_file(
        &self, 
        #[tool(param)] source: String, 
        #[tool(param)] destination: String
    ) -> String {
        match tools::directory::move_file(self, &source, &destination).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    // Search operations
    #[tool(description = "Recursively search for files and directories matching a pattern. Searches through all subdirectories from the starting path. The search is case-insensitive and matches partial names. Returns full paths to all matching items. Great for finding files when you don't know their exact location. Only searches within allowed directories.")]
    async fn search_files(
        &self, 
        #[tool(param)] path: String, 
        #[tool(param)] pattern: String,
        #[tool(param)] exclude_patterns: Option<Vec<String>>
    ) -> String {
        match tools::search::search_files(self, &path, &pattern, exclude_patterns).await {
            Ok(results) => results,
            Err(e) => format!("Error: {}", e),
        }
    }

    // File info operations
    #[tool(description = "Retrieve detailed metadata about a file or directory. Returns comprehensive information including size, creation time, last modified time, permissions, and type. This tool is perfect for understanding file characteristics without reading the actual content. Only works within allowed directories.")]
    async fn get_file_info(
        &self, 
        #[tool(param)] path: String
    ) -> String {
        match tools::info::get_file_info(self, &path).await {
            Ok(info) => info,
            Err(e) => format!("Error: {}", e),
        }
    }

    // Server info operations
    #[tool(description = "Returns the list of directories that this server is allowed to access. Use this to understand which directories are available before trying to access files.")]
    async fn list_allowed_directories(&self) -> String {
        serde_json::to_string_pretty(&self.allowed_dirs)
            .unwrap_or_else(|e| format!("Error serializing allowed directories: {}", e))
    }
}

#[tool(tool_box)]
impl ServerHandler for FilesystemService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides filesystem operations through the Model Context Protocol. It allows reading, writing, and managing files and directories, but only within the allowed directories specified when starting the server.".into()),
            ..Default::default()
        }
    }
}
