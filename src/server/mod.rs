//! MCP server implementation
//!
//! This module provides a server implementation for the Model Context Protocol (MCP).
//! It handles connection management, protocol encoding/decoding, and provides
//! a convenient API for implementing MCP servers.

mod builder;
mod handler;
mod state;
mod service;

pub use builder::ServerBuilder;
pub use service::{ServerService, ServiceContext, ServiceRequest, ServiceResponse};

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use async_trait::async_trait;
use dashmap::DashMap;
use uuid::Uuid;

use crate::error::Error;
use crate::protocol::{
    Implementation, RequestId, ProgressToken,
    JSONRPCMessage, JSONRPCNotification,
    logging::LoggingLevel,
};

use self::state::{ServerState, Connection};
use self::handler::ServerMessageHandler;

/// Server capability flags and settings
#[derive(Debug, Clone)]
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
    pub experimental: HashMap<String, serde_json::Value>,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            logging: true,
            completions: false,
            prompts: false,
            prompts_list_changed: false,
            resources: false,
            resources_list_changed: false,
            resources_subscribe: false,
            tools: false,
            tools_list_changed: false,
            experimental: HashMap::new(),
        }
    }
}

/// MCP server options
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Server implementation info
    pub implementation: Implementation,
    /// Server capabilities
    pub capabilities: ServerCapabilities,
    /// Server instructions
    pub instructions: Option<String>,
    /// Automatically acknowledge ping requests
    pub auto_acknowledge_ping: bool,
    /// Default timeout for requests in milliseconds (0 = no timeout)
    pub default_timeout_ms: u64,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            implementation: Implementation::new("mcpx-server", env!("CARGO_PKG_VERSION")),
            capabilities: ServerCapabilities::default(),
            instructions: None,
            auto_acknowledge_ping: true,
            default_timeout_ms: 30000, // 30 seconds
        }
    }
}

/// An event that can be emitted by the server
#[derive(Debug, Clone)]
pub enum ServerEvent {
    /// Client connected
    ClientConnected {
        /// Client ID
        client_id: String,
        /// Client implementation info
        client_info: Implementation,
        /// Client protocol version
        protocol_version: String,
        /// Client capabilities
        capabilities: ClientCapabilities,
    },
    /// Client disconnected
    ClientDisconnected {
        /// Client ID
        client_id: String,
        /// Reason for disconnection
        reason: String,
    },
    /// Roots list updated
    RootsUpdated {
        /// Client ID
        client_id: String,
    },
    /// Error occurred
    Error {
        /// Client ID (if any)
        client_id: Option<String>,
        /// Error details
        error: Error,
    },
}

/// Client capabilities
#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    /// Whether the client supports listing roots
    pub roots: bool,
    /// Whether the client supports notifications for changes to the roots list
    pub roots_list_changed: bool,
    /// Whether the client supports sampling from an LLM
    pub sampling: bool,
    /// Experimental capabilities
    pub experimental: HashMap<String, serde_json::Value>,
}

/// MCP server
pub struct Server {
    /// Unique server identifier
    id: String,
    /// Server state
    state: Arc<RwLock<ServerState>>,
    /// Connection tracking
    connections: Arc<DashMap<String, Connection>>,
    /// Event sender
    event_sender: mpsc::Sender<ServerEvent>,
    /// Server options
    options: ServerOptions,
    /// Message handler
    handler: Arc<ServerMessageHandler>,
    /// Service implementation
    service: Arc<Box<dyn ServerService + Send + Sync>>,
}

/// Server event listener
#[async_trait]
pub trait EventListener: Send + Sync {
    /// Called when a server event occurs
    async fn on_event(&self, event: ServerEvent);
}

impl Server {
    /// Create a new server with the given options and service
    pub fn new(
        options: ServerOptions,
        service: Box<dyn ServerService + Send + Sync>,
    ) -> (Self, mpsc::Receiver<ServerEvent>) {
        let id = Uuid::new_v4().to_string();
        let (event_sender, event_receiver) = mpsc::channel(100);

        let state = Arc::new(RwLock::new(ServerState::new()));
        let connections = Arc::new(DashMap::new());

        let handler = Arc::new(ServerMessageHandler::new(
            state.clone(),
            connections.clone(),
            event_sender.clone(),
            options.clone(),
        ));

        let server = Self {
            id,
            state,
            connections,
            event_sender,
            options,
            handler,
            service: Arc::new(service),
        };

        (server, event_receiver)
    }

    /// Get the server ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Start the server
    pub async fn start(&self) -> Result<(), Error> {
        // Set server state to running
        let mut state = self.state.write().await;
        state.set_running();

        Ok(())
    }

