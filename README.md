# 🚀 MCPX: Model Context Protocol SDK for Rust

Hey there! Welcome to MCPX, your friendly neighborhood Rust implementation of the Model Context Protocol (MCP). We've built this library to be super robust, blazing fast, and delightfully easy to use whether you're building client apps or server systems.

## 🌟 Overview

The Model Context Protocol (MCP) is the cool new standard for communication between Large Language Models (LLMs) and the applications that feed them context data. Think of it as the universal translator that helps your AI models and your apps have meaningful conversations.

This library gives you everything you need to implement the MCP specification (as of 2025-03-26), with full support for all the message types, capabilities, and flows that make the protocol tick. Whether you're building the next revolutionary AI assistant or creating tools that enhance existing models, MCPX has got your back.

## ✨ Features

- 🔄 **Complete MCP Implementation**: Full support for the latest MCP specification (2025-03-26)
- ⚡ **Async All The Way**: Built on Tokio for maximum performance and scalability
- 🛡️ **Type-Safe API**: Rust's type system keeps your code safe and predictable
- 🧩 **Dual-Role Support**: Easily implement clients, servers, or both
- 🔌 **Smart Connection Management**: Handles reconnections and state management
- 🚦 **Comprehensive Error Handling**: Detailed error types for easy debugging
- 🧰 **Extensible Architecture**: Design your own extensions or customize existing ones
- 🌐 **Multiple Transports**: Ready-to-use WebSocket and HTTP implementations included

## 📦 Installation

Getting started with MCPX is super easy! Just add it to your `Cargo.toml`:

```toml
[dependencies]
mcpx = "0.1.0"
tokio = { version = "1.0", features = ["full"] } # You'll need Tokio for async goodness
```

If you're planning to use the WebSocket transport (which most folks do), you might also want:

```toml
tokio-tungstenite = "0.26.0"
futures = "0.3"
```

## 🚀 Quick Start

Let's dive right in with some examples to get you up and running!

### 🧑‍💻 Client Example

Here's how to create a simple MCP client that connects to a server, lists available resources, and gracefully disconnects:

```rust
use mcpx::{
    client::{Client, ClientBuilder, ClientEvent},
    protocol::logging::LoggingLevel,
    error::Error,
};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a shiny new client
    let (client, event_receiver) = ClientBuilder::new()
        .with_implementation("my-awesome-client", "0.1.0")
        .with_roots(true)         // Enable roots capability
        .with_sampling(true)       // Enable sampling capability
        .with_websocket_url("ws://localhost:3000") // Connect via WebSocket
        .build()?;

    // Set up event handling (important for real applications!)
    tokio::spawn(async move {
        handle_events(event_receiver).await;
    });

    // Connect to the server
    println!("Connecting to MCP server...");
    client.connect().await?;
    println!("Connected successfully! 🎉");

    // List all available resources
    let resources = client.list_resources().await?;
    println!("Found {} resources:", resources.len());
    for resource in resources {
        println!("📄 {}: {}", resource.name, resource.uri);

        // You could also read the resource content
        // let content = client.read_resource(&resource.uri).await?;
        // ...
    }

    // You can also work with prompts
    if let Ok(prompts) = client.list_prompts().await {
        println!("\nAvailable prompts:");
        for prompt in prompts {
            println!("📝 {}", prompt.name);
        }
    }

    // And tools
    if let Ok(tools) = client.list_tools().await {
        println!("\nAvailable tools:");
        for tool in tools {
            println!("🔧 {}", tool.name);
        }
    }

    // When you're done, disconnect gracefully
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Disconnected! 👋");

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
            // Handle other events...
            _ => {},
        }
    }
}
```

### 🖥️ Server Example

On the flip side, here's how to create an MCP server that provides resources, prompts, and tools to clients:

```rust
use mcpx::{
    server::{Server, ServerBuilder, ServerEvent, ServerService, ServiceContext, ServiceRequest, ServiceResponse},
    protocol::{
        resources::{Resource, TextResourceContents},
        prompts::{Prompt, PromptMessage, PromptArgument},
        tools::{Tool, CallToolResult, ToolCallContent},
        Role,
    },
    error::Error,
};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::mpsc;

// Our custom service implementation
struct MyService {
    resources: HashMap<String, Resource>,
    resource_contents: HashMap<String, String>,
}

impl MyService {
    fn new() -> Self {
        let mut service = Self {
            resources: HashMap::new(),
            resource_contents: HashMap::new(),
        };

        // Add a sample resource
        service.add_resource(
            "example",
            "Example Resource",
            "A simple example resource",
            "text/plain",
            "Hello from the MCP server! This is an example resource.",
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
impl ServerService for MyService {
    async fn handle_request(
        &self,
        _context: ServiceContext,
        request: ServiceRequest,
    ) -> Result<ServiceResponse, Error> {
        match request {
            // Handle resource listing
            ServiceRequest::ListResources { cursor: _ } => {
                let resources = self.resources.values().cloned().collect();
                Ok(ServiceResponse::ListResources {
                    resources,
                    next_cursor: None,
                })
            },

            // Handle resource reading
            ServiceRequest::ReadResource { uri } => {
                if let Some(content) = self.resource_contents.get(&uri) {
                    // Create a TextResourceContents
                    let text_content = TextResourceContents {
                        uri: uri.clone(),
                        text: content.clone(),
                        mime_type: self.resources.get(&uri).and_then(|r| r.mime_type.clone()),
                    };

                    Ok(ServiceResponse::ReadResource {
                        contents: vec![mcpx::protocol::resources::ResourceContent::Text(text_content)],
                    })
                } else {
                    Err(Error::ProtocolError(format!("Resource not found: {}", uri)))
                }
            },

            // Handle other request types...
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
        println!("Client connected: {} {} {}", client_id, client_info.name, client_info.version);
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

// Function to handle server events
async fn handle_events(mut receiver: mpsc::Receiver<ServerEvent>) {
    while let Some(event) = receiver.recv().await {
        match event {
            ServerEvent::ClientConnected { client_id, client_info, .. } => {
                println!("Event: Client connected: {} {}", client_id, client_info.name);
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Event: Client disconnected: {} ({})", client_id, reason);
            },
            // Handle other events...
            _ => {},
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create our service
    let service = MyService::new();

    // Create the server
    let (server, event_receiver) = ServerBuilder::new()
        .with_implementation("my-awesome-server", "0.1.0")
        .with_instructions("This is an example MCP server")
        .with_resources(true)              // Enable resources capability
        .with_resources_list_changed(true) // Enable resource change notifications
        .with_prompts(true)                // Enable prompts capability
        .with_tools(true)                  // Enable tools capability
        .build(Box::new(service))?;

    // Start event handler
    tokio::spawn(handle_events(event_receiver));

    // Start the server
    println!("Starting MCP server...");
    server.start().await?;
    println!("Server started! 🎉");

    // In a real application, you would integrate with a WebSocket server here
    // For example, using tokio-tungstenite to accept WebSocket connections
    // and forward them to the MCP server.

    // For this example, we'll just wait for user input to stop
    println!("\nPress Enter to stop the server...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Stop the server
    println!("Stopping server...");
    server.stop().await?;
    println!("Server stopped! 👋");

    Ok(())
}
```

