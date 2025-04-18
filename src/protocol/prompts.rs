//! Protocol types and definitions for MCP prompts
//!
//! This module contains types for working with prompts in the MCP protocol,
//! including prompt templates, arguments, and messages.

use serde::{Deserialize, Serialize};
use crate::protocol::{
    Cursor,
    Role,
    Annotations,
    Request,
    Notification,
    resources::{TextResourceContents, BlobResourceContents},
    sampling::{TextContent, ImageContent, AudioContent},
};
use crate::protocol::messages::MessageResult;

/// A prompt or prompt template that the server offers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prompt {
    /// The name of the prompt or prompt template.
    pub name: String,
    /// An optional description of what this prompt provides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A list of arguments to use for templating the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Describes an argument that a prompt can accept.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptArgument {
    /// The name of the argument.
    pub name: String,
    /// A human-readable description of the argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this argument must be provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Describes a message returned as part of a prompt.
///
/// This is similar to `SamplingMessage`, but also supports the embedding of
/// resources from the MCP server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptMessage {
    /// The role of the message sender (user or assistant)
    pub role: Role,
    /// The content of the message
    #[serde(flatten)]
    pub content: PromptContent,
}

/// The contents of a message in a prompt template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PromptContent {
    /// Text content
    Text(TextContent),
    /// Image content
    Image(ImageContent),
    /// Audio content
    Audio(AudioContent),
    /// Embedded resource
    Resource(EmbeddedResource),
}

/// The contents of a resource, embedded into a prompt or tool call result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedResource {
    /// Type identifier for the content
    pub r#type: String,
    /// The embedded resource
    pub resource: ResourceContents,
    /// Optional annotations for the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

/// Contents of an embedded resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResourceContents {
    /// Text resource
    Text(TextResourceContents),
    /// Binary resource
    Blob(BlobResourceContents),
}

/// Identifies a prompt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptReference {
    /// Type identifier for the reference
    pub r#type: String,
    /// The name of the prompt or prompt template
    pub name: String,
}

impl PromptReference {
    /// Create a new prompt reference
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            r#type: "ref/prompt".to_string(),
            name: name.into(),
        }
    }
}

/// Sent from the client to request a list of prompts and prompt templates the server has.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<ListPromptsParams>,
}

/// Parameters for the list prompts request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsParams {
    /// An opaque token representing the current pagination position.
    /// If provided, the server should return results starting after this cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// The server's response to a prompts/list request from the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListPromptsResult {
    /// Available prompts
    pub prompts: Vec<Prompt>,
    /// An opaque token representing the pagination position after the last returned result.
    /// If present, there may be more results available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// Used by the client to get a prompt provided by the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    pub params: GetPromptParams,
}

/// Parameters for the get prompt request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptParams {
    /// The name of the prompt or prompt template.
    pub name: String,
    /// Arguments to use for templating the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<std::collections::HashMap<String, String>>,
}

/// The server's response to a prompts/get request from the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetPromptResult {
    /// An optional description for the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The prompt messages
    pub messages: Vec<PromptMessage>,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// An optional notification from the server to the client, informing it that the list of prompts
/// it offers has changed. This may be issued by servers without any previous subscription from the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptListChangedNotification {
    /// The method name
    pub method: String,
    /// The notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

// Implementation for Request trait
impl Request for ListPromptsRequest {
    const METHOD: &'static str = "prompts/list";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl Request for GetPromptRequest {
    const METHOD: &'static str = "prompts/get";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for Notification trait
impl Notification for PromptListChangedNotification {
    const METHOD: &'static str = "notifications/prompts/list_changed";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for MessageResult trait
impl MessageResult for ListPromptsResult {}
impl MessageResult for GetPromptResult {}

// Helper constructors
impl ListPromptsRequest {
    /// Create a new list prompts request
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }

    /// Create a new list prompts request with pagination
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: Some(ListPromptsParams {
                cursor: Some(cursor.into()),
            }),
        }
    }
}

impl GetPromptRequest {
    /// Create a new get prompt request
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: GetPromptParams {
                name: name.into(),
                arguments: None,
            },
        }
    }

    /// Create a new get prompt request with arguments
    pub fn with_arguments(
        name: impl Into<String>,
        arguments: std::collections::HashMap<String, String>,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: GetPromptParams {
                name: name.into(),
                arguments: Some(arguments),
            },
        }
    }
}

impl PromptListChangedNotification {
    /// Create a new prompt list changed notification
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }
}
