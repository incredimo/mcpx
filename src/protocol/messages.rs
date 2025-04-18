//! Basic message types for the MCP protocol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ProgressToken;

/// A request that expects a response.
pub trait Request {
    /// Method name constant
    const METHOD: &'static str;

    /// Get the method name
    fn method(&self) -> &str;

    /// Get the parameters
    fn params(&self) -> Option<&serde_json::Value>;
}

/// A concrete request implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestImpl {
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Request for RequestImpl {
    const METHOD: &'static str = "";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        self.params.as_ref()
    }
}

impl RequestImpl {
    /// Create a new request
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            params: None,
        }
    }

    /// Create a new request with parameters
    pub fn with_params(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            method: method.into(),
            params: Some(params),
        }
    }
}

impl RequestImpl {
    /// Create a new request with a progress token
    pub fn with_progress_token(method: impl Into<String>, progress_token: ProgressToken) -> Self {
        let mut params = serde_json::Map::new();
        params.insert("progressToken".to_string(), serde_json::to_value(progress_token).unwrap());
        Self {
            method: method.into(),
            params: Some(serde_json::Value::Object(params)),
        }
    }
}

/// A notification that does not expect a response.
pub trait Notification {
    /// Method name constant
    const METHOD: &'static str;

    /// Get the method name
    fn method(&self) -> &str;

    /// Get the parameters
    fn params(&self) -> Option<&serde_json::Value>;
}

/// A concrete notification implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationImpl {
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Notification for NotificationImpl {
    const METHOD: &'static str = "";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        self.params.as_ref()
    }
}

impl NotificationImpl {
    /// Create a new notification
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            params: None,
        }
    }

    /// Create a new notification with parameters
    pub fn with_params(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            method: method.into(),
            params: Some(params),
        }
    }
}

/// Parameters for a request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestParams {
    /// Metadata for the request
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
    /// Additional parameters (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl RequestParams {
    /// Create empty request parameters
    pub fn new() -> Self {
        Self {
            meta: None,
            additional: HashMap::new(),
        }
    }

    /// Add a progress token to the request parameters
    pub fn set_progress_token(&mut self, token: ProgressToken) {
        if self.meta.is_none() {
            self.meta = Some(RequestMeta {
                progress_token: Some(token),
            });
        } else if let Some(meta) = &mut self.meta {
            meta.progress_token = Some(token);
        }
    }

    /// Add a parameter to the request
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for RequestParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for a request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestMeta {
    /// Progress token for the request
    #[serde(rename = "progressToken", skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,
}

/// Parameters for a notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationParams {
    /// Metadata for the notification
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<NotificationMeta>,
    /// Additional parameters (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl NotificationParams {
    /// Create empty notification parameters
    pub fn new() -> Self {
        Self {
            meta: None,
            additional: HashMap::new(),
        }
    }

    /// Add a parameter to the notification
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for NotificationParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for a notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationMeta {
    /// Additional metadata (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl NotificationMeta {
    /// Create empty notification metadata
    pub fn new() -> Self {
        Self {
            additional: HashMap::new(),
        }
    }

    /// Add a metadata field to the notification
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for NotificationMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// A result returned by a request handler
pub trait MessageResult {}

/// A concrete result implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Result {
    /// Metadata for the result
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResultMeta>,
    /// Additional fields (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl MessageResult for Result {}

impl Result {
    /// Create an empty result
    pub fn new() -> Self {
        Self {
            meta: None,
            additional: HashMap::new(),
        }
    }

    /// Add a field to the result
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for Result {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata for a result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultMeta {
    /// Additional metadata (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl ResultMeta {
    /// Create empty result metadata
    pub fn new() -> Self {
        Self {
            additional: HashMap::new(),
        }
    }

    /// Add a metadata field to the result
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for ResultMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// An empty result with no additional fields
pub type EmptyResult = Result;

/// A paginated request with a cursor parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedRequest {
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<PaginatedRequestParams>,
}

impl PaginatedRequest {
    /// Create a new paginated request
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            params: None,
        }
    }

    /// Create a new paginated request with a cursor
    pub fn with_cursor(method: impl Into<String>, cursor: impl Into<String>) -> Self {
        let params = PaginatedRequestParams {
            cursor: Some(cursor.into()),
            meta: None,
            additional: HashMap::new(),
        };

        Self {
            method: method.into(),
            params: Some(params),
        }
    }
}

/// Parameters for a paginated request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedRequestParams {
    /// Cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Metadata for the request
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestMeta>,
    /// Additional parameters (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// A paginated result with a next cursor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResult {
    /// Cursor for the next page, if any
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Metadata for the result
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResultMeta>,
    /// Additional fields (method-specific)
    #[serde(flatten)]
    pub additional: HashMap<String, serde_json::Value>,
}

