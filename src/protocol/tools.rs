//! Protocol types and definitions for MCP tools
//!
//! This module contains types for working with tools in the MCP protocol,
//! including tool definitions, arguments, and call results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::protocol::{
    Cursor,
    Request,
    Notification,
    sampling::{TextContent, ImageContent, AudioContent},
    prompts::EmbeddedResource,
};
use crate::protocol::messages::MessageResult;

/// Definition for a tool the client can call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// The name of the tool.
    pub name: String,
    /// A human-readable description of the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON Schema object defining the expected parameters for the tool.
    pub input_schema: ToolInputSchema,
    /// Optional additional tool information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
}

/// Input schema for a tool, defining the expected parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolInputSchema {
    /// The schema type (always "object")
    pub r#type: String,
    /// Properties the tool accepts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// Required properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// Additional properties describing a Tool to clients.
///
/// NOTE: all properties in ToolAnnotations are **hints**.
/// They are not guaranteed to provide a faithful description of
/// tool behavior (including descriptive properties like `title`).
///
/// Clients should never make tool use decisions based on ToolAnnotations
/// received from untrusted servers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolAnnotations {
    /// A human-readable title for the tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// If true, the tool does not modify its environment.
    ///
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    /// If true, the tool may perform destructive updates to its environment.
    /// If false, the tool performs only additive updates.
    ///
    /// (This property is meaningful only when `read_only_hint == false`)
    ///
    /// Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    /// If true, calling the tool repeatedly with the same arguments
    /// will have no additional effect on the its environment.
    ///
    /// (This property is meaningful only when `read_only_hint == false`)
    ///
    /// Default: false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    /// If true, this tool may interact with an "open world" of external
    /// entities. If false, the tool's domain of interaction is closed.
    /// For example, the world of a web search tool is open, whereas that
    /// of a memory tool is not.
    ///
    /// Default: true
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Sent from the client to request a list of tools the server has.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<ListToolsParams>,
}

/// Parameters for the list tools request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsParams {
    /// An opaque token representing the current pagination position.
    /// If provided, the server should return results starting after this cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// The server's response to a tools/list request from the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListToolsResult {
    /// Available tools
    pub tools: Vec<Tool>,
    /// An opaque token representing the pagination position after the last returned result.
    /// If present, there may be more results available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Used by the client to invoke a tool provided by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    pub params: CallToolParams,
}

/// Parameters for the call tool request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolParams {
    /// The name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// The server's response to a tool call.
///
/// Any errors that originate from the tool SHOULD be reported inside the result
/// object, with `isError` set to true, _not_ as an MCP protocol-level error
/// response. Otherwise, the LLM would not be able to see that an error occurred
/// and self-correct.
///
/// However, any errors in _finding_ the tool, an error indicating that the
/// server does not support tool calls, or any other exceptional conditions,
/// should be reported as an MCP error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CallToolResult {
    /// The content returned from the tool call
    pub content: Vec<ToolCallContent>,
    /// Whether the tool call ended in an error.
    ///
    /// If not set, this is assumed to be false (the call was successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

impl MessageResult for CallToolResult {}

/// Content that can be returned from a tool call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolCallContent {
    /// Text content
    Text(TextContent),
    /// Image content
    Image(ImageContent),
    /// Audio content
    Audio(AudioContent),
    /// Embedded resource
    Resource(EmbeddedResource),
}

/// An optional notification from the server to the client, informing it that the list of tools
/// it offers has changed. This may be issued by servers without any previous subscription from the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolListChangedNotification {
    /// The method name
    pub method: String,
    /// The notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

// Implementation for Request trait
impl Request for ListToolsRequest {
    const METHOD: &'static str = "tools/list";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl Request for CallToolRequest {
    const METHOD: &'static str = "tools/call";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for Notification trait
impl Notification for ToolListChangedNotification {
    const METHOD: &'static str = "notifications/tools/list_changed";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for MessageResult trait
impl MessageResult for ListToolsResult {}

// Helper constructors
impl ListToolsRequest {
    /// Create a new list tools request
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }

    /// Create a new list tools request with pagination
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: Some(ListToolsParams {
                cursor: Some(cursor.into()),
            }),
        }
    }
}

impl CallToolRequest {
    /// Create a new call tool request
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: CallToolParams {
                name: name.into(),
                arguments: None,
            },
        }
    }

    /// Create a new call tool request with arguments
    pub fn with_arguments(
        name: impl Into<String>,
        arguments: serde_json::Value,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: CallToolParams {
                name: name.into(),
                arguments: Some(arguments),
            },
        }
    }
}

impl ToolListChangedNotification {
    /// Create a new tool list changed notification
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }
}

impl Tool {
    /// Create a new tool with basic properties
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema: ToolInputSchema {
                r#type: "object".to_string(),
                properties: None,
                required: None,
            },
            annotations: None,
        }
    }

    /// Set the input schema for the tool
    pub fn with_schema(mut self, properties: HashMap<String, serde_json::Value>, required: Vec<String>) -> Self {
        self.input_schema = ToolInputSchema {
            r#type: "object".to_string(),
            properties: Some(properties),
            required: Some(required),
        };
        self
    }

    /// Add annotations to the tool
    pub fn with_annotations(mut self, annotations: ToolAnnotations) -> Self {
        self.annotations = Some(annotations);
        self
    }
}

impl ToolAnnotations {
    /// Create new tool annotations
    pub fn new() -> Self {
        Self {
            title: None,
            read_only_hint: None,
            destructive_hint: None,
            idempotent_hint: None,
            open_world_hint: None,
        }
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Mark the tool as read-only
    pub fn read_only(mut self) -> Self {
        self.read_only_hint = Some(true);
        self
    }

    /// Set whether the tool is destructive
    pub fn destructive(mut self, is_destructive: bool) -> Self {
        self.destructive_hint = Some(is_destructive);
        self
    }

    /// Set whether the tool is idempotent
    pub fn idempotent(mut self, is_idempotent: bool) -> Self {
        self.idempotent_hint = Some(is_idempotent);
        self
    }

    /// Set whether the tool interacts with an open world
    pub fn open_world(mut self, is_open_world: bool) -> Self {
        self.open_world_hint = Some(is_open_world);
        self
    }
}
