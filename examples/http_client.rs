//! # HTTP Client Example
//!
//! This example demonstrates how to use the HTTP transport to connect to an MCP server.
//! 
//! ## What This Example Does
//!
//! 1. Creates an MCP client using HTTP transport
//! 2. Connects to a server at http://localhost:3000/api/mcp
//! 3. Lists available resources, prompts, and tools
//! 4. Disconnects from the server
//!
//! ## Running This Example
//!
//! ```bash
//! cargo run --example http_client
//! ```
//!
//! Note: You need an MCP server running at http://localhost:3000/api/mcp
//! or you can modify the URL in the code to point to your server.

use mcpx::{
    client::{ClientBuilder, ClientEvent},
    error::Error,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create client with HTTP transport
    println!("📡 Creating MCP client with HTTP transport...");
    let (client, event_receiver) = ClientBuilder::new()
        .with_implementation("http-client-example", "0.1.0")
        .with_roots(true)           // Enable roots capability
        .with_roots_list_changed(true) // Enable roots list changed notifications
        .with_sampling(true)        // Enable sampling capability
        .with_http_url("http://localhost:3000/api/mcp")
        .build()?;
    
    // Start event handler
    tokio::spawn(handle_events(event_receiver));
    
    // Connect to server
    println!("🔌 Connecting to MCP server via HTTP...");
    println!("   URL: http://localhost:3000/api/mcp");
    
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
    
    // Disconnect
    println!("\n🔌 Disconnecting from server...");
    client.disconnect().await?;
    println!("✅ Disconnected successfully!");
    
    // Wait a moment for event handler to process final events
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(())
}

// Handle events from the server
async fn handle_events(mut receiver: mpsc::Receiver<ClientEvent>) {
    while let Some(event) = receiver.recv().await {
        match event {
            ClientEvent::Connected { server_info, .. } => {
                println!("Connected to {} {}", server_info.name, server_info.version);
            },
            ClientEvent::Disconnected { reason } => {
                println!("Disconnected: {}", reason);
            },
            ClientEvent::ResourcesChanged => {
                println!("Resources changed on server");
            },
            ClientEvent::Progress {
                request_id,
                token: _,
                progress,
                total,
                message,
            } => {
                if let Some(total) = total {
                    println!("Progress for request {:?}: {}/{} - {:?}", request_id, progress, total, message);
                } else {
                    println!("Progress for request {:?}: {} - {:?}", request_id, progress, message);
                }
            },
            _ => {},
        }
    }
}