    /// Stop the server
    pub async fn stop(&self) -> Result<(), Error> {
        // Set server state to stopping
        let mut state = self.state.write().await;
        state.set_stopping();

        // Disconnect all clients
        self.connections.clear();

        // Set server state to stopped
        state.set_stopped();

        Ok(())
    }

    /// Add a connection to the server
    pub async fn add_connection(&self, id: &str) -> Result<(), Error> {
        let connection = Connection::new(id);
        self.connections.insert(id.to_string(), connection);
        Ok(())
    }

    /// Remove a connection from the server
    pub async fn remove_connection(&self, id: &str) -> Result<(), Error> {
        self.connections.remove(id);
        Ok(())
    }

    /// Handle an incoming message from a client
    pub async fn handle_message(
        &self,
        client_id: &str,
        message: JSONRPCMessage,
    ) -> Result<Option<JSONRPCMessage>, Error> {
        // Check if client exists
        if !self.connections.contains_key(client_id) {
            return Err(Error::InternalError(format!("Unknown client: {}", client_id)));
        }

        // Handle the message
        let response = self.handler.handle_message(client_id, message).await?;

        Ok(response)
    }

    /// Send a notification to a specific client
    pub async fn send_notification(
        &self,
        client_id: &str,
        _notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        // Check if client exists
        if !self.connections.contains_key(client_id) {
            return Err(Error::InternalError(format!("Unknown client: {}", client_id)));
        }

        // Send notification through transport - this would be handled elsewhere
        // For now, we just return success
        Ok(())
    }

    /// Send a log message to a client
    pub async fn send_log(
        &self,
        client_id: &str,
        level: LoggingLevel,
        message: &str,
    ) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports logging
        if !connection.capabilities.logging {
            return Err(Error::UnsupportedFeature("Logging".to_string()));
        }

        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/message".to_string(),
            params: Some(serde_json::json!({
                "level": level,
                "data": message
            })),
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Notify a client that resources have changed
    pub async fn notify_resources_changed(&self, client_id: &str) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports resource list changed notifications
        if !connection.capabilities.resources_list_changed {
            return Err(Error::UnsupportedFeature("Resource list changed notifications".to_string()));
        }

        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/resources/list_changed".to_string(),
            params: None,
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Notify a client that a resource has been updated
    pub async fn notify_resource_updated(
        &self,
        client_id: &str,
        uri: &str,
    ) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports resource subscriptions
        if !connection.capabilities.resources_subscribe {
            return Err(Error::UnsupportedFeature("Resource subscriptions".to_string()));
        }

        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/resources/updated".to_string(),
            params: Some(serde_json::json!({
                "uri": uri
            })),
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Notify a client that prompts have changed
    pub async fn notify_prompts_changed(&self, client_id: &str) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports prompts list changed notifications
        if !connection.capabilities.prompts_list_changed {
            return Err(Error::UnsupportedFeature("Prompts list changed notifications".to_string()));
        }

        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/prompts/list_changed".to_string(),
            params: None,
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Notify a client that tools have changed
    pub async fn notify_tools_changed(&self, client_id: &str) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports tools list changed notifications
        if !connection.capabilities.tools_list_changed {
            return Err(Error::UnsupportedFeature("Tools list changed notifications".to_string()));
        }

        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/tools/list_changed".to_string(),
            params: None,
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Send a progress update for a request
    pub async fn send_progress(
        &self,
        client_id: &str,
        token: ProgressToken,
        progress: f64,
        total: Option<f64>,
        message: Option<&str>,
    ) -> Result<(), Error> {
        // Create notification
        let mut params = serde_json::json!({
            "progressToken": token,
            "progress": progress
        });

        if let Some(total) = total {
            params["total"] = serde_json::json!(total);
        }

        if let Some(message) = message {
            params["message"] = serde_json::json!(message);
        }

        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/progress".to_string(),
            params: Some(params),
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Cancel a request
    pub async fn cancel_request(
        &self,
        client_id: &str,
        request_id: RequestId,
        reason: Option<String>,
    ) -> Result<(), Error> {
        // Create notification
        let notification = JSONRPCNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/cancelled".to_string(),
            params: Some(serde_json::json!({
                "requestId": request_id,
                "reason": reason
            })),
        };

        // Send notification
        self.send_notification(client_id, notification).await
    }

    /// Request roots from a client
    pub async fn request_roots(&self, client_id: &str) -> Result<(), Error> {
        // Get the connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports roots
        if !connection.capabilities.roots {
            return Err(Error::UnsupportedFeature("Roots".to_string()));
        }

        // Create request (would be sent in a real implementation)
        // JSONRPCRequest {
        //     jsonrpc: "2.0".to_string(),
        //     id: Uuid::new_v4().to_string().into(),
        //     method: "roots/list".to_string(),
        //     params: None,
        // };

        // Send request - again, this would be handled elsewhere
        // For now, just track the request

        Ok(())
    }
}
