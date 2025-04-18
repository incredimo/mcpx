//! MCP client implementation
//!
//! This module provides a client implementation for the Model Context Protocol (MCP).
//! It handles connection management, protocol encoding/decoding, and provides
//! a convenient API for interacting with MCP servers.

mod builder;
mod handler;
mod state;

pub use builder::ClientBuilder;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use async_trait::async_trait;
use dashmap::DashMap;
use log::{debug, error};
use uuid::Uuid;

use crate::error::Error;
use crate::protocol::{
    Implementation, RequestId, ProgressToken,
    JSONRPCMessage, JSONRPCRequest, JSONRPCNotification,
    resources::{Resource, ResourceTemplate, TextResourceContents, BlobResourceContents},
    prompts::{Prompt, PromptMessage},
    tools::{Tool, CallToolResult},
    roots::{Root},
    completion::CompleteResult,
    logging::LoggingLevel,
};
use crate::transport::Transport;

use self::state::{ClientState, PendingRequest};
use self::handler::ClientMessageHandler;

/// Client capability flags
#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    /// Whether the client supports listing roots
    pub roots: bool,
    /// Whether the client supports notifications for changes to the roots list
    pub roots_list_changed: bool,
    /// Whether the client supports sampling from an LLM
    pub sampling: bool,
    /// Experimental capabilities
    pub experimental: Vec<String>,
}

/// MCP client options
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// Client implementation info
    pub implementation: Implementation,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Automatically acknowledge roots list changed notifications
    pub auto_acknowledge_roots_changed: bool,
    /// Default timeout for requests in milliseconds (0 = no timeout)
    pub default_timeout_ms: u64,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            implementation: Implementation::new("mcpx-client", env!("CARGO_PKG_VERSION")),
            capabilities: ClientCapabilities::default(),
            auto_acknowledge_roots_changed: true,
            default_timeout_ms: 30000, // 30 seconds
        }
    }
}

/// An event that can be emitted by the client
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// Connected to server
    Connected {
        /// Server implementation info
        server_info: Implementation,
        /// Server protocol version
        protocol_version: String,
        /// Server capabilities
        capabilities: ServerCapabilities,
        /// Server instructions (if any)
        instructions: Option<String>,
    },
    /// Disconnected from server
    Disconnected {
        /// Reason for disconnection
        reason: String,
    },
    /// Resources list changed
    ResourcesChanged,
    /// Prompts list changed
    PromptsChanged,
    /// Tools list changed
    ToolsChanged,
    /// Roots list changed
    RootsChanged,
    /// Resource updated
    ResourceUpdated {
        /// URI of the updated resource
        uri: String,
    },
    /// Log message received
    LogMessage {
        /// Severity level
        level: LoggingLevel,
        /// Logger name (if any)
        logger: Option<String>,
        /// Log data
        data: serde_json::Value,
    },
    /// Progress update for a request
    Progress {
        /// Request ID
        request_id: RequestId,
        /// Progress token
        token: ProgressToken,
        /// Current progress
        progress: f64,
        /// Total progress (if known)
        total: Option<f64>,
        /// Progress message (if any)
        message: Option<String>,
    },
    /// Error occurred
    Error {
        /// Error details
        error: Error,
    },
}

/// Server capabilities
#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    /// Whether the server supports sending log messages
    pub logging: bool,
    /// Whether the server supports completions
    pub completions: bool,
    /// Whether the server supports prompts
    pub prompts: bool,
    /// Whether the server supports prompts list changed notifications
    pub prompts_list_changed: bool,
    /// Whether the server supports resources
    pub resources: bool,
    /// Whether the server supports resource list changed notifications
    pub resources_list_changed: bool,
    /// Whether the server supports resource subscriptions
    pub resources_subscribe: bool,
    /// Whether the server supports tools
    pub tools: bool,
    /// Whether the server supports tool list changed notifications
    pub tools_list_changed: bool,
    /// Experimental capabilities
    pub experimental: Vec<String>,
}

