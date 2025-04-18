//! Full-featured MCP client example
//!
//! This example demonstrates most of the client API features.

use mcpx::{
    client::{Client, ClientBuilder, ClientEvent, CompletionReferenceType, ResourceContent},
    protocol::{
        logging::LoggingLevel,
        tools::ToolCallContent,
    },
    error::Error,
};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::time::Duration;

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
                println!("✅ Connected to server: {} {}", server_info.name, server_info.version);
                println!("✅ Protocol version: {}", protocol_version);
                println!("✅ Server capabilities:");
                println!("   - Logging: {}", capabilities.logging);
                println!("   - Completions: {}", capabilities.completions);
                println!("   - Prompts: {}", capabilities.prompts);
                println!("   - Resources: {}", capabilities.resources);
                println!("   - Tools: {}", capabilities.tools);

                if let Some(instructions) = instructions {
                    println!("✅ Server instructions: {}", instructions);
                }
            }
            ClientEvent::Disconnected { reason } => {
                println!("❌ Disconnected from server: {}", reason);
                break;
            }
            ClientEvent::ResourcesChanged => {
                println!("📄 Resources changed");
            }
            ClientEvent::PromptsChanged => {
                println!("📝 Prompts changed");
            }
            ClientEvent::ToolsChanged => {
                println!("🔧 Tools changed");
            }
            ClientEvent::RootsChanged => {
                println!("📁 Roots changed");
            }
            ClientEvent::ResourceUpdated { uri } => {
                println!("📄 Resource updated: {}", uri);
            }
            ClientEvent::LogMessage {
                level,
                logger,
                data,
            } => {
                let logger_str = logger.as_deref().unwrap_or("server");
                let level_icon = match level {
                    LoggingLevel::Debug => "🔍",
                    LoggingLevel::Info => "ℹ️",
                    LoggingLevel::Notice => "📢",
                    LoggingLevel::Warning => "⚠️",
                    LoggingLevel::Error => "❌",
                    LoggingLevel::Critical => "🔥",
                    LoggingLevel::Alert => "🚨",
                    LoggingLevel::Emergency => "☢️",
                };
                println!("{} [{}] [{}] {}", level_icon, level_icon, logger_str, data);
            }
            ClientEvent::Progress {
                request_id: _,
                token: _,
                progress,
                total,
                message,
            } => {
                let total_str = total.map_or("?".to_string(), |t| t.to_string());
                let message_str = message.as_deref().unwrap_or("");
                println!(
                    "⏳ Progress: {:.1}% ({}/{}) {}",
                    if let Some(t) = total { progress / t * 100.0 } else { progress },
                    progress,
                    total_str,
                    message_str
                );
            }
            ClientEvent::Error { error } => {
                println!("❌ Error: {}", error);
            }
        }
    }

    println!("Event handler stopped");
}

