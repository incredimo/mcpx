use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::process::Stdio;
use tokio::process::Child;
use tokio::sync::Mutex;
use tokio::io::{AsyncReadExt, BufReader};
use std::sync::Arc;

use crate::powershell::PowerShellService;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProcessStatus {
    pub process_id: String,
    pub command: String,
    pub running: bool,
    pub exit_code: Option<i32>,
    pub start_time: String,
    pub end_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProcessOutput {
    pub process_id: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub completed: bool,
}

/// Structure to hold a running PowerShell process
#[derive(Debug)]
pub struct PowerShellProcess {
    pub process_id: String,
    pub command: String,
    pub process: Arc<Mutex<Child>>,
    pub stdout_buffer: Arc<Mutex<Vec<u8>>>,
    pub stderr_buffer: Arc<Mutex<Vec<u8>>>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_running: Arc<Mutex<bool>>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
}

/// Start a PowerShell command as a background process
pub async fn start_background_process(service: &PowerShellService, command: String) -> Result<String> {
    // Create a PowerShell process with the command
    let mut cmd = tokio::process::Command::new("powershell.exe");
    cmd.arg("-NoProfile")
       .arg("-NonInteractive")
       .arg("-Command")
       .arg(&command)
       .stdin(Stdio::null())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    log::info!("Starting background PowerShell process: {}", command);
    
    // Start the process
    let mut child = cmd.spawn()?;
    
    // Get stdout and stderr handles
    let stdout = child.stdout.take()
        .ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let stderr = child.stderr.take()
        .ok_or_else(|| anyhow!("Failed to capture stderr"))?;
    
    // Create buffers for stdout and stderr
    let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
    let stderr_buffer = Arc::new(Mutex::new(Vec::new()));
    
    // Generate a process ID
    let process_id = service.generate_process_id();
    
    // Create a PowerShell process structure
    let ps_process = PowerShellProcess {
        process_id: process_id.clone(),
        command: command.clone(),
        process: Arc::new(Mutex::new(child)),
        stdout_buffer: stdout_buffer.clone(),
        stderr_buffer: stderr_buffer.clone(),
        start_time: Utc::now(),
        end_time: None,
        is_running: Arc::new(Mutex::new(true)),
        exit_code: Arc::new(Mutex::new(None)),
    };
    
    // Store the process in the running processes map
    service.running_processes.insert(process_id.clone(), ps_process);
    
    // Spawn a task to collect stdout
    let stdout_buffer_clone = stdout_buffer.clone();
    
    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut buffer = [0u8; 4096];
        
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break, // End of stream
                Ok(n) => {
                    let mut stdout_lock = stdout_buffer_clone.lock().await;
                    stdout_lock.extend_from_slice(&buffer[0..n]);
                }
                Err(e) => {
                    log::error!("Error reading stdout: {}", e);
                    break;
                }
            }
        }
    });
    
    // Spawn a task to collect stderr
    let stderr_buffer_clone = stderr_buffer.clone();
    
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut buffer = [0u8; 4096];
        
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break, // End of stream
                Ok(n) => {
                    let mut stderr_lock = stderr_buffer_clone.lock().await;
                    stderr_lock.extend_from_slice(&buffer[0..n]);
                }
                Err(e) => {
                    log::error!("Error reading stderr: {}", e);
                    break;
                }
            }
        }
    });
    
    // Spawn a task to monitor process completion
    let process_id_clone = process_id.clone();
    let running_processes_clone = service.running_processes.clone();
    
    tokio::spawn(async move {
        if let Some(ps_process) = running_processes_clone.get(&process_id_clone) {
            let process_mutex = ps_process.process.clone();
            let is_running_mutex = ps_process.is_running.clone();
            let exit_code_mutex = ps_process.exit_code.clone();
            
            let mut process_lock = process_mutex.lock().await;
            
            match process_lock.wait().await {
                Ok(status) => {
                    // Update the process status
                    let mut is_running = is_running_mutex.lock().await;
                    *is_running = false;
                    
                    let mut exit_code = exit_code_mutex.lock().await;
                    *exit_code = status.code();
                    
                    // Update end time
                    if let Some(mut ps_process) = running_processes_clone.get_mut(&process_id_clone) {
                        ps_process.end_time = Some(Utc::now());
                    }
                    
                    log::info!("Background process completed: {}, exit code: {:?}", 
                               process_id_clone, status.code());
                }
                Err(e) => {
                    log::error!("Error waiting for process to complete: {}", e);
                    
                    // Mark the process as not running
                    let mut is_running = is_running_mutex.lock().await;
                    *is_running = false;
                    
                    // Update end time
                    if let Some(mut ps_process) = running_processes_clone.get_mut(&process_id_clone) {
                        ps_process.end_time = Some(Utc::now());
                    }
                }
            }
        }
    });
    
    Ok(process_id)
}

