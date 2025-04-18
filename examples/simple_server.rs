//! Simple MCP server example
//!
//! This example demonstrates how to create a simple MCP server and
//! handle client connections.

use mcpx::{
    server::{
        ServerBuilder, ServerEvent, ServerService, ServiceContext,
        ServiceRequest, ServiceResponse,
    },
    protocol::{
        resources::{Resource, TextResourceContents},
        prompts::{Prompt, PromptMessage, PromptArgument, PromptContent},
        tools::{Tool, CallToolResult, ToolAnnotations, ToolInputSchema, ToolCallContent},
        Role,
        sampling::TextContent,
    },
    error::Error,
};
use async_trait::async_trait;
use std::sync::Mutex;
use std::collections::HashMap;
use tokio::sync::mpsc;

// A simple service implementation that provides static resources, prompts, and tools
struct SimpleService {
    // Store resources by URI
    resources: HashMap<String, Resource>,
    // Store resource contents by URI
    resource_contents: HashMap<String, String>,
    // Store prompts by name
    prompts: HashMap<String, Prompt>,
    // Store prompt templates by name
    prompt_templates: HashMap<String, Vec<PromptMessage>>,
    // Store tools by name
    tools: HashMap<String, Tool>,
    // Track connected clients
    connected_clients: Mutex<Vec<String>>,
}

impl SimpleService {
    // Create a new service with some example data
    fn new() -> Self {
        let mut service = Self {
            resources: HashMap::new(),
            resource_contents: HashMap::new(),
            prompts: HashMap::new(),
            prompt_templates: HashMap::new(),
            tools: HashMap::new(),
            connected_clients: Mutex::new(Vec::new()),
        };

        // Add some example resources
        service.add_resource(
            "example-text",
            "Example Text",
            "A simple text resource",
            "text/plain",
            "This is an example text resource provided by the server.",
        );

        service.add_resource(
            "example-json",
            "Example JSON",
            "A simple JSON resource",
            "application/json",
            r#"{"name": "Example", "type": "JSON", "items": [1, 2, 3]}"#,
        );

        // Add a prompt
        service.add_prompt(
            "example-prompt",
            "Example Prompt",
            vec![
                PromptArgument {
                    name: "name".to_string(),
                    description: Some("Your name".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "topic".to_string(),
                    description: Some("A topic to discuss".to_string()),
                    required: Some(false),
                },
            ],
            vec![
                PromptMessage {
                    role: Role::User,
                    content: mcpx::protocol::prompts::PromptContent::Text(TextContent {
                        r#type: "text".to_string(),
                        text: "Hello, my name is {{name}}. Let's talk about {{topic}}".to_string(),
                        annotations: None,
                    }),
                },
            ],
        );

        // Add a tool
        let mut properties = HashMap::new();
        properties.insert(
            "query".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "The search query"
            }),
        );

        let tool = Tool {
            name: "search".to_string(),
            description: Some("Search for information".to_string()),
            input_schema: mcpx::protocol::tools::ToolInputSchema {
                r#type: "object".to_string(),
                properties: Some(properties),
                required: Some(vec!["query".to_string()]),
            },
            annotations: Some(ToolAnnotations {
                title: Some("Search Tool".to_string()),
                read_only_hint: Some(true),
                destructive_hint: None,
                idempotent_hint: None,
                open_world_hint: Some(true),
            }),
        };

        service.tools.insert("search".to_string(), tool);

        service
    }

    // Add a resource
    fn add_resource(
        &mut self,
        uri: &str,
        name: &str,
        description: &str,
        mime_type: &str,
        content: &str,
    ) {
        let resource = Resource {
            uri: format!("resource://{}", uri),
            name: name.to_string(),
            description: Some(description.to_string()),
            mime_type: Some(mime_type.to_string()),
            annotations: None,
            size: Some(content.len() as i64),
        };

        self.resources.insert(resource.uri.clone(), resource);
        self.resource_contents.insert(format!("resource://{}", uri), content.to_string());
    }

    // Add a prompt
    fn add_prompt(
        &mut self,
        name: &str,
        description: &str,
        arguments: Vec<PromptArgument>,
        messages: Vec<PromptMessage>,
    ) {
        let prompt = Prompt {
            name: name.to_string(),
            description: Some(description.to_string()),
            arguments: Some(arguments),
        };

        self.prompts.insert(name.to_string(), prompt);
        self.prompt_templates.insert(name.to_string(), messages);
    }
}



