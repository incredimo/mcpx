# PowerShell MCP Server

A Rust implementation of the Model Context Protocol (MCP) for PowerShell command execution.

## Features

* Execute PowerShell commands synchronously
* Run PowerShell commands as background processes
* Monitor and retrieve output from running processes
* Kill running processes
* Execute command sequences in a single PowerShell session
* Execute PowerShell script files
* Optional restricted mode for enhanced security

## API

### Resources

* `powershell://system`: PowerShell command execution interface

### Tools

* **execute_command**
   * Execute a PowerShell command synchronously
   * Input: `command` (string)
   * Returns: Command output with stdout, stderr, and exit code

* **start_background_process**
   * Start a PowerShell command as a background process
   * Input: `command` (string)
   * Returns: A process ID to track the running process

* **get_process_status**
   * Check the status of a background process
   * Input: `process_id` (string)
   * Returns: Status information including whether the process is running, exit code, and timestamps

* **kill_process**
   * Terminate a running PowerShell process
   * Input: `process_id` (string)
   * Returns: Confirmation of process termination

* **get_process_output**
   * Retrieve the current output of a background process
   * Input: `process_id` (string)
   * Returns: Current stdout and stderr content from the process

* **execute_command_sequence**
   * Execute multiple commands in a single PowerShell session
   * Input: `commands` (array of strings)
   * Returns: Combined output from all commands

* **list_running_processes**
   * List all background processes managed by the server
   * Returns: Array of process information objects

* **execute_script_file**
   * Execute a PowerShell script file (.ps1)
   * Input: `script_path` (string)
   * Returns: Output from script execution

## Building

```
cargo build --release
```

## Usage

### Basic Usage

```
mcpx-powershell
```

### Restricted Mode

In restricted mode, only specified commands are allowed to run:

```
mcpx-powershell --restricted --allow="Get-Process" --allow="Get-Service"
```

## Usage with Claude Desktop

Add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "powershell": {
      "command": "mcpx-powershell",
      "args": []
    }
  }
}
```

For restricted mode:

```json
{
  "mcpServers": {
    "powershell": {
      "command": "mcpx-powershell",
      "args": ["--restricted", "--allow=Get-Process", "--allow=Get-Service"]
    }
  }
}
```

## Security Considerations

* Use restricted mode to limit which PowerShell commands can be executed
* Be cautious when running as an elevated user, as PowerShell commands will have those privileges
* All executed commands are logged for audit purposes

## License

MIT
