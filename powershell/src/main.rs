use anyhow::Result;
use log::info;
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};

mod powershell;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    info!("Starting PowerShell MCP Server...");

    // Parse command line arguments for security options if needed
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    // Initialize the PowerShell service
    let service = powershell::PowerShellService::new(&args);

    // Use stdin/stdout as the transport mechanism
    let transport = (stdin(), stdout());

    // Initialize the server
    info!("Initializing MCP server...");
    let server = service.serve(transport).await?;
    
    // Wait for server to shutdown
    let quit_reason = server.waiting().await?;
    info!("Server shutdown: {:?}", quit_reason);

    Ok(())
}
