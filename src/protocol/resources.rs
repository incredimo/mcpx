//! Resource types for the MCP protocol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Annotations, Request, Notification};
use super::messages::MessageResult;

/// A known resource that the server is capable of reading.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    /// The URI of this resource.
    pub uri: String,

    /// A human-readable name for this resource.
    pub name: String,

    /// A description of what this resource represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Optional annotations for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,

    /// The size of the raw resource content in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

impl Resource {
    /// Create a new resource
    pub fn new(uri: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
        }
    }

    /// Create a new resource with description
    pub fn with_description(uri: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            name: name.into(),
            description: Some(description.into()),
            mime_type: None,
            annotations: None,
            size: None,
        }
    }

    /// Set the MIME type for the resource
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set the annotations for the resource
    pub fn with_annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Set the size for the resource
    pub fn with_size(mut self, size: i64) -> Self {
        self.size = Some(size);
        self
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self {
            uri: String::new(),
            name: String::new(),
            description: None,
            mime_type: None,
            annotations: None,
            size: None,
        }
    }
}

/// A template description for resources available on the server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceTemplate {
    /// A URI template that can be used to construct resource URIs.
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,

    /// A human-readable name for the type of resource this template refers to.
    pub name: String,

    /// A description of what this template is for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The MIME type for all resources that match this template.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Optional annotations for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl ResourceTemplate {
    /// Create a new resource template
    pub fn new(uri_template: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uri_template: uri_template.into(),
            name: name.into(),
            description: None,
            mime_type: None,
            annotations: None,
        }
    }

    /// Create a new resource template with description
    pub fn with_description(
        uri_template: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            uri_template: uri_template.into(),
            name: name.into(),
            description: Some(description.into()),
            mime_type: None,
            annotations: None,
        }
    }

    /// Set the MIME type for the resource template
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set the annotations for the resource template
    pub fn with_annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }
}

/// The contents of a specific resource or sub-resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceContents {
    /// The URI of this resource.
    pub uri: String,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ResourceContents {
    /// Create new resource contents
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: None,
        }
    }

    /// Create new resource contents with MIME type
    pub fn with_mime_type(uri: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
        }
    }
}

/// Text contents of a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextResourceContents {
    /// The URI of this resource.
    pub uri: String,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// The text of the item.
    pub text: String,
}

impl TextResourceContents {
    /// Create new text resource contents
    pub fn new(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: None,
            text: text.into(),
        }
    }

    /// Create new text resource contents with MIME type
    pub fn with_mime_type(uri: impl Into<String>, text: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
            text: text.into(),
        }
    }
}

/// Binary contents of a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlobResourceContents {
    /// The URI of this resource.
    pub uri: String,

    /// The MIME type of this resource, if known.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// The base64-encoded binary data of the item.
    pub blob: String,
}

impl BlobResourceContents {
    /// Create new blob resource contents
    pub fn new(uri: impl Into<String>, blob: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: None,
            blob: blob.into(),
        }
    }

    /// Create new blob resource contents with MIME type
    pub fn with_mime_type(uri: impl Into<String>, blob: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            mime_type: Some(mime_type.into()),
            blob: blob.into(),
        }
    }
}

/// Request to list resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesRequest {
    /// Method is always "resources/list"
    pub method: String,

    /// Optional request parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<ListResourcesParams>,
}

impl Request for ListResourcesRequest {
    const METHOD: &'static str = "resources/list";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl ListResourcesRequest {
    /// Create a new list resources request
    pub fn new() -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: None,
        }
    }

    /// Create a new list resources request with a cursor
    pub fn with_cursor(cursor: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: Some(ListResourcesParams {
                cursor: Some(cursor.into()),
            }),
        }
    }
}

/// Parameters for list resources request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesParams {
    /// An opaque token representing the current pagination position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Result of listing resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListResourcesResult {
    /// The list of resources
    pub resources: Vec<Resource>,

    /// An opaque token representing the pagination position after the last returned result.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Metadata for the result
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<super::messages::ResultMeta>,
}

impl MessageResult for ListResourcesResult {}

impl ListResourcesResult {
    /// Create a new list resources result
    pub fn new(resources: Vec<Resource>) -> Self {
        Self {
            resources,
            next_cursor: None,
            meta: None,
        }
    }

    /// Create a new list resources result with a next cursor
    pub fn with_next_cursor(resources: Vec<Resource>, next_cursor: impl Into<String>) -> Self {
        Self {
            resources,
            next_cursor: Some(next_cursor.into()),
            meta: None,
        }
    }
}

