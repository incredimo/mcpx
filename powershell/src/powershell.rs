use anyhow::Result;
use dashmap::DashMap;
use rmcp::{model::ServerInfo, ServerHandler, tool};
use std::sync::Arc;
use uuid::Uuid;

use crate::tools;

/// Main service for PowerShell command execution
#[derive(Debug, Clone)]
pub struct PowerShellService {
    pub allowed_commands: Vec<String>,
    pub restricted_mode: bool,
    pub running_processes: Arc<DashMap<String, tools::process::PowerShellProcess>>,
}

impl PowerShellService {
    pub fn new(args: &[String]) -> Self {
        // Parse arguments
        let mut restricted_mode = false;
        let mut allowed_commands = Vec::new();

        for arg in args {
            if arg == "--restricted" {
                restricted_mode = true;
            } else if arg.starts_with("--allow=") {
                let cmd = arg.trim_start_matches("--allow=").to_string();
                allowed_commands.push(cmd);
            }
        }

        Self {
            allowed_commands,
            restricted_mode,
            running_processes: Arc::new(DashMap::new()),
        }
    }

    /// Check if a command is allowed to run in restricted mode
    pub fn is_command_allowed(&self, command: &str) -> bool {
        if !self.restricted_mode {
            return true;
        }

        // If restricted mode is enabled, check against the allowed list
        self.allowed_commands.iter().any(|allowed| {
            command.starts_with(allowed) || command.contains(allowed)
        })
    }

    /// Generate a unique ID for a process
    pub fn generate_process_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

#[tool(tool_box)]
impl PowerShellService {
    /// Execute a PowerShell command synchronously and return the output
    #[tool(description = "Execute a PowerShell command and wait for it to complete. Returns the complete output of the command including standard output and error streams.")]
    async fn execute_command(&self, #[tool(param)] command: String) -> String {
        if !self.is_command_allowed(&command) {
            return format!("Error: Command '{}' is not allowed in restricted mode", command);
        }

        match tools::execute::execute_command(command).await {
            Ok(output) => output,
            Err(e) => format!("Error executing PowerShell command: {}", e),
        }
    }

    /// Start a PowerShell command as a background process
    #[tool(description = "Start a PowerShell command as a background process. Returns a process ID that can be used to check status or retrieve output later.")]
    async fn start_background_process(&self, #[tool(param)] command: String) -> String {
        if !self.is_command_allowed(&command) {
            return format!("Error: Command '{}' is not allowed in restricted mode", command);
        }

        match tools::process::start_background_process(self, command).await {
            Ok(process_id) => format!("{{\"process_id\": \"{}\", \"status\": \"started\"}}", process_id),
            Err(e) => format!("Error starting background process: {}", e),
        }
    }

    /// Get the status of a background process
    #[tool(description = "Check the status of a previously started background process. Returns information about whether the process is still running, its exit code if completed, and any available output.")]
    async fn get_process_status(&self, #[tool(param)] process_id: String) -> String {
        match tools::process::get_process_status(self, &process_id).await {
            Ok(status) => status,
            Err(e) => format!("Error checking process status: {}", e),
        }
    }

    /// Kill a running PowerShell process
    #[tool(description = "Terminate a running PowerShell process by its process ID. Force kills the process if it doesn't respond to a normal termination request.")]
    async fn kill_process(&self, #[tool(param)] process_id: String) -> String {
        match tools::process::kill_process(self, &process_id).await {
            Ok(result) => result,
            Err(e) => format!("Error killing process: {}", e),
        }
    }

    /// Attach to a background process and stream its output
    #[tool(description = "Attach to a background process and retrieve its current output. This does not wait for the process to complete, but returns whatever output is currently available.")]
    async fn get_process_output(&self, #[tool(param)] process_id: String) -> String {
        match tools::process::get_process_output(self, &process_id).await {
            Ok(output) => output,
            Err(e) => format!("Error retrieving process output: {}", e),
        }
    }

    /// Execute a sequence of PowerShell commands in the same session
    #[tool(description = "Execute a sequence of PowerShell commands in the same session, preserving state between commands. This is useful for multi-step operations where each step depends on previous steps.")]
    async fn execute_command_sequence(&self, #[tool(param)] commands: Vec<String>) -> String {
        // Check if all commands are allowed
        if self.restricted_mode {
            for cmd in &commands {
                if !self.is_command_allowed(cmd) {
                    return format!("Error: Command '{}' is not allowed in restricted mode", cmd);
                }
            }
        }

        match tools::execute::execute_command_sequence(commands).await {
            Ok(output) => output,
            Err(e) => format!("Error executing command sequence: {}", e),
        }
    }

    /// List all running background processes
    #[tool(description = "List all currently running background PowerShell processes that were started by this server. Returns the process IDs and their current status.")]
    async fn list_running_processes(&self) -> String {
        match tools::process::list_running_processes(self).await {
            Ok(processes) => processes,
            Err(e) => format!("Error listing processes: {}", e),
        }
    }

    /// Execute a PowerShell script file
    #[tool(description = "Execute a PowerShell script file (.ps1) at the specified path. Returns the output of the script execution.")]
    async fn execute_script_file(&self, #[tool(param)] script_path: String) -> String {
        // In restricted mode, we need to check the content of the script
        if self.restricted_mode {
            return format!("Error: Script execution is not allowed in restricted mode");
        }

        match tools::execute::execute_script_file(script_path).await {
            Ok(output) => output,
            Err(e) => format!("Error executing script file: {}", e),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for PowerShellService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides PowerShell command execution through the Model Context Protocol. It allows running PowerShell commands synchronously or as background processes, checking their status, and retrieving their output.".into()),
            ..Default::default()
        }
    }
}
