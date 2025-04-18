//! Server service interface for implementing MCP services
//!
//! This module provides the interface for implementing MCP services that can
//! be attached to a server.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::Error;
use crate::protocol::{
    resources::{Resource, ResourceTemplate, TextResourceContents, BlobResourceContents},
    prompts::{Prompt, PromptMessage},
    tools::{Tool, CallToolResult},
    roots::Root,
    completion::CompleteResult,
    logging::LoggingLevel,
};

/// Service request types
#[derive(Debug, Clone)]
pub enum ServiceRequest {
    /// List resources
    ListResources {
        /// Pagination cursor
        cursor: Option<String>,
    },
    /// List resource templates
    ListResourceTemplates {
        /// Pagination cursor
        cursor: Option<String>,
    },
    /// Read a resource
    ReadResource {
        /// Resource URI
        uri: String,
    },
    /// Subscribe to a resource
    SubscribeResource {
        /// Resource URI
        uri: String,
    },
    /// Unsubscribe from a resource
    UnsubscribeResource {
        /// Resource URI
        uri: String,
    },
    /// List prompts
    ListPrompts {
        /// Pagination cursor
        cursor: Option<String>,
    },
    /// Get a prompt
    GetPrompt {
        /// Prompt name
        name: String,
        /// Prompt arguments
        arguments: Option<std::collections::HashMap<String, String>>,
    },
    /// List tools
    ListTools {
        /// Pagination cursor
        cursor: Option<String>,
    },
    /// Call a tool
    CallTool {
        /// Tool name
        name: String,
        /// Tool arguments
        arguments: Option<serde_json::Value>,
    },
    /// Set logging level
    SetLoggingLevel {
        /// Logging level
        level: LoggingLevel,
    },
    /// Get completions
    GetCompletions {
        /// Reference type
        reference_type: CompletionReferenceType,
        /// Reference name or URI
        reference_name: String,
        /// Argument name
        argument_name: String,
        /// Argument value
        argument_value: String,
    },
    /// List roots
    ListRoots,
}

/// Service response types
#[derive(Debug, Clone)]
pub enum ServiceResponse {
    /// List resources response
    ListResources {
        /// Resources
        resources: Vec<Resource>,
        /// Next pagination cursor
        next_cursor: Option<String>,
    },
    /// List resource templates response
    ListResourceTemplates {
        /// Resource templates
        templates: Vec<ResourceTemplate>,
        /// Next pagination cursor
        next_cursor: Option<String>,
    },
    /// Read resource response
    ReadResource {
        /// Resource contents
        contents: Vec<ResourceContent>,
    },
    /// Subscribe resource response
    SubscribeResource,
    /// Unsubscribe resource response
    UnsubscribeResource,
    /// List prompts response
    ListPrompts {
        /// Prompts
        prompts: Vec<Prompt>,
        /// Next pagination cursor
        next_cursor: Option<String>,
    },
    /// Get prompt response
    GetPrompt {
        /// Prompt messages
        messages: Vec<PromptMessage>,
        /// Prompt description
        description: Option<String>,
    },
    /// List tools response
    ListTools {
        /// Tools
        tools: Vec<Tool>,
        /// Next pagination cursor
        next_cursor: Option<String>,
    },
    /// Call tool response
    CallTool {
        /// Tool call result
        result: CallToolResult,
    },
    /// Set logging level response
    SetLoggingLevel,
    /// Get completions response
    GetCompletions {
        /// Completion result
        result: CompleteResult,
    },
    /// List roots response
    ListRoots {
        /// Roots
        roots: Vec<Root>,
    },
}

/// Reference type for completions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionReferenceType {
    /// Reference to a prompt
    Prompt,
    /// Reference to a resource
    Resource,
}

/// A resource content that can be text or binary
#[derive(Debug, Clone)]
pub enum ResourceContent {
    /// Text content
    Text(TextResourceContents),
    /// Binary content
    Blob(BlobResourceContents),
}

/// Context for a service request
pub struct ServiceContext {
    /// Client ID
    pub client_id: String,
    /// Is the client initialized?
    pub initialized: bool,
    /// Client implementation info (available after initialization)
    pub client_info: Option<crate::protocol::Implementation>,
    /// Client protocol version (available after initialization)
    pub protocol_version: Option<String>,
    /// Client capabilities (available after initialization)
    pub capabilities: Option<super::ClientCapabilities>,
    /// Server options
    pub server_options: super::ServerOptions,
}

/// Server service interface
#[async_trait]
pub trait ServerService: Send + Sync {
    /// Handle a service request
    async fn handle_request(
        &self,
        context: ServiceContext,
        request: ServiceRequest,
    ) -> Result<ServiceResponse, Error>;

    /// Notification that a client has connected
    async fn client_connected(
        &self,
        client_id: String,
        client_info: crate::protocol::Implementation,
        protocol_version: String,
        capabilities: super::ClientCapabilities,
    ) -> Result<(), Error> {
        // Default implementation does nothing
        Ok(())
    }

    /// Notification that a client has disconnected
    async fn client_disconnected(
        &self,
        client_id: String,
        reason: String,
    ) -> Result<(), Error> {
        // Default implementation does nothing
        Ok(())
    }

    /// Notification that a client's roots list has changed
    async fn roots_updated(
        &self,
        client_id: String,
    ) -> Result<(), Error> {
        // Default implementation does nothing
        Ok(())
    }
}