#[async_trait]
impl ServerService for SimpleService {
    async fn handle_request(
        &self,
        context: ServiceContext,
        request: ServiceRequest,
    ) -> Result<ServiceResponse, Error> {
        match request {
            ServiceRequest::ListResources { cursor: _ } => {
                // Return all resources
                let resources = self.resources.values().cloned().collect();
                Ok(ServiceResponse::ListResources {
                    resources,
                    next_cursor: None,
                })
            }

            ServiceRequest::ReadResource { uri } => {
                // Get resource content
                if let Some(content) = self.resource_contents.get(&uri) {
                    let _resource_content = mcpx::protocol::resources::ResourceContent::Text(TextResourceContents {
                        uri: uri.clone(),
                        text: content.clone(),
                        mime_type: self.resources.get(&uri).and_then(|r| r.mime_type.clone()),
                    });

                    Ok(ServiceResponse::ReadResource {
                        contents: vec![],
                    })
                } else {
                    Err(Error::ProtocolError(format!("Resource not found: {}", uri)))
                }
            }

            ServiceRequest::ListPrompts { cursor: _ } => {
                // Return all prompts
                let prompts = self.prompts.values().cloned().collect();
                Ok(ServiceResponse::ListPrompts {
                    prompts,
                    next_cursor: None,
                })
            }

            ServiceRequest::GetPrompt { name, arguments } => {
                // Get prompt template
                if let Some(template) = self.prompt_templates.get(&name) {
                    // Process template with arguments
                    let mut messages = template.clone();

                    // Simple template substitution for {{variable}} patterns
                    if let Some(args) = &arguments {
                        for message in &mut messages {
                            if let mcpx::protocol::prompts::PromptContent::Text(content) = &mut message.content {
                                let mut text = content.text.clone();
                                for (key, value) in args {
                                    text = text.replace(&format!("{{{{{}}}}}", key), value);
                                }
                                content.text = text;
                            }
                        }
                    }

                    let description = self.prompts.get(&name).and_then(|p| p.description.clone());

                    Ok(ServiceResponse::GetPrompt {
                        messages,
                        description,
                    })
                } else {
                    Err(Error::ProtocolError(format!("Prompt not found: {}", name)))
                }
            }

            ServiceRequest::ListTools { cursor: _ } => {
                // Return all tools
                let tools = self.tools.values().cloned().collect();
                Ok(ServiceResponse::ListTools {
                    tools,
                    next_cursor: None,
                })
            }

            ServiceRequest::CallTool { name, arguments } => {
                // Handle tool call
                if name == "search" {
                    // Simple search tool that just echoes the query
                    let query = if let Some(args) = &arguments {
                        args["query"].as_str().unwrap_or("").to_string()
                    } else {
                        "".to_string()
                    };

                    let content = TextContent {
                        r#type: "text".to_string(),
                        text: format!("Search results for: {}", query),
                        annotations: None,
                    };

                    Ok(ServiceResponse::CallTool {
                        result: CallToolResult {
                            content: vec![mcpx::protocol::tools::ToolCallContent::Text(content)],
                            is_error: None,
                            meta: None,
                        },
                    })
                } else {
                    Err(Error::ProtocolError(format!("Tool not found: {}", name)))
                }
            }

            ServiceRequest::SetLoggingLevel { level } => {
                // Just acknowledge the request
                println!("Client {} set logging level to {:?}", context.client_id, level);
                Ok(ServiceResponse::SetLoggingLevel)
            }

            _ => {
                // Other requests not implemented in this example
                Err(Error::ProtocolError(format!("Request not implemented: {:?}", request)))
            }
        }
    }

    async fn client_connected(
        &self,
        client_id: String,
        client_info: mcpx::protocol::Implementation,
        _protocol_version: String,
        _capabilities: mcpx::server::ClientCapabilities,
    ) -> Result<(), Error> {
        println!(
            "Client connected: {} {} {}",
            client_id, client_info.name, client_info.version
        );

        // Track the client
        let mut clients = self.connected_clients.lock().unwrap();
        clients.push(client_id);

        Ok(())
    }

    async fn client_disconnected(
        &self,
        client_id: String,
        reason: String,
    ) -> Result<(), Error> {
        println!("Client disconnected: {} ({})", client_id, reason);

        // Remove the client
        let mut clients = self.connected_clients.lock().unwrap();
        clients.retain(|id| id != &client_id);

        Ok(())
    }
}

// Function to handle server events
async fn handle_events(mut receiver: mpsc::Receiver<ServerEvent>) {
    println!("Starting event handler...");

    while let Some(event) = receiver.recv().await {
        match event {
            ServerEvent::ClientConnected {
                client_id,
                client_info,
                protocol_version,
                capabilities,
            } => {
                println!("Client connected: {} {} {}", client_id, client_info.name, client_info.version);
                println!("Protocol version: {}", protocol_version);
                println!("Client capabilities:");
                println!("  - Roots: {}", capabilities.roots);
                println!("  - Roots list changed: {}", capabilities.roots_list_changed);
                println!("  - Sampling: {}", capabilities.sampling);
            }

            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client disconnected: {} ({})", client_id, reason);
            }

            ServerEvent::RootsUpdated { client_id } => {
                println!("Roots updated for client: {}", client_id);
            }

            ServerEvent::Error { client_id, error } => {
                let client_str = client_id.as_deref().unwrap_or("server");
                println!("Error from {}: {}", client_str, error);
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

    // Create service
    let service = SimpleService::new();

    // Create server
    let (server, event_receiver) = ServerBuilder::new()
        .with_implementation("simple-server", "0.1.0")
        .with_instructions("This is a simple MCP server example")
        .with_logging(true)
        .with_resources(true)
        .with_resources_list_changed(true)
        .with_prompts(true)
        .with_prompts_list_changed(true)
        .with_tools(true)
        .with_tools_list_changed(true)
        .build(Box::new(service))?;

    // Start event handler
    let event_handler = tokio::spawn(handle_events(event_receiver));

    // Start server
    println!("Starting server...");
    server.start().await?;
    println!("Server started!");

    // This is where you would normally integrate with a WebSocket server or other transport
    // For this example, we'll just wait for the user to press Enter to stop the server
    println!("\nPress Enter to stop the server...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop server
    println!("Stopping server...");
    server.stop().await?;
    println!("Server stopped");

    // Wait for event handler to finish
    event_handler.await.unwrap();

    Ok(())
}