/// MCP client session
pub struct Client {
    /// Unique client identifier
    id: String,
    /// Client state
    state: Arc<RwLock<ClientState>>,
    /// Pending requests
    pending_requests: Arc<DashMap<RequestId, PendingRequest>>,
    /// Event sender
    event_sender: mpsc::Sender<ClientEvent>,
    /// Transport layer
    transport: Arc<Box<dyn Transport + Send + Sync>>,
    /// Client options
    options: ClientOptions,
    /// Server capabilities (available after initialization)
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Message handler
    handler: Arc<ClientMessageHandler>,
}

/// Client event listener
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Called when a client event occurs
    async fn on_event(&self, event: ClientEvent);
}

impl Client {
    /// Create a new client with the given transport and options
    pub fn new(
        transport: Box<dyn Transport + Send + Sync>,
        options: ClientOptions,
    ) -> (Self, mpsc::Receiver<ClientEvent>) {
        let id = Uuid::new_v4().to_string();
        let (event_sender, event_receiver) = mpsc::channel(100);

        let state = Arc::new(RwLock::new(ClientState::new()));
        let pending_requests = Arc::new(DashMap::new());
        let server_capabilities = Arc::new(RwLock::new(None));

        let handler = Arc::new(ClientMessageHandler::new(
            state.clone(),
            pending_requests.clone(),
            event_sender.clone(),
            server_capabilities.clone(),
            options.clone(),
        ));

        let client = Self {
            id,
            state,
            pending_requests,
            event_sender,
            transport: Arc::new(transport),
            options,
            server_capabilities,
            handler,
        };

        (client, event_receiver)
    }

    /// Get the client ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Connect to the server
    pub async fn connect(&self) -> Result<(), Error> {
        // Connect transport
        self.transport.connect().await?;

        // Start message processing loop
        let transport = self.transport.clone();
        let handler = self.handler.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            debug!("Starting message processing loop");
            while let Some(msg) = transport.receive().await {
                match msg {
                    Ok(message) => {
                        if let Err(e) = handler.handle_message(message).await {
                            error!("Error handling message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        break;
                    }
                }
            }

            debug!("Message processing loop ended");
            let mut state = state.write().await;
            state.set_disconnected();
        });

        // Initialize the connection
        self.initialize().await?;

        Ok(())
    }

    /// Disconnect from the server
    pub async fn disconnect(&self) -> Result<(), Error> {
        self.transport.disconnect().await?;
        Ok(())
    }