async fn demo_resources(client: &Client) -> Result<(), Error> {
    println!("\n📄 === RESOURCES DEMO ===");

    // List resources
    println!("📄 Listing resources...");
    match client.list_resources().await {
        Ok(resources) => {
            println!("📄 Found {} resources:", resources.len());
            for resource in resources {
                println!("   - {} ({})", resource.name, resource.uri);

                // Read each resource
                match client.read_resource(&resource.uri).await {
                    Ok(contents) => {
                        for content in contents {
                            match content {
                                ResourceContent::Text(text) => {
                                    println!("     Content: {} ({})", text.text, text.mime_type.unwrap_or_default());
                                }
                                ResourceContent::Blob(blob) => {
                                    println!("     Binary content: {} bytes ({})",
                                        blob.blob.len(),
                                        blob.mime_type.unwrap_or_default()
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("     Failed to read resource: {}", e);
                    }
                }

                // Try to subscribe to resource updates
                match client.subscribe_resource(&resource.uri).await {
                    Ok(_) => {
                        println!("     Subscribed to updates");
                    }
                    Err(e) => {
                        println!("     Failed to subscribe: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to list resources: {}", e);
        }
    }

    Ok(())
}

async fn demo_prompts(client: &Client) -> Result<(), Error> {
    println!("\n📝 === PROMPTS DEMO ===");

    // List prompts
    println!("📝 Listing prompts...");
    match client.list_prompts().await {
        Ok(prompts) => {
            println!("📝 Found {} prompts:", prompts.len());
            for prompt in prompts {
                println!("   - {}", prompt.name);
                if let Some(desc) = &prompt.description {
                    println!("     Description: {}", desc);
                }

                if let Some(args) = &prompt.arguments {
                    println!("     Arguments:");
                    for arg in args {
                        println!("       - {} ({})",
                            arg.name,
                            if arg.required.unwrap_or(false) { "required" } else { "optional" }
                        );
                        if let Some(desc) = &arg.description {
                            println!("         Description: {}", desc);
                        }
                    }

                    // Try to get the prompt with some arguments
                    let mut arguments = HashMap::new();
                    arguments.insert("name".to_string(), "User".to_string());
                    arguments.insert("topic".to_string(), "Model Context Protocol".to_string());

                    match client.get_prompt(&prompt.name, Some(arguments)).await {
                        Ok(messages) => {
                            println!("     Got prompt with {} messages:", messages.len());
                            for message in messages {
                                println!("       - {:?} message", message.role);
                                // We're simplifying here - in a real application you'd handle different content types
                            }
                        }
                        Err(e) => {
                            println!("     Failed to get prompt: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to list prompts: {}", e);
        }
    }

    Ok(())
}

async fn demo_tools(client: &Client) -> Result<(), Error> {
    println!("\n🔧 === TOOLS DEMO ===");

    // List tools
    println!("🔧 Listing tools...");
    match client.list_tools().await {
        Ok(tools) => {
            println!("🔧 Found {} tools:", tools.len());
            for tool in tools {
                println!("   - {}", tool.name);
                if let Some(desc) = &tool.description {
                    println!("     Description: {}", desc);
                }

                let annotations = if let Some(annotations) = &tool.annotations {
                    let mut hints = Vec::new();
                    if let Some(title) = &annotations.title {
                        hints.push(format!("title: {}", title));
                    }
                    if let Some(read_only) = annotations.read_only_hint {
                        hints.push(format!("read-only: {}", read_only));
                    }
                    if let Some(destructive) = annotations.destructive_hint {
                        hints.push(format!("destructive: {}", destructive));
                    }
                    if let Some(idempotent) = annotations.idempotent_hint {
                        hints.push(format!("idempotent: {}", idempotent));
                    }
                    if let Some(open_world) = annotations.open_world_hint {
                        hints.push(format!("open-world: {}", open_world));
                    }

                    if hints.is_empty() {
                        "".to_string()
                    } else {
                        format!(" ({})", hints.join(", "))
                    }
                } else {
                    "".to_string()
                };

                println!("     Annotations:{}", annotations);

                // Try to call the tool
                if tool.name == "search" {
                    let arguments = serde_json::json!({
                        "query": "Model Context Protocol"
                    });

                    match client.call_tool(&tool.name, Some(arguments)).await {
                        Ok(result) => {
                            println!("     Called tool successfully");
                            println!("     Result content ({}):", result.content.len());
                            for content in result.content {
                                match content {
                                    ToolCallContent::Text(text) => {
                                        println!("       Text: {}", text.text);
                                    }
                                    ToolCallContent::Image(_) => {
                                        println!("       Image");
                                    }
                                    ToolCallContent::Audio(_) => {
                                        println!("       Audio");
                                    }
                                    ToolCallContent::Resource(_) => {
                                        println!("       Resource");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("     Failed to call tool: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to list tools: {}", e);
        }
    }

    Ok(())
}

async fn demo_completions(client: &Client) -> Result<(), Error> {
    println!("\n🔍 === COMPLETIONS DEMO ===");

    // Try to get completions for a prompt argument
    println!("🔍 Getting completions for a prompt argument...");

    match client.get_completions(
        CompletionReferenceType::Prompt,
        "example-prompt",
        "topic",
        "mo",
    ).await {
        Ok(result) => {
            println!("🔍 Completion suggestions:");
            for value in result.completion.values {
                println!("   - {}", value);
            }

            if let Some(total) = result.completion.total {
                println!("   Total available: {}", total);
            }

            if let Some(true) = result.completion.has_more {
                println!("   More suggestions available");
            }
        }
        Err(e) => {
            println!("❌ Failed to get completions: {}", e);
        }
    }

    Ok(())
}

async fn demo_logging(client: &Client) -> Result<(), Error> {
    println!("\n📝 === LOGGING DEMO ===");

    // Set logging level
    println!("📝 Setting logging level to debug...");
    match client.set_logging_level(LoggingLevel::Debug).await {
        Ok(_) => {
            println!("✅ Logging level set to debug");
        }
        Err(e) => {
            println!("❌ Failed to set logging level: {}", e);
        }
    }

    // Wait for some log messages
    println!("📝 Waiting for log messages (3 seconds)...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("mcpx=debug")
        .init();

    // Create client
    let (client, event_receiver) = ClientBuilder::new()
        .with_implementation("full-client", "0.1.0")
        .with_roots(true)
        .with_roots_list_changed(true)
        .with_sampling(true)
        .with_websocket_url("ws://localhost:3000")
        .build()?;

    // Start event handler
    let event_handler = tokio::spawn(handle_events(event_receiver));

    // Connect to server
    println!("🔌 Connecting to server...");
    client.connect().await?;

    // Wait a moment for initialization to complete
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Run demo functions sequentially
    demo_resources(&client).await?;
    demo_prompts(&client).await?;
    demo_tools(&client).await?;
    demo_completions(&client).await?;
    demo_logging(&client).await?;

    // All demos completed successfully

    // Ping the server
    println!("\n🏓 Pinging server...");
    match client.ping().await {
        Ok(_) => {
            println!("✅ Server responded to ping");
        }
        Err(e) => {
            println!("❌ Ping failed: {}", e);
        }
    }

    // Disconnect from server
    println!("\n🔌 Disconnecting from server...");
    client.disconnect().await?;
    println!("✅ Disconnected");

    // Wait for event handler to finish
    event_handler.await.unwrap();

    Ok(())
}
