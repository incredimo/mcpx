//! Error types for the MCP SDK

use thiserror::Error;

/// Errors that can occur in the MCP SDK
#[derive(Error, Debug, Clone)]
pub enum Error {
    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Server error
    #[error("Server error {0}: {1}")]
    ServerError(i32, String, Option<serde_json::Value>),

    /// Request timeout
    #[error("Timeout: {0}")]
    Timeout(String),

    /// The requested feature is not supported
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    /// The client or server is not initialized
    #[error("Not initialized")]
    NotInitialized,

    /// The connection was closed
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    UrlError(String),
}

/// Result type using our Error type
pub type Result<T> = std::result::Result<T, Error>;

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::UrlError(err.to_string())
    }
}
