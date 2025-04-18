//! # Simple MCP Client Example
//!
//! This example demonstrates how to create a simple MCP client and connect to a server.
//! 
//! ## What This Example Does
//!
//! 1. Creates an MCP client using WebSocket transport
//! 2. Connects to a server at ws://localhost:3000
//! 3. Lists available resources, prompts, and tools
//! 4. Sets the logging level to debug
//! 5. Disconnects from the server
//!
//! ## Running This Example
//!
//! ```bash
//! cargo run --example simple_client
//! ```
//!
//! Note: You need an MCP server running at ws://localhost:3000
//! or you can modify the URL in the code to point to your server.

use mcpx::{
    client::{ClientBuilder, ClientEvent},
    protocol::logging::LoggingLevel,
    error::Error,
};
use tokio::sync::mpsc;

// Function to handle client events
async fn handle_events(mut receiver: mpsc::Receiver<ClientEvent>) {
    println!("Starting event handler...");
    
    while let Some(event) = receiver.recv().await {
        match event {
            ClientEvent::Connected {
                server_info,
                protocol_version,
                capabilities,
                instructions,
            } => {
                println!("Connected to server: {} {}", server_info.name, server_info.version);
                println!("Protocol version: {}", protocol_version);
                println!("Server capabilities:");
                println!("  - Logging: {}", capabilities.logging);
                println!("  - Completions: {}", capabilities.completions);
                println!("  - Prompts: {}", capabilities.prompts);
                println!("  - Resources: {}", capabilities.resources);
                println!("  - Tools: {}", capabilities.tools);
                
                if let Some(instructions) = instructions {
                    println!("Server instructions: {}", instructions);
                }
            }
            ClientEvent::Disconnected { reason } => {
                println!("Disconnected from server: {}", reason);
                // Exit the event loop
                break;
            }
            ClientEvent::ResourcesChanged => {
                println!("Resources changed");
            }
            ClientEvent::PromptsChanged => {
                println!("Prompts changed");
            }
            ClientEvent::ToolsChanged => {
                println!("Tools changed");
            }
            ClientEvent::RootsChanged => {
                println!("Roots changed");
            }
            ClientEvent::ResourceUpdated { uri } => {
                println!("Resource updated: {}", uri);
            }
            ClientEvent::LogMessage {
                level,
                logger,
                data,
            } => {
                let logger_str = logger.as_deref().unwrap_or("unknown");
                println!("[{:?}] [{}] {}", level, logger_str, data);
            }
            ClientEvent::Progress {
                request_id,
                token: _,
                progress,
                total,
                message,
            } => {
                let total_str = total.map_or("?".to_string(), |t| t.to_string());
                let message_str = message.as_deref().unwrap_or("");
                println!(
                    "Progress for request {:?}: {}/{} {}",
                    request_id, progress, total_str, message_str
                );
            }
            ClientEvent::Error { error } => {
                println!("Error: {}", error);
            }
        }
    }
    
    println!("Event handler stopped");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("mcpx=debug")
        .init();
    
    // Create client with WebSocket transport
    println!("📡 Creating MCP client with WebSocket transport...");
    let (client, event_receiver) = ClientBuilder::new()
        .with_implementation("simple-client", "0.1.0")
        .with_roots(true)           // Enable roots capability
        .with_roots_list_changed(true) // Enable roots list changed notifications
        .with_sampling(true)        // Enable sampling capability
        .with_websocket_url("ws://localhost:3000")
        .build()?;
    
    // Start event handler
    let event_handler = tokio::spawn(handle_events(event_receiver));
    
    // Connect to server
    println!("🔌 Connecting to MCP server via WebSocket...");
    println!("   URL: ws://localhost:3000");
    
    // Note: This will fail if no server is running at the specified URL
    // In a real application, you would handle this error gracefully
    match client.connect().await {
        Ok(_) => println!("✅ Connected successfully!"),
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("   Make sure an MCP server is running at the specified URL.");
            return Ok(());
        }
    }
    
    // Wait for a moment
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // List resources
    println!("\n📚 Listing resources...");
    match client.list_resources().await {
        Ok(resources) => {
            if resources.is_empty() {
                println!("   No resources found.");
            } else {
                println!("   Found {} resources:", resources.len());
                for (i, resource) in resources.iter().enumerate() {
                    println!("   {}. {} ({})", i+1, resource.name, resource.uri);
                    if let Some(desc) = &resource.description {
                        println!("      Description: {}", desc);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to list resources: {}", e);
        }
    }
    
    // List prompts
    println!("\n🔍 Listing prompts...");
    match client.list_prompts().await {
        Ok(prompts) => {
            if prompts.is_empty() {
                println!("   No prompts found.");
            } else {
                println!("   Found {} prompts:", prompts.len());
                for (i, prompt) in prompts.iter().enumerate() {
                    println!("   {}. {}", i+1, prompt.name);
                    if let Some(desc) = &prompt.description {
                        println!("      Description: {}", desc);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to list prompts: {}", e);
        }
    }
    
    // List tools
    println!("\n🛠️ Listing tools...");
    match client.list_tools().await {
        Ok(tools) => {
            if tools.is_empty() {
                println!("   No tools found.");
            } else {
                println!("   Found {} tools:", tools.len());
                for (i, tool) in tools.iter().enumerate() {
                    println!("   {}. {}", i+1, tool.name);
                    if let Some(desc) = &tool.description {
                        println!("      Description: {}", desc);
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to list tools: {}", e);
        }
    }
    
    // Set logging level
    println!("\n📝 Setting logging level to debug...");
    match client.set_logging_level(LoggingLevel::Debug).await {
        Ok(_) => {
            println!("   ✅ Logging level set to debug");
        }
        Err(e) => {
            println!("   ❌ Failed to set logging level: {}", e);
        }
    }
    
    // Wait for a moment
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Disconnect from server
    println!("\n🔌 Disconnecting from server...");
    client.disconnect().await?;
    println!("✅ Disconnected successfully!");
    
    // Wait for event handler to finish
    event_handler.await.unwrap();
    
    Ok(())
}