impl PaginatedResult {
    /// Create a new paginated result
    pub fn new() -> Self {
        Self {
            next_cursor: None,
            meta: None,
            additional: HashMap::new(),
        }
    }

    /// Create a new paginated result with a next cursor
    pub fn with_next_cursor(next_cursor: impl Into<String>) -> Self {
        Self {
            next_cursor: Some(next_cursor.into()),
            meta: None,
            additional: HashMap::new(),
        }
    }

    /// Add a field to the result
    pub fn add<T: Serialize>(&mut self, name: impl Into<String>, value: &T) -> std::result::Result<(), serde_json::Error> {
        self.additional.insert(name.into(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Default for PaginatedResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard MCP cancel notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CancelledNotification {
    /// Method is always "notifications/cancelled"
    pub method: String,
    /// Parameters for the cancellation
    pub params: CancelledParams,
}

/// Parameters for a cancel notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CancelledParams {
    /// The ID of the request to cancel
    #[serde(rename = "requestId")]
    pub request_id: super::RequestId,
    /// Optional reason for cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl CancelledNotification {
    /// Create a new cancelled notification
    pub fn new<I: Into<super::RequestId>>(request_id: I) -> Self {
        Self {
            method: "notifications/cancelled".to_string(),
            params: CancelledParams {
                request_id: request_id.into(),
                reason: None,
            },
        }
    }

    /// Create a new cancelled notification with a reason
    pub fn with_reason<I: Into<super::RequestId>>(request_id: I, reason: impl Into<String>) -> Self {
        Self {
            method: "notifications/cancelled".to_string(),
            params: CancelledParams {
                request_id: request_id.into(),
                reason: Some(reason.into()),
            },
        }
    }
}

/// Standard MCP progress notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressNotification {
    /// Method is always "notifications/progress"
    pub method: String,
    /// Parameters for the progress notification
    pub params: ProgressParams,
}

/// Parameters for a progress notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressParams {
    /// Progress token from the original request
    #[serde(rename = "progressToken")]
    pub progress_token: ProgressToken,
    /// Current progress value
    pub progress: f64,
    /// Total progress value, if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// Optional progress message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl ProgressNotification {
    /// Create a new progress notification
    pub fn new(progress_token: ProgressToken, progress: f64) -> Self {
        Self {
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token,
                progress,
                total: None,
                message: None,
            },
        }
    }

    /// Create a new progress notification with a total
    pub fn with_total(progress_token: ProgressToken, progress: f64, total: f64) -> Self {
        Self {
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token,
                progress,
                total: Some(total),
                message: None,
            },
        }
    }

    /// Create a new progress notification with a message
    pub fn with_message(progress_token: ProgressToken, progress: f64, message: impl Into<String>) -> Self {
        Self {
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token,
                progress,
                total: None,
                message: Some(message.into()),
            },
        }
    }

    /// Create a new progress notification with a total and message
    pub fn with_total_and_message(
        progress_token: ProgressToken,
        progress: f64,
        total: f64,
        message: impl Into<String>,
    ) -> Self {
        Self {
            method: "notifications/progress".to_string(),
            params: ProgressParams {
                progress_token,
                progress,
                total: Some(total),
                message: Some(message.into()),
            },
        }
    }
}
