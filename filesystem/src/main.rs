use anyhow::Result;
use log::{error, info};
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};

mod filesystem;
mod tools;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    info!("Starting Filesystem MCP Server...");

    // Get allowed directories from command line arguments
    let allowed_dirs: Vec<String> = std::env::args()
        .skip(1) // Skip the program name
        .collect();

    if allowed_dirs.is_empty() {
        error!("No allowed directories specified. Please provide at least one directory as a command line argument.");
        std::process::exit(1);
    }

    info!("Allowed directories: {:?}", allowed_dirs);

    // Create the filesystem service
    let service = filesystem::FilesystemService::new(allowed_dirs);

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