These examples should give you a good starting point, but there's so much more you can do with MCPX! Check out the examples directory for more complete implementations.

## 🧠 Protocol Support

MCPX implements all the cool features of the MCP protocol:

- 📚 **Resources**: Everything you need to work with resources
  - List all available resources
  - Read resource contents (text and binary)
  - Work with resource templates
  - Subscribe to resource updates
  - Get notified when resources change

- 💬 **Prompts**: Powerful prompt management
  - List available prompts
  - Get prompt templates with arguments
  - Process templates with variable substitution
  - Handle different prompt formats

- 🛠️ **Tools**: Extend your AI's capabilities
  - List available tools
  - Call tools with arguments
  - Process tool results
  - Handle tool errors gracefully

- 📝 **Logging**: Keep track of what's happening
  - Set logging levels
  - Send and receive log messages
  - Filter logs by severity

- 🔍 **Completion**: Smart suggestions
  - Get completion suggestions for arguments
  - Support for different completion types
  - Handle partial inputs

- 📁 **Roots**: Organize your resources
  - List available roots
  - Get notified when roots change
  - Navigate resource hierarchies

- 🎲 **Sampling**: Get AI responses
  - Request completions from LLMs
  - Stream responses in real-time
  - Support for different content types

## 🌐 Transport Options

MCPX comes with multiple ready-to-use transport implementations that make network communication a breeze:

### WebSocket Transport

For real-time, bidirectional communication (recommended for most use cases):

```rust
// Client-side WebSocket setup
let client = ClientBuilder::new()
    .with_websocket_url("ws://localhost:3000")
    .build()?;
```

### HTTP Transport

For simpler REST-style communication or when WebSockets aren't available:

```rust
// Client-side HTTP setup
let client = ClientBuilder::new()
    .with_http_url("http://localhost:3000/api/mcp")
    .build()?;
```

### Custom Transport

Want to use a different transport mechanism? No problem! You can implement your own custom transport by implementing the `Transport` trait:

```rust
#[async_trait]
impl Transport for MyCustomTransport {
    async fn connect(&mut self) -> Result<(), Error> {
        // Your connection logic here
    }

    async fn disconnect(&mut self) -> Result<(), Error> {
        // Your disconnection logic here
    }

    async fn send(&mut self, message: JSONRPCMessage) -> Result<(), Error> {
        // Your message sending logic here
    }

    async fn receive(&mut self) -> Option<Result<JSONRPCMessage, Error>> {
        // Your message receiving logic here
    }

    async fn is_connected(&mut self) -> bool {
        // Your connection status logic here
    }
}
```

## 📚 Documentation

Want to dive deeper? Generate the full API documentation with:

```bash
cargo doc --open
```

This will build the docs and open them in your browser. The documentation includes detailed explanations of all types, methods, and examples of how to use them.

## 🧪 Examples

The repository includes several examples to help you get started:

- 🔄 **simple_client.rs**: A basic MCP client that connects to a server and lists resources
- 🖥️ **simple_server.rs**: A basic MCP server that provides resources, prompts, and tools
- 🌐 **websocket_server.rs**: A complete WebSocket MCP server implementation
- 🚀 **full_client.rs**: A comprehensive client that demonstrates all client capabilities

To run an example:

```bash
cargo run --example simple_client
```

## 🤝 Contributing

Contributions are always welcome! Whether it's bug reports, feature requests, or code contributions, we appreciate all help in making MCPX better.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📋 Roadmap

Here's what we're planning for future releases:

- Additional transport implementations (HTTP, gRPC)
- More comprehensive examples
- Performance optimizations
- Extended testing suite
- Integration with popular AI frameworks

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details. Feel free to use it in your projects, commercial or otherwise!

## 🙏 Acknowledgements

- The MCP specification team for creating such a useful protocol
- The Rust community for the amazing ecosystem
- All contributors who have helped make this library better (right now, that's just me! 😁)

---

Happy coding! If you build something cool with MCPX, we'd love to hear about it! 🎉