    /// Initialize the connection to the server
    async fn initialize(&self) -> Result<(), Error> {
        let mut state = self.state.write().await;
        if state.is_initializing() || state.is_initialized() {
            return Ok(());
        }

        state.set_initializing();
        drop(state);

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: "init".into(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": crate::protocol::LATEST_PROTOCOL_VERSION,
                "clientInfo": self.options.implementation,
                "capabilities": {
                    "sampling": {},
                    "roots": {
                        "listChanged": self.options.capabilities.roots_list_changed,
                    }
                }
            })),
        };

        let response = self.send_request(request).await?;

        let result = match response {
            JSONRPCMessage::Response(resp) => resp.result,
            JSONRPCMessage::Error(err) => {
                return Err(Error::ServerError(
                    err.error.code,
                    err.error.message,
                    err.error.data,
                ));
            }
            _ => return Err(Error::ProtocolError("Unexpected response type".to_string())),
        };

        let server_info: Implementation = serde_json::from_value(
            result["serverInfo"].clone(),
        )
        .map_err(|e| Error::ParseError(e.to_string()))?;

        let protocol_version = result["protocolVersion"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing protocolVersion".to_string()))?
            .to_string();

        let instructions = result
            .get("instructions")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse server capabilities
        let capabilities = self.parse_server_capabilities(&result["capabilities"]);

        // Store server capabilities
        {
            let mut server_caps = self.server_capabilities.write().await;
            *server_caps = Some(capabilities.clone());
        }

        // Update client state
        {
            let mut state = self.state.write().await;
            state.set_initialized();
        }

        // Send initialized notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
            params: None,
        };

        self.send_notification(notification).await?;

        // Emit connected event
        self.event_sender
            .send(ClientEvent::Connected {
                server_info,
                protocol_version,
                capabilities,
                instructions,
            })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Parse server capabilities from JSON
    fn parse_server_capabilities(&self, json: &serde_json::Value) -> ServerCapabilities {
        let mut capabilities = ServerCapabilities::default();

        // Check for logging capability
        if json.get("logging").is_some() {
            capabilities.logging = true;
        }

        // Check for completions capability
        if json.get("completions").is_some() {
            capabilities.completions = true;
        }

        // Check for prompts capabilities
        if let Some(prompts) = json.get("prompts") {
            capabilities.prompts = true;

            if let Some(list_changed) = prompts.get("listChanged") {
                capabilities.prompts_list_changed = list_changed.as_bool().unwrap_or(false);
            }
        }

        // Check for resources capabilities
        if let Some(resources) = json.get("resources") {
            capabilities.resources = true;

            if let Some(list_changed) = resources.get("listChanged") {
                capabilities.resources_list_changed = list_changed.as_bool().unwrap_or(false);
            }

            if let Some(subscribe) = resources.get("subscribe") {
                capabilities.resources_subscribe = subscribe.as_bool().unwrap_or(false);
            }
        }

        // Check for tools capabilities
        if let Some(tools) = json.get("tools") {
            capabilities.tools = true;

            if let Some(list_changed) = tools.get("listChanged") {
                capabilities.tools_list_changed = list_changed.as_bool().unwrap_or(false);
            }
        }

        // Check for experimental capabilities
        if let Some(experimental) = json.get("experimental") {
            if let Some(obj) = experimental.as_object() {
                capabilities.experimental = obj.keys().map(|k| k.clone()).collect();
            }
        }

        capabilities
    }

    /// Send a request to the server and wait for a response
    pub async fn send_request(&self, request: JSONRPCRequest) -> Result<JSONRPCMessage, Error> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        // Store the pending request
        let pending = PendingRequest {
            sender,
            method: request.method.clone(),
            start_time: std::time::Instant::now(),
        };

        let request_id = request.id.clone();
        self.pending_requests.insert(request_id.clone(), pending);

        // Send the request
        let message = JSONRPCMessage::Request(request);
        self.transport.send(message).await?;

        // Wait for response with optional timeout
        let timeout_ms = self.options.default_timeout_ms;
        let response = if timeout_ms > 0 {
            match tokio::time::timeout(
                std::time::Duration::from_millis(timeout_ms),
                receiver,
            )
            .await
            {
                Ok(r) => r.map_err(|_| Error::InternalError("Response channel closed".to_string()))?,
                Err(_) => {
                    self.pending_requests.remove(&request_id);
                    return Err(Error::Timeout(format!(
                        "Request timed out after {} ms",
                        timeout_ms
                    )));
                }
            }
        } else {
            receiver.await.map_err(|_| Error::InternalError("Response channel closed".to_string()))?
        };

        Ok(response)
    }

    /// Send a notification to the server
    pub async fn send_notification(&self, notification: JSONRPCNotification) -> Result<(), Error> {
        let message = JSONRPCMessage::Notification(notification);
        self.transport.send(message).await
    }

    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>, Error> {
        // Check if server supports resources
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources {
                return Err(Error::UnsupportedFeature("Resources".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let resources = resp.result["resources"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing resources array".to_string()))?;

                let mut result = Vec::with_capacity(resources.len());
                for resource in resources {
                    let res: Resource = serde_json::from_value(resource.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(res);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// List available resource templates
    pub async fn list_resource_templates(&self) -> Result<Vec<ResourceTemplate>, Error> {
        // Check if server supports resources
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources {
                return Err(Error::UnsupportedFeature("Resources".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "resources/templates/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let templates = resp.result["resourceTemplates"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing resourceTemplates array".to_string()))?;

                let mut result = Vec::with_capacity(templates.len());
                for template in templates {
                    let tmpl: ResourceTemplate = serde_json::from_value(template.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(tmpl);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> Result<Vec<ResourceContent>, Error> {
        // Check if server supports resources
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources {
                return Err(Error::UnsupportedFeature("Resources".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "resources/read".to_string(),
            params: Some(serde_json::json!({ "uri": uri })),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let contents = resp.result["contents"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing contents array".to_string()))?;

                let mut result = Vec::with_capacity(contents.len());
                for content in contents {
                    if content.get("text").is_some() {
                        let text: TextResourceContents = serde_json::from_value(content.clone())
                            .map_err(|e| Error::ParseError(e.to_string()))?;
                        result.push(ResourceContent::Text(text));
                    } else if content.get("blob").is_some() {
                        let blob: BlobResourceContents = serde_json::from_value(content.clone())
                            .map_err(|e| Error::ParseError(e.to_string()))?;
                        result.push(ResourceContent::Blob(blob));
                    } else {
                        return Err(Error::ParseError("Unknown resource content type".to_string()));
                    }
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Subscribe to resource updates
    pub async fn subscribe_resource(&self, uri: &str) -> Result<(), Error> {
        // Check if server supports resource subscriptions
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources || !caps.resources_subscribe {
                return Err(Error::UnsupportedFeature("Resource subscriptions".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "resources/subscribe".to_string(),
            params: Some(serde_json::json!({ "uri": uri })),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(_) => Ok(()),
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Unsubscribe from resource updates
    pub async fn unsubscribe_resource(&self, uri: &str) -> Result<(), Error> {
        // Check if server supports resource subscriptions
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources || !caps.resources_subscribe {
                return Err(Error::UnsupportedFeature("Resource subscriptions".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "resources/unsubscribe".to_string(),
            params: Some(serde_json::json!({ "uri": uri })),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(_) => Ok(()),
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// List available prompts
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, Error> {
        // Check if server supports prompts
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.prompts {
                return Err(Error::UnsupportedFeature("Prompts".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "prompts/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let prompts = resp.result["prompts"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing prompts array".to_string()))?;

                let mut result = Vec::with_capacity(prompts.len());
                for prompt in prompts {
                    let p: Prompt = serde_json::from_value(prompt.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(p);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Get a prompt
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<std::collections::HashMap<String, String>>,
    ) -> Result<Vec<PromptMessage>, Error> {
        // Check if server supports prompts
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.prompts {
                return Err(Error::UnsupportedFeature("Prompts".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let mut params = serde_json::json!({ "name": name });
        if let Some(args) = arguments {
            params["arguments"] = serde_json::to_value(args)
                .map_err(|e| Error::InternalError(e.to_string()))?;
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "prompts/get".to_string(),
            params: Some(params),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let messages = resp.result["messages"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing messages array".to_string()))?;

                let mut result = Vec::with_capacity(messages.len());
                for message in messages {
                    let msg: PromptMessage = serde_json::from_value(message.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(msg);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Error> {
        // Check if server supports tools
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.tools {
                return Err(Error::UnsupportedFeature("Tools".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let tools = resp.result["tools"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing tools array".to_string()))?;

                let mut result = Vec::with_capacity(tools.len());
                for tool in tools {
                    let t: Tool = serde_json::from_value(tool.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(t);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<CallToolResult, Error> {
        // Check if server supports tools
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.tools {
                return Err(Error::UnsupportedFeature("Tools".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let mut params = serde_json::json!({ "name": name });
        if let Some(args) = arguments {
            params["arguments"] = args;
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "tools/call".to_string(),
            params: Some(params),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let result: CallToolResult = serde_json::from_value(resp.result)
                    .map_err(|e| Error::ParseError(e.to_string()))?;
                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Set the logging level
    pub async fn set_logging_level(&self, level: LoggingLevel) -> Result<(), Error> {
        // Check if server supports logging
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.logging {
                return Err(Error::UnsupportedFeature("Logging".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "logging/setLevel".to_string(),
            params: Some(serde_json::json!({ "level": level })),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(_) => Ok(()),
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Get completions for an argument
    pub async fn get_completions(
        &self,
        reference_type: CompletionReferenceType,
        reference_name: &str,
        argument_name: &str,
        argument_value: &str,
    ) -> Result<CompleteResult, Error> {
        // Check if server supports completions
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.completions {
                return Err(Error::UnsupportedFeature("Completions".to_string()));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        let ref_obj = match reference_type {
            CompletionReferenceType::Prompt => {
                serde_json::json!({
                    "type": "ref/prompt",
                    "name": reference_name
                })
            }
            CompletionReferenceType::Resource => {
                serde_json::json!({
                    "type": "ref/resource",
                    "uri": reference_name
                })
            }
        };

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "completion/complete".to_string(),
            params: Some(serde_json::json!({
                "ref": ref_obj,
                "argument": {
                    "name": argument_name,
                    "value": argument_value
                }
            })),
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let result: CompleteResult = serde_json::from_value(resp.result)
                    .map_err(|e| Error::ParseError(e.to_string()))?;
                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// List available roots (from server to client)
    pub async fn list_roots(&self) -> Result<Vec<Root>, Error> {
        // This method handles the server requesting roots from the client
        // Usually the server would initiate this, but we provide it for testing/manual use

        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "roots/list".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(resp) => {
                let roots = resp.result["roots"]
                    .as_array()
                    .ok_or_else(|| Error::ParseError("Missing roots array".to_string()))?;

                let mut result = Vec::with_capacity(roots.len());
                for root in roots {
                    let r: Root = serde_json::from_value(root.clone())
                        .map_err(|e| Error::ParseError(e.to_string()))?;
                    result.push(r);
                }

                Ok(result)
            }
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Notify the server that the roots list has changed
    pub async fn notify_roots_changed(&self) -> Result<(), Error> {
        // Check if client has roots capability
        if !self.options.capabilities.roots || !self.options.capabilities.roots_list_changed {
            return Err(Error::UnsupportedFeature("Roots list changed notifications".to_string()));
        }

        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/roots/list_changed".to_string(),
            params: None,
        };

        self.send_notification(notification).await
    }

    /// Ping the server
    pub async fn ping(&self) -> Result<(), Error> {
        let request = JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: Uuid::new_v4().to_string().into(),
            method: "ping".to_string(),
            params: None,
        };

        let response = self.send_request(request).await?;

        match response {
            JSONRPCMessage::Response(_) => Ok(()),
            JSONRPCMessage::Error(err) => Err(Error::ServerError(
                err.error.code,
                err.error.message,
                err.error.data,
            )),
            _ => Err(Error::ProtocolError("Unexpected response type".to_string())),
        }
    }

    /// Cancel a request
    pub async fn cancel_request(
        &self,
        request_id: RequestId,
        reason: Option<String>,
    ) -> Result<(), Error> {
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/cancelled".to_string(),
            params: Some(serde_json::json!({
                "requestId": request_id,
                "reason": reason
            })),
        };

        self.send_notification(notification).await
    }
}

/// A resource content that can be text or binary
#[derive(Debug, Clone)]
pub enum ResourceContent {
    /// Text content
    Text(TextResourceContents),
    /// Binary content
    Blob(BlobResourceContents),
}

/// Reference type for completions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionReferenceType {
    /// Reference to a prompt
    Prompt,
    /// Reference to a resource
    Resource,
}
