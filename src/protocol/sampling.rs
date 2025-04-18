//! Sampling message types for the MCP protocol

use serde::{Deserialize, Serialize};
use super::{Role, Annotations};
use std::collections::HashMap;

/// The client's response to a sampling/create_message request from the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageResult {
    /// Metadata for the result
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<super::messages::ResultMeta>,
    
    /// The content of the message
    pub content: ContentType,
    
    /// The role that produced the message
    pub role: Role,
    
    /// The name of the model that generated the message
    pub model: String,
    
    /// The reason why sampling stopped, if known
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

impl CreateMessageResult {
    /// Create a new message result
    pub fn new(role: Role, content: ContentType, model: impl Into<String>) -> Self {
        Self {
            meta: None,
            content,
            role,
            model: model.into(),
            stop_reason: None,
        }
    }

    /// Create a new message result with a stop reason
    pub fn with_stop_reason(
        role: Role,
        content: ContentType,
        model: impl Into<String>,
        stop_reason: impl Into<String>,
    ) -> Self {
        Self {
            meta: None,
            content,
            role,
            model: model.into(),
            stop_reason: Some(stop_reason.into()),
        }
    }
}

/// A request from the server to sample an LLM via the client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageRequest {
    /// Method is always "sampling/createMessage"
    pub method: String,
    
    /// Parameters for the request
    pub params: CreateMessageParams,
}

impl CreateMessageRequest {
    /// Create a new create message request
    pub fn new(messages: Vec<SamplingMessage>, max_tokens: i32) -> Self {
        Self {
            method: "sampling/createMessage".to_string(),
            params: CreateMessageParams {
                messages,
                model_preferences: None,
                system_prompt: None,
                include_context: None,
                temperature: None,
                max_tokens,
                stop_sequences: None,
                metadata: None,
            },
        }
    }
}

/// Parameters for a create message request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateMessageParams {
    /// Messages in the conversation history
    pub messages: Vec<SamplingMessage>,
    
    /// The server's preferences for which model to select
    #[serde(rename = "modelPreferences", skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
    
    /// An optional system prompt
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    
    /// A request to include context from one or more MCP servers
    #[serde(rename = "includeContext", skip_serializing_if = "Option::is_none")]
    pub include_context: Option<IncludeContext>,
    
    /// Sampling temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    
    /// Maximum number of tokens to sample
    #[serde(rename = "maxTokens")]
    pub max_tokens: i32,
    
    /// Optional stop sequences
    #[serde(rename = "stopSequences", skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    
    /// Optional metadata to pass through to the LLM provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Include context option for sampling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum IncludeContext {
    /// Don't include any server context
    None,
    /// Include context from this server only
    ThisServer,
    /// Include context from all servers
    AllServers,
}

/// A message in a sampling conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SamplingMessage {
    /// The role of the message sender
    pub role: Role,
    
    /// The content of the message
    pub content: ContentType,
}

impl SamplingMessage {
    /// Create a new message with text content
    pub fn text(role: Role, text: impl Into<String>) -> Self {
        Self {
            role,
            content: ContentType::Text(TextContent::new(text)),
        }
    }

    /// Create a new message with image content
    pub fn image(role: Role, data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            role,
            content: ContentType::Image(ImageContent::new(data, mime_type)),
        }
    }

    /// Create a new message with audio content
    pub fn audio(role: Role, data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            role,
            content: ContentType::Audio(AudioContent::new(data, mime_type)),
        }
    }
}

/// Content type for a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentType {
    /// Text content
    Text(TextContent),
    /// Image content
    Image(ImageContent),
    /// Audio content
    Audio(AudioContent),
}

/// Text content in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    /// Content type, always "text"
    pub r#type: String,
    
    /// The text content
    pub text: String,
    
    /// Optional annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl TextContent {
    /// Create new text content
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            r#type: "text".to_string(),
            text: text.into(),
            annotations: None,
        }
    }

    /// Create new text content with annotations
    pub fn with_annotations(text: impl Into<String>, annotations: Annotations) -> Self {
        Self {
            r#type: "text".to_string(),
            text: text.into(),
            annotations: Some(annotations),
        }
    }
}

/// Image content in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageContent {
    /// Content type, always "image"
    pub r#type: String,
    
    /// The base64-encoded image data
    pub data: String,
    
    /// The MIME type of the image
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    
    /// Optional annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl ImageContent {
    /// Create new image content
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            r#type: "image".to_string(),
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    /// Create new image content with annotations
    pub fn with_annotations(
        data: impl Into<String>,
        mime_type: impl Into<String>,
        annotations: Annotations,
    ) -> Self {
        Self {
            r#type: "image".to_string(),
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: Some(annotations),
        }
    }
}

/// Audio content in a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioContent {
    /// Content type, always "audio"
    pub r#type: String,
    
    /// The base64-encoded audio data
    pub data: String,
    
    /// The MIME type of the audio
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    
    /// Optional annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl AudioContent {
    /// Create new audio content
    pub fn new(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            r#type: "audio".to_string(),
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    /// Create new audio content with annotations
    pub fn with_annotations(
        data: impl Into<String>,
        mime_type: impl Into<String>,
        annotations: Annotations,
    ) -> Self {
        Self {
            r#type: "audio".to_string(),
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: Some(annotations),
        }
    }
}

/// Model preferences for sampling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelPreferences {
    /// Optional hints for model selection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hints: Option<Vec<ModelHint>>,
    
    /// How much to prioritize cost when selecting a model
    #[serde(rename = "costPriority", skip_serializing_if = "Option::is_none")]
    pub cost_priority: Option<f64>,
    
    /// How much to prioritize sampling speed when selecting a model
    #[serde(rename = "speedPriority", skip_serializing_if = "Option::is_none")]
    pub speed_priority: Option<f64>,
    
    /// How much to prioritize intelligence when selecting a model
    #[serde(rename = "intelligencePriority", skip_serializing_if = "Option::is_none")]
    pub intelligence_priority: Option<f64>,
}

impl ModelPreferences {
    /// Create new empty model preferences
    pub fn new() -> Self {
        Self::default()
    }

    /// Create model preferences with priorities
    pub fn with_priorities(cost: f64, speed: f64, intelligence: f64) -> Self {
        // Clamp priorities between 0 and 1
        let cost = cost.max(0.0).min(1.0);
        let speed = speed.max(0.0).min(1.0);
        let intelligence = intelligence.max(0.0).min(1.0);
        
        Self {
            hints: None,
            cost_priority: Some(cost),
            speed_priority: Some(speed),
            intelligence_priority: Some(intelligence),
        }
    }

    /// Add a model hint
    pub fn add_hint(&mut self, hint: ModelHint) {
        if let Some(hints) = &mut self.hints {
            hints.push(hint);
        } else {
            self.hints = Some(vec![hint]);
        }
    }

    /// Set the cost priority
    pub fn set_cost_priority(&mut self, priority: f64) {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        self.cost_priority = Some(priority);
    }

    /// Set the speed priority
    pub fn set_speed_priority(&mut self, priority: f64) {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        self.speed_priority = Some(priority);
    }

    /// Set the intelligence priority
    pub fn set_intelligence_priority(&mut self, priority: f64) {
        // Clamp priority between 0 and 1
        let priority = priority.max(0.0).min(1.0);
        self.intelligence_priority = Some(priority);
    }
}

/// Hint for model selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelHint {
    /// A hint for a model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// Additional properties for the hint
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl ModelHint {
    /// Create a new model hint with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            additional: HashMap::new(),
        }
    }

    /// Add a custom hint property
    pub fn add_property<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}
