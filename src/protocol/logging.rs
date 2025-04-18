//! Protocol types and definitions for MCP logging
//!
//! This module contains types for working with logging in the MCP protocol,
//! including log levels and message notifications.

use serde::{Deserialize, Serialize};
use crate::protocol::{Request, Notification};
use crate::protocol::messages::MessageResult;

/// The severity of a log message.
///
/// These map to syslog message severities, as specified in RFC-5424:
/// https://datatracker.ietf.org/doc/html/rfc5424#section-6.2.1
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    /// Debug-level message
    Debug,
    /// Informational message
    Info,
    /// Normal but significant condition
    Notice,
    /// Warning conditions
    Warning,
    /// Error conditions
    Error,
    /// Critical conditions
    Critical,
    /// Action must be taken immediately
    Alert,
    /// System is unusable
    Emergency,
}

impl LoggingLevel {
    /// Get the numeric value of the log level (higher = more severe)
    pub fn as_severity(&self) -> u8 {
        match self {
            Self::Debug => 7,
            Self::Info => 6,
            Self::Notice => 5,
            Self::Warning => 4,
            Self::Error => 3,
            Self::Critical => 2,
            Self::Alert => 1,
            Self::Emergency => 0,
        }
    }

    /// Check if this log level is at least as severe as another level
    pub fn is_at_least_as_severe_as(&self, other: &Self) -> bool {
        self.as_severity() <= other.as_severity()
    }
}

/// A request from the client to the server, to enable or adjust logging.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLevelRequest {
    /// The method name
    pub method: String,
    /// The request parameters
    pub params: SetLevelParams,
}

/// Parameters for the set level request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetLevelParams {
    /// The level of logging that the client wants to receive from the server.
    /// The server should send all logs at this level and higher (i.e., more severe)
    /// to the client as notifications/message.
    pub level: LoggingLevel,
}

/// Notification of a log message passed from server to client.
/// If no logging/setLevel request has been sent from the client, the server
/// MAY decide which messages to send automatically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingMessageNotification {
    /// The method name
    pub method: String,
    /// The notification parameters
    pub params: LoggingMessageParams,
}

/// Parameters for the logging message notification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LoggingMessageParams {
    /// The severity of this log message.
    pub level: LoggingLevel,
    /// An optional name of the logger issuing this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// The data to be logged, such as a string message or an object.
    /// Any JSON serializable type is allowed here.
    pub data: serde_json::Value,
}

// Implementation for Request trait
impl Request for SetLevelRequest {
    const METHOD: &'static str = "logging/setLevel";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for Notification trait
impl Notification for LoggingMessageNotification {
    const METHOD: &'static str = "notifications/message";

    fn method(&self) -> &str {
        &self.method
    }

    fn params(&self) -> Option<&serde_json::Value> {
        None
    }
}

// Implementation for MessageResult
impl MessageResult for serde_json::Value {}

// Helper constructors
impl SetLevelRequest {
    /// Create a new set level request
    pub fn new(level: LoggingLevel) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: SetLevelParams { level },
        }
    }
}

impl LoggingMessageNotification {
    /// Create a new logging message notification with a string message
    pub fn new_text(level: LoggingLevel, message: impl Into<String>) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: LoggingMessageParams {
                level,
                logger: None,
                data: serde_json::Value::String(message.into()),
            },
        }
    }

    /// Create a new logging message notification with a logger name and string message
    pub fn new_text_with_logger(
        level: LoggingLevel,
        logger: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: LoggingMessageParams {
                level,
                logger: Some(logger.into()),
                data: serde_json::Value::String(message.into()),
            },
        }
    }

    /// Create a new logging message notification with structured data
    pub fn new_structured(
        level: LoggingLevel,
        data: serde_json::Value,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: LoggingMessageParams {
                level,
                logger: None,
                data,
            },
        }
    }

    /// Create a new logging message notification with a logger name and structured data
    pub fn new_structured_with_logger(
        level: LoggingLevel,
        logger: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            method: Self::METHOD.to_string(),
            params: LoggingMessageParams {
                level,
                logger: Some(logger.into()),
                data,
            },
        }
    }
}

/// Shorthand for creating debug level log messages
pub fn debug(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Debug, message)
}

/// Shorthand for creating info level log messages
pub fn info(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Info, message)
}

/// Shorthand for creating notice level log messages
pub fn notice(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Notice, message)
}

/// Shorthand for creating warning level log messages
pub fn warning(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Warning, message)
}

/// Shorthand for creating error level log messages
pub fn error(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Error, message)
}

/// Shorthand for creating critical level log messages
pub fn critical(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Critical, message)
}

/// Shorthand for creating alert level log messages
pub fn alert(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Alert, message)
}

/// Shorthand for creating emergency level log messages
pub fn emergency(message: impl Into<String>) -> LoggingMessageNotification {
    LoggingMessageNotification::new_text(LoggingLevel::Emergency, message)
}
