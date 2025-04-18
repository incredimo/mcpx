//! Protocol types and definitions for the Model Context Protocol (MCP)
//!
//! This module contains the core types and definitions for the MCP protocol,
//! based on the specification version 2025-03-26.

pub mod json_rpc;
pub mod messages;
pub mod resources;
pub mod prompts;
pub mod tools;
pub mod sampling;
pub mod annotations;
pub mod logging;
pub mod completion;
pub mod roots;

use serde::{Deserialize, Serialize};

/// Latest protocol version supported by this library
pub const LATEST_PROTOCOL_VERSION: &str = "2025-03-26";

/// Describes the name and version of an MCP implementation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Implementation {
    /// Name of the implementation
    pub name: String,
    /// Version of the implementation
    pub version: String,
}

impl Implementation {
    /// Create a new Implementation
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// The sender or recipient of messages and data in a conversation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// A user or human participant
    User,
    /// An AI assistant
    Assistant,
}

/// An opaque token used to represent a cursor for pagination.
pub type Cursor = String;

/// A progress token, used to associate progress notifications with the original request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ProgressToken {
    /// String token
    String(String),
    /// Integer token
    Integer(i64),
}

/// A unique request ID for JSON-RPC messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// String ID
    String(String),
    /// Integer ID
    Integer(i64),
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        Self::String(s.to_owned())
    }
}

impl From<i64> for RequestId {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<i32> for RequestId {
    fn from(i: i32) -> Self {
        Self::Integer(i as i64)
    }
}

impl From<u32> for RequestId {
    fn from(i: u32) -> Self {
        Self::Integer(i as i64)
    }
}

// Re-export common types for convenience
pub use self::annotations::Annotations;
pub use self::json_rpc::{JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification};
pub use self::messages::{Request, Notification, Result as MessageResult};
pub use self::resources::{Resource, ResourceTemplate, ResourceContents, TextResourceContents, BlobResourceContents};
pub use self::prompts::{Prompt, PromptArgument, PromptMessage};
pub use self::tools::{Tool, ToolAnnotations};
pub use self::sampling::{SamplingMessage, TextContent, ImageContent, AudioContent};