/// Get the status of a background process
pub async fn get_process_status(service: &PowerShellService, process_id: &str) -> Result<String> {
    // Try to get the process from the running processes map
    if let Some(ps_process) = service.running_processes.get(process_id) {
        let is_running = *ps_process.is_running.lock().await;
        let exit_code = *ps_process.exit_code.lock().await;
        
        let status = ProcessStatus {
            process_id: ps_process.process_id.clone(),
            command: ps_process.command.clone(),
            running: is_running,
            exit_code,
            start_time: ps_process.start_time.to_rfc3339(),
            end_time: ps_process.end_time.map(|t| t.to_rfc3339()),
        };
        
        Ok(serde_json::to_string_pretty(&status)?)
    } else {
        Err(anyhow!("Process not found: {}", process_id))
    }
}

/// Kill a running PowerShell process
pub async fn kill_process(service: &PowerShellService, process_id: &str) -> Result<String> {
    // Try to get the process from the running processes map
    if let Some(ps_process) = service.running_processes.get(process_id) {
        let is_running = *ps_process.is_running.lock().await;
        
        if !is_running {
            return Ok(format!("Process {} is already terminated", process_id));
        }
        
        // Get the process
        let mut process = ps_process.process.lock().await;
        
        // Try to kill the process
        match process.kill().await {
            Ok(_) => {
                // Update the process status
                let mut is_running = ps_process.is_running.lock().await;
                *is_running = false;
                
                // Update end time
                if let Some(mut ps_process) = service.running_processes.get_mut(process_id) {
                    ps_process.end_time = Some(Utc::now());
                }
                
                Ok(format!("Process {} killed successfully", process_id))
            }
            Err(e) => {
                Err(anyhow!("Failed to kill process {}: {}", process_id, e))
            }
        }
    } else {
        Err(anyhow!("Process not found: {}", process_id))
    }
}

/// Get the output of a background process
pub async fn get_process_output(service: &PowerShellService, process_id: &str) -> Result<String> {
    // Try to get the process from the running processes map
    if let Some(ps_process) = service.running_processes.get(process_id) {
        // Get stdout and stderr
        let stdout_buffer = ps_process.stdout_buffer.lock().await;
        let stderr_buffer = ps_process.stderr_buffer.lock().await;
        
        let is_running = *ps_process.is_running.lock().await;
        let exit_code = *ps_process.exit_code.lock().await;
        
        let output = ProcessOutput {
            process_id: ps_process.process_id.clone(),
            stdout: String::from_utf8_lossy(&stdout_buffer).to_string(),
            stderr: String::from_utf8_lossy(&stderr_buffer).to_string(),
            exit_code,
            completed: !is_running,
        };
        
        Ok(serde_json::to_string_pretty(&output)?)
    } else {
        Err(anyhow!("Process not found: {}", process_id))
    }
}

/// List all running background processes
pub async fn list_running_processes(service: &PowerShellService) -> Result<String> {
    let mut process_list = Vec::new();
    
    for item in service.running_processes.iter() {
        let ps_process = item.value();
        let is_running = *ps_process.is_running.lock().await;
        let exit_code = *ps_process.exit_code.lock().await;
        
        let status = ProcessStatus {
            process_id: ps_process.process_id.clone(),
            command: ps_process.command.clone(),
            running: is_running,
            exit_code,
            start_time: ps_process.start_time.to_rfc3339(),
            end_time: ps_process.end_time.map(|t| t.to_rfc3339()),
        };
        
        process_list.push(status);
    }
    
    Ok(serde_json::to_string_pretty(&process_list)?)
}
