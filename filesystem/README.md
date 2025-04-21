# Filesystem MCP Server

A Rust implementation of the Model Context Protocol (MCP) for filesystem operations.

## Features

* Read/write files
* Create/list/delete directories
* Move files/directories
* Search files
* Get file metadata

**Note**: The server will only allow operations within directories specified via command-line arguments.

## API

### Resources

* `file://system`: File system operations interface

### Tools

* **read_file**
   * Read complete contents of a file
   * Input: `path` (string)
   * Reads complete file contents with UTF-8 encoding

* **read_multiple_files**
   * Read multiple files simultaneously
   * Input: `paths` (string[])
   * Failed reads won't stop the entire operation

* **write_file**
   * Create new file or overwrite existing (exercise caution with this)
   * Inputs:
      * `path` (string): File location
      * `content` (string): File content

* **edit_file**
   * Make line-based edits to a text file
   * Inputs:
      * `path` (string): File to edit
      * `edits` (array): Array of edit objects with `oldText` and `newText` properties
      * `dryRun` (boolean, optional): Preview changes without applying them

* **create_directory**
   * Create new directory or ensure it exists
   * Input: `path` (string)
   * Creates parent directories if needed
   * Succeeds silently if directory exists

* **list_directory**
   * List directory contents with [FILE] or [DIR] prefixes
   * Input: `path` (string)

* **directory_tree**
   * Get a recursive tree view of files and directories
   * Input: `path` (string)
   * Returns JSON structure with file/directory hierarchy

* **move_file**
   * Move or rename files and directories
   * Inputs:
      * `source` (string)
      * `destination` (string)
   * Fails if destination exists

* **search_files**
   * Recursively search for files/directories
   * Inputs:
      * `path` (string): Starting directory
      * `pattern` (string): Search pattern
      * `excludePatterns` (string[], optional): Patterns to exclude
   * Case-insensitive matching
   * Returns full paths to matches

* **get_file_info**
   * Get detailed file/directory metadata
   * Input: `path` (string)
   * Returns:
      * Size
      * Creation time
      * Modified time
      * Access time
      * Type (file/directory)
      * Permissions

* **list_allowed_directories**
   * List all directories the server is allowed to access
   * No input required
   * Returns:
      * Directories that this server can read/write from

## Building

```
cargo build --release
```

## Usage

```
./mcpx-filesystem /path/to/allowed/dir1 /path/to/allowed/dir2
```

## Usage with Claude Desktop

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "mcpx-filesystem",
      "args": [
        "/Users/username/Desktop",
        "/path/to/other/allowed/dir"
      ]
    }
  }
}
```

## Security

The server only allows operations within directories specified via command-line arguments. Any attempt to access paths outside these directories will be rejected.

## License

MIT
