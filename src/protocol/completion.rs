//! Protocol types and definitions for MCP completion
//!
//! This module contains types for working with completion suggestions in the MCP protocol.

use serde::{Deserialize, Serialize};
use crate::protocol::{
    Request,
    prompts::PromptReference,
    resources::ResourceReference,
};
use crate::protocol::messages::MessageResult;

/// A request from the client to the server, to ask for completion options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    pub params: CompleteParams,
}

/// Parameters for the complete request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteParams {
    /// Reference to a prompt or resource for completion
    #[serde(flatten)]
    pub ref_type: CompletionReference,
    /// The argument's information
    pub argument: CompletionArgument,
}

/// Reference to a prompt or resource for completion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CompletionReference {
    /// Reference to a prompt
    Prompt(PromptReference),
    /// Reference to a resource
    Resource(ResourceReference),
}

/// The argument information for a completion request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionArgument {
    /// The name of the argument
    pub name: String,
    /// The value of the argument to use for completion matching
    pub value: String,
}

/// The server's response to a completion/complete request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompleteResult {
    /// The completion suggestions
    pub completion: CompletionResults,
    /// Optional metadata
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<serde_json::Value>,
}

/// The completion results
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionResults {
    /// An array of completion values. Must not exceed 100 items.
    pub values: Vec<String>,
    /// The total number of completion options available.
    /// This can exceed the number of values actually sent in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    /// Indicates whether there are additional completion options beyond those
    /// provided in the current response, even if the exact total is unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
}

// Implementation for Request trait
impl Request for CompleteRequest {
    const METHOD: &'static str = "completion/complete";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        // This is a placeholder implementation since we can't directly return &self.params
        // In a real implementation, we would need to convert the params to a serde_json::Value
        None
    }
}

// Implementation for MessageResult trait
impl MessageResult for CompleteResult {}

// Helper constructors
impl CompleteRequest {
    /// Create a new completion request for a prompt
    pub fn for_prompt(
        prompt_name: impl Into<String>,
        argument_name: impl Into<String>,
        argument_value: impl Into<String>,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: CompleteParams {
                ref_type: CompletionReference::Prompt(PromptReference::new(prompt_name)),
                argument: CompletionArgument {
                    name: argument_name.into(),
                    value: argument_value.into(),
                },
            },
        }
    }

    /// Create a new completion request for a resource
    pub fn for_resource(
        resource_uri: impl Into<String>,
        argument_name: impl Into<String>,
        argument_value: impl Into<String>,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: CompleteParams {
                ref_type: CompletionReference::Resource(ResourceReference {
                    r#type: "ref/resource".to_string(),
                    uri: resource_uri.into(),
                }),
                argument: CompletionArgument {
                    name: argument_name.into(),
                    value: argument_value.into(),
                },
            },
        }
    }
}

impl CompleteResult {
    /// Create a new completion result with values
    pub fn new(values: Vec<String>) -> Self {
        Self {
            completion: CompletionResults {
                values,
                total: None,
                has_more: None,
            },
            meta: None,
        }
    }

    /// Create a new completion result with values and pagination info
    pub fn with_pagination(
        values: Vec<String>,
        total: i64,
        has_more: bool,
    ) -> Self {
        Self {
            completion: CompletionResults {
                values,
                total: Some(total),
                has_more: Some(has_more),
            },
            meta: None,
        }
    }
}
