//! Transport layer implementations for MCP
//!
//! This module provides transport implementations for sending and receiving
//! MCP messages over various protocols.

mod websocket;
mod http;

pub use websocket::WebSocketTransport;
pub use http::HttpTransport;

use async_trait::async_trait;
use crate::error::Error;
use crate::protocol::JSONRPCMessage;

/// Transport interface for sending and receiving MCP messages
#[async_trait]
pub trait Transport {
    /// Connect to the server
    async fn connect(&self) -> Result<(), Error>;

    /// Disconnect from the server
    async fn disconnect(&self) -> Result<(), Error>;

    /// Send a message to the server
    async fn send(&self, message: JSONRPCMessage) -> Result<(), Error>;

    /// Receive a message from the server
    async fn receive(&self) -> Option<Result<JSONRPCMessage, Error>>;

    /// Check if the connection is still active
    async fn is_connected(&self) -> bool;
}
