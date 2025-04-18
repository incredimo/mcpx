//! JSON-RPC message types for the Model Context Protocol

use serde::{Deserialize, Serialize};

use super::RequestId;
use crate::protocol::messages::{Request, Notification, Result as MessageResult};

/// JSON-RPC version used by the MCP protocol
pub const JSONRPC_VERSION: &str = "2.0";

/// Standard JSON-RPC error codes
pub mod error_codes {
    /// Invalid JSON was received by the server.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Custom MCP error codes can start at this value
    pub const MCP_ERROR_START: i32 = -32000;
}

/// A JSON-RPC message that can be sent or received.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JSONRPCMessage {
    /// A request that expects a response
    Request(JSONRPCRequest),
    /// A notification that does not expect a response
    Notification(JSONRPCNotification),
    /// A successful response to a request
    Response(JSONRPCResponse),
    /// An error response to a request
    Error(JSONRPCError),
    /// A batch of requests and/or notifications
    BatchRequest(Vec<JSONRPCBatchRequestItem>),
    /// A batch of responses and/or errors
    BatchResponse(Vec<JSONRPCBatchResponseItem>),
}

/// A JSON-RPC request that expects a response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JSONRPCRequest {
    /// Always "2.0"
    pub jsonrpc: String,
    /// Request ID
    pub id: RequestId,
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JSONRPCRequest {
    /// Create a new JSON-RPC request
    pub fn new<I: Into<RequestId>>(id: I, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }

    /// Create a new JSON-RPC request from a Request object
    pub fn from_request<I: Into<RequestId>, T: Request>(id: I, request: &T) -> Result<Self, serde_json::Error> {
        let params = if let Some(params) = request.params() {
            Some(serde_json::to_value(params)?)
        } else {
            None
        };

        Ok(Self::new(id, request.method().to_string(), params))
    }
}

/// A JSON-RPC notification that does not expect a response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JSONRPCNotification {
    /// Always "2.0"
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Optional parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JSONRPCNotification {
    /// Create a new JSON-RPC notification
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method: method.into(),
            params,
        }
    }

    /// Create a new JSON-RPC notification from a Notification object
    pub fn from_notification<T: Notification>(notification: &T) -> Result<Self, serde_json::Error> {
        let params = if let Some(params) = notification.params() {
            Some(serde_json::to_value(params)?)
        } else {
            None
        };

        Ok(Self::new(notification.method().to_string(), params))
    }
}

/// A successful JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JSONRPCResponse {
    /// Always "2.0"
    pub jsonrpc: String,
    /// Request ID that this is responding to
    pub id: RequestId,
    /// Result value
    pub result: serde_json::Value,
}

impl JSONRPCResponse {
    /// Create a new JSON-RPC response
    pub fn new<I: Into<RequestId>>(id: I, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: id.into(),
            result,
        }
    }

    /// Create a new JSON-RPC response from a MessageResult object
    pub fn from_result<I: Into<RequestId>>(id: I, result: &MessageResult) -> Result<Self, serde_json::Error> {
        Ok(Self::new(id, serde_json::to_value(result)?))
    }
}

/// A JSON-RPC error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JSONRPCError {
    /// Always "2.0"
    pub jsonrpc: String,
    /// Request ID that this is responding to
    pub id: RequestId,
    /// Error information
    pub error: JSONRPCErrorInfo,
}

impl JSONRPCError {
    /// Create a new JSON-RPC error
    pub fn new<I: Into<RequestId>>(id: I, code: i32, message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id: id.into(),
            error: JSONRPCErrorInfo {
                code,
                message: message.into(),
                data,
            },
        }
    }
}

/// JSON-RPC error information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JSONRPCErrorInfo {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// An item in a JSON-RPC batch request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JSONRPCBatchRequestItem {
    /// A request that expects a response
    Request(JSONRPCRequest),
    /// A notification that does not expect a response
    Notification(JSONRPCNotification),
}

/// An item in a JSON-RPC batch response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JSONRPCBatchResponseItem {
    /// A successful response
    Response(JSONRPCResponse),
    /// An error response
    Error(JSONRPCError),
}

/// A batch of JSON-RPC requests or notifications
pub type JSONRPCBatchRequest = Vec<JSONRPCBatchRequestItem>;

/// A batch of JSON-RPC responses or errors
pub type JSONRPCBatchResponse = Vec<JSONRPCBatchResponseItem>;