/// Request to read a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceRequest {
    /// Method is always "resources/read"
    pub method: String,

    /// Request parameters
    pub params: ReadResourceParams,
}

impl Request for ReadResourceRequest {
    const METHOD: &'static str = "resources/read";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl ReadResourceRequest {
    /// Create a new read resource request
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: ReadResourceParams {
                uri: uri.into(),
            },
        }
    }
}

/// Parameters for read resource request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceParams {
    /// The URI of the resource to read.
    pub uri: String,
}

/// Result of reading a resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReadResourceResult {
    /// The contents of the resource
    pub contents: Vec<ResourceContent>,

    /// Metadata for the result
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<super::messages::ResultMeta>,
}

impl ReadResourceResult {
    /// Create a new read resource result
    pub fn new(contents: Vec<ResourceContent>) -> Self {
        Self {
            contents,
            meta: None,
        }
    }
}

/// Content of a resource, either text or binary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ResourceContent {
    /// Text content
    Text(TextResourceContents),
    /// Binary content
    Blob(BlobResourceContents),
}

/// Request to subscribe to resource updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeRequest {
    /// Method is always "resources/subscribe"
    pub method: String,

    /// Request parameters
    pub params: SubscribeParams,
}

impl Request for SubscribeRequest {
    const METHOD: &'static str = "resources/subscribe";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl SubscribeRequest {
    /// Create a new subscribe request
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: SubscribeParams {
                uri: uri.into(),
            },
        }
    }
}

/// Parameters for subscribe request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribeParams {
    /// The URI of the resource to subscribe to.
    pub uri: String,
}

/// Request to unsubscribe from resource updates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeRequest {
    /// Method is always "resources/unsubscribe"
    pub method: String,

    /// Request parameters
    pub params: UnsubscribeParams,
}

impl Request for UnsubscribeRequest {
    const METHOD: &'static str = "resources/unsubscribe";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl UnsubscribeRequest {
    /// Create a new unsubscribe request
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: UnsubscribeParams {
                uri: uri.into(),
            },
        }
    }
}

/// Parameters for unsubscribe request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeParams {
    /// The URI of the resource to unsubscribe from.
    pub uri: String,
}

/// Notification that a resource has been updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUpdatedNotification {
    /// Method is always "notifications/resources/updated"
    pub method: String,

    /// Notification parameters
    pub params: ResourceUpdatedParams,
}

impl Notification for ResourceUpdatedNotification {
    const METHOD: &'static str = "notifications/resources/updated";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

impl ResourceUpdatedNotification {
    /// Create a new resource updated notification
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            method: "notifications/resources/updated".to_string(),
            params: ResourceUpdatedParams {
                uri: uri.into(),
            },
        }
    }
}

/// Parameters for resource updated notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceUpdatedParams {
    /// The URI of the resource that has been updated.
    pub uri: String,
}

/// Notification that the list of resources has changed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceListChangedNotification {
    /// Method is always "notifications/resources/list_changed"
    pub method: String,

    /// Optional notification parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<super::messages::NotificationParams>,
}

impl ResourceListChangedNotification {
    /// Create a new resource list changed notification
    pub fn new() -> Self {
        Self {
            method: "notifications/resources/list_changed".to_string(),
            params: None,
        }
    }
}

/// The contents of a resource, embedded into a prompt or tool call result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedResource {
    /// Type is always "resource"
    pub r#type: String,

    /// The resource content
    pub resource: ResourceContent,

    /// Optional annotations for the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Annotations>,
}

impl EmbeddedResource {
    /// Create a new embedded resource with text content
    pub fn text(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            r#type: "resource".to_string(),
            resource: ResourceContent::Text(TextResourceContents::new(uri, text)),
            annotations: None,
        }
    }

    /// Create a new embedded resource with binary content
    pub fn blob(uri: impl Into<String>, blob: impl Into<String>) -> Self {
        Self {
            r#type: "resource".to_string(),
            resource: ResourceContent::Blob(BlobResourceContents::new(uri, blob)),
            annotations: None,
        }
    }

    /// Set the annotations for the embedded resource
    pub fn with_annotations(mut self, annotations: Annotations) -> Self {
        self.annotations = Some(annotations);
        self
    }
}

/// A reference to a resource or resource template definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceReference {
    /// Type is always "ref/resource"
    pub r#type: String,

    /// The URI or URI template of the resource.
    pub uri: String,
}

impl ResourceReference {
    /// Create a new resource reference
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            r#type: "ref/resource".to_string(),
            uri: uri.into(),
        }
    }
}
