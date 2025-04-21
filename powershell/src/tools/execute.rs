use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

/// Execute a PowerShell command synchronously and capture its output
pub async fn execute_command(command: String) -> Result<String> {
    // Create a PowerShell process with the command
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
       .arg("-NonInteractive")
       .arg("-Command")
       .arg(&command)
       .stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    log::info!("Executing PowerShell command: {}", command);
    
    // Execute the command and capture output
    let mut child = cmd.spawn()?;
    
    // Capture stdout
    let stdout_handle = child.stdout.take()
        .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let mut stdout_reader = BufReader::new(stdout_handle);
    let mut stdout = Vec::new();
    stdout_reader.read_to_end(&mut stdout).await?;
    
    // Capture stderr
    let stderr_handle = child.stderr.take()
        .ok_or_else(|| anyhow!("Failed to capture stderr"))?;
    let mut stderr_reader = BufReader::new(stderr_handle);
    let mut stderr = Vec::new();
    stderr_reader.read_to_end(&mut stderr).await?;
    
    // Wait for the process to complete and get the exit status
    let status = child.wait().await?;
    
    // Prepare the output
    let output = CommandOutput {
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
        exit_code: status.code(),
        success: status.success(),
    };
    
    // Convert to JSON
    Ok(serde_json::to_string_pretty(&output)?)
}

/// Execute a sequence of PowerShell commands in a single session
pub async fn execute_command_sequence(commands: Vec<String>) -> Result<String> {
    if commands.is_empty() {
        return Err(anyhow!("No commands provided to execute"));
    }
    
    // Join all commands with semicolons to execute in sequence
    let combined_command = commands.join("; ");
    
    // Execute the combined command
    execute_command(combined_command).await
}

/// Execute a PowerShell script file
pub async fn execute_script_file(script_path: String) -> Result<String> {
    // Validate that the file exists and has .ps1 extension
    let path = Path::new(&script_path);
    if !path.exists() {
        return Err(anyhow!("Script file does not exist: {}", script_path));
    }
    
    if path.extension().map_or(false, |ext| ext != "ps1") {
        return Err(anyhow!("File is not a PowerShell script (.ps1): {}", script_path));
    }
    
    // Create a PowerShell process to execute the script
    let mut cmd = Command::new("powershell.exe");
    cmd.arg("-NoProfile")
       .arg("-NonInteractive")
       .arg("-File")
       .arg(&script_path)
       .stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    log::info!("Executing PowerShell script: {}", script_path);
    
    // Execute the script and capture output
    let mut child = cmd.spawn()?;
    
    // Capture stdout
    let stdout_handle = child.stdout.take()
        .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let mut stdout_reader = BufReader::new(stdout_handle);
    let mut stdout = Vec::new();
    stdout_reader.read_to_end(&mut stdout).await?;
    
    // Capture stderr
    let stderr_handle = child.stderr.take()
        .ok_or_else(|| anyhow!("Failed to capture stderr"))?;
    let mut stderr_reader = BufReader::new(stderr_handle);
    let mut stderr = Vec::new();
    stderr_reader.read_to_end(&mut stderr).await?;
    
    // Wait for the process to complete and get the exit status
    let status = child.wait().await?;
    
    // Prepare the output
    let output = CommandOutput {
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
        exit_code: status.code(),
        success: status.success(),
    };
    
    // Convert to JSON
    Ok(serde_json::to_string_pretty(&output)?)
}
