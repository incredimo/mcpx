//! Protocol types and definitions for MCP roots
//!
//! This module contains types for working with file system roots in the MCP protocol.

use serde::{Deserialize, Serialize};
use crate::protocol::{Request, Notification};
use crate::protocol::messages::MessageResult;

/// Represents a root directory or file that the server can operate on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Root {
    /// The URI identifying the root. This *must* start with file:// for now.
    /// This restriction may be relaxed in future versions of the protocol to allow
    /// other URI schemes.
    pub uri: String,
    /// An optional name for the root. This can be used to provide a human-readable
    /// identifier for the root, which may be useful for display purposes or for
    /// referencing the root in other parts of the application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Sent from the server to request a list of root URIs from the client. Roots allow
/// servers to ask for specific directories or files to operate on. A common example
/// for roots is providing a set of repositories or directories a server should operate
/// on.
///
/// This request is typically used when the server needs to understand the file system
/// structure or access specific locations that the client has permission to read from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListRootsRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// The client's response to a roots/list request from the server.
/// This result contains an array of Root objects, each representing a root directory
/// or file that the server can operate on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListRootsResult {
    /// The available roots
    pub roots: Vec<Root>,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// A notification from the client to the server, informing it that the list of roots has changed.
/// This notification should be sent whenever the client adds, removes, or modifies any root.
/// The server should then request an updated list of roots using the ListRootsRequest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RootsListChangedNotification {
    /// The method name
    pub method: String,
    /// The notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

// Implementation for Request trait
impl Request for ListRootsRequest {
    const METHOD: &'static str = "roots/list";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for Notification trait
impl Notification for RootsListChangedNotification {
    const METHOD: &'static str = "notifications/roots/list_changed";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for MessageResult trait
impl MessageResult for ListRootsResult {}

// Helper constructors
impl Root {
    /// Create a new root with a URI
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: None,
        }
    }

    /// Create a new root with a URI and name
    pub fn with_name(uri: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: Some(name.into()),
        }
    }

    /// Verify if the root URI is valid (currently must start with file://)
    pub fn is_valid(&self) -> bool {
        self.uri.starts_with("file://")
    }
}

impl ListRootsRequest {
    /// Create a new list roots request
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }
}

impl ListRootsResult {
    /// Create a new list roots result
    pub fn new(roots: Vec<Root>) -> Self {
        Self {
            roots,
            meta: None,
        }
    }

    /// Create a new list roots result with a single root
    pub fn with_single_root(uri: impl Into<String>) -> Self {
        Self {
            roots: vec![Root::new(uri)],
            meta: None,
        }
    }
}

impl RootsListChangedNotification {
    /// Create a new roots list changed notification
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }
}
