//! WebSocket MCP server example
//!
//! This example demonstrates how to create an MCP server that accepts
//! WebSocket connections.

use mcpx::{
    server::{
        Server, ServerBuilder, ServerEvent, ServerService, ServiceContext,
        ServiceRequest, ServiceResponse,
    },
    protocol::{
        resources::Resource,
        JSONRPCMessage, JSONRPCError,
        json_rpc::JSONRPCErrorInfo,
    },
    error::Error,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::protocol::Message as WsMessage,
};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

// A simple service implementation
struct SimpleService {
    resources: HashMap<String, Resource>,
    resource_contents: HashMap<String, String>,
}

impl SimpleService {
    fn new() -> Self {
        let mut service = Self {
            resources: HashMap::new(),
            resource_contents: HashMap::new(),
        };

        // Add example resources
        service.add_resource(
            "example",
            "Example Resource",
            "A simple example resource",
            "text/plain",
            "This is an example resource provided by the WebSocket server.",
        );

        service
    }

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
}

#[async_trait]
impl ServerService for SimpleService {
    async fn handle_request(
        &self,
        _context: ServiceContext,
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
                if let Some(_content) = self.resource_contents.get(&uri) {
                    // Return the response
                    return Ok(ServiceResponse::ReadResource {
                        contents: vec![],
                    })
                } else {
                    Err(Error::ProtocolError(format!("Resource not found: {}", uri)))
                }
            }

            // For the example, we'll just handle resources
            _ => Err(Error::ProtocolError("Not implemented".to_string())),
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
        Ok(())
    }

    async fn client_disconnected(
        &self,
        client_id: String,
        reason: String,
    ) -> Result<(), Error> {
        println!("Client disconnected: {} ({})", client_id, reason);
        Ok(())
    }
}



// Main function to run the WebSocket server
async fn run_websocket_server(
    server: Arc<Server>,
    addr: &str,
) -> Result<(), Error> {
    // Create a TCP listener
    let listener = TcpListener::bind(addr).await
        .map_err(|e| Error::TransportError(format!("Failed to bind to {}: {}", addr, e)))?;

    println!("WebSocket server listening on {}", addr);

    // Accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);

        // Clone the server for this connection
        let server = server.clone();

        // Spawn a task to handle this connection
        tokio::spawn(async move {
            if let Err(e) = handle_connection(server, stream).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }

    Ok(())
}

// Handle a single WebSocket connection
async fn handle_connection(
    server: Arc<Server>,
    stream: TcpStream,
) -> Result<(), Error> {
    // Accept the WebSocket connection
    let ws_stream = accept_async(stream).await
        .map_err(|e| Error::TransportError(format!("WebSocket handshake failed: {}", e)))?;

    println!("WebSocket connection established");

    // Generate a client ID
    let client_id = Uuid::new_v4().to_string();

    // Add the client to the server
    server.add_connection(&client_id).await?;

    // Split the WebSocket stream
    let (mut sink, mut stream) = ws_stream.split();

    // Handle messages
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(ws_msg) => {
                match ws_msg {
                    WsMessage::Text(text) => {
                        // Parse the JSON-RPC message
                        let json_msg: serde_json::Result<JSONRPCMessage> = serde_json::from_str(&text);

                        match json_msg {
                            Ok(message) => {
                                // Handle the message
                                let response = server.handle_message(&client_id, message).await;

                                // Send response if there is one
                                if let Ok(Some(response)) = response {
                                    let response_text = serde_json::to_string(&response)
                                        .map_err(|e| Error::JsonError(e.to_string()))?;

                                    sink.send(WsMessage::Text(response_text.into())).await
                                        .map_err(|e| Error::TransportError(format!("Failed to send response: {}", e)))?;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse message: {}", e);

                                // Send error response
                                let error = JSONRPCError {
                                    jsonrpc: "2.0".to_string(),
                                    id: 0.into(), // We don't know the ID, so use 0
                                    error: JSONRPCErrorInfo {
                                        code: -32700, // Parse error
                                        message: format!("Parse error: {}", e),
                                        data: None,
                                    },
                                };

                                let error_text = serde_json::to_string(&JSONRPCMessage::Error(error))
                                    .map_err(|e| Error::JsonError(e.to_string()))?;

                                sink.send(WsMessage::Text(error_text.into())).await
                                    .map_err(|e| Error::TransportError(format!("Failed to send error: {}", e)))?;
                            }
                        }
                    }
                    WsMessage::Binary(_) => {
                        // Ignore binary messages for this example
                    }
                    WsMessage::Ping(data) => {
                        // Respond to ping with pong
                        sink.send(WsMessage::Pong(data)).await
                            .map_err(|e| Error::TransportError(format!("Failed to send pong: {}", e)))?;
                    }
                    WsMessage::Pong(_) => {
                        // Ignore pong messages
                    }
                    WsMessage::Close(_) => {
                        // Connection closed
                        println!("WebSocket connection closed");
                        break;
                    },
                    WsMessage::Frame(_) => {
                        // Ignore frame messages for this example
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Remove the client from the server
    server.remove_connection(&client_id).await?;

    println!("WebSocket connection closed");

    Ok(())
}

// Function to handle server events
async fn handle_events(mut receiver: mpsc::Receiver<ServerEvent>) {
    while let Some(event) = receiver.recv().await {
        match event {
            ServerEvent::ClientConnected {
                client_id,
                client_info: _,
                protocol_version: _,
                capabilities: _,
            } => {
                println!("Event: Client connected: {}", client_id);
            }

            ServerEvent::ClientDisconnected { client_id, reason: _ } => {
                println!("Event: Client disconnected: {}", client_id);
            }

            ServerEvent::RootsUpdated { client_id } => {
                println!("Event: Roots updated for client: {}", client_id);
            }

            ServerEvent::Error { client_id, error } => {
                let client_str = client_id.as_deref().unwrap_or("server");
                println!("Event: Error from {}: {}", client_str, error);
            }
        }
    }
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
        .with_implementation("websocket-server", "0.1.0")
        .with_instructions("This is a WebSocket MCP server example")
        .with_resources(true)
        .with_resources_list_changed(true)
        .build(Box::new(service))?;

    // Start event handler
    let event_handler = tokio::spawn(handle_events(event_receiver));

    // Start server
    println!("Starting MCP server...");
    server.start().await?;

    // Create an Arc to share the server
    let server_arc = Arc::new(server);

    // Run the WebSocket server
    let addr = "127.0.0.1:3000";
    println!("Starting WebSocket server on {}...", addr);

    match run_websocket_server(server_arc.clone(), addr).await {
        Ok(()) => println!("WebSocket server stopped"),
        Err(e) => eprintln!("WebSocket server error: {}", e),
    }

    // Stop server
    println!("Stopping MCP server...");
    server_arc.stop().await?;

    // Wait for event handler to finish
    event_handler.abort();

    Ok(())
}
