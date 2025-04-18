//! Client state management

use std::time::Instant;
use tokio::sync::oneshot;
use crate::protocol::{JSONRPCMessage, RequestId};

/// Current client connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected to server
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Initializing protocol
    Initializing,
    /// Fully initialized and ready
    Initialized,
}

/// Client state
pub(crate) struct ClientState {
    /// Current connection state
    connection_state: ConnectionState,
}

/// Pending request information
pub(crate) struct PendingRequest {
    /// Channel to send response back to caller
    pub sender: oneshot::Sender<JSONRPCMessage>,
    /// Method name for this request
    pub method: String,
    /// Time when request was sent
    pub start_time: Instant,
}

impl ClientState {
    /// Create a new client state
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
        }
    }

    /// Get the current connection state
    pub fn connection_state(&self) -> ConnectionState {
        self.connection_state
    }

    /// Set the client state to connecting
    pub fn set_connecting(&mut self) {
        self.connection_state = ConnectionState::Connecting;
    }

    /// Set the client state to initializing
    pub fn set_initializing(&mut self) {
        self.connection_state = ConnectionState::Initializing;
    }

    /// Set the client state to initialized
    pub fn set_initialized(&mut self) {
        self.connection_state = ConnectionState::Initialized;
    }

    /// Set the client state to disconnected
    pub fn set_disconnected(&mut self) {
        self.connection_state = ConnectionState::Disconnected;
    }

    /// Check if the client is disconnected
    pub fn is_disconnected(&self) -> bool {
        self.connection_state == ConnectionState::Disconnected
    }

    /// Check if the client is connecting
    pub fn is_connecting(&self) -> bool {
        self.connection_state == ConnectionState::Connecting
    }

    /// Check if the client is initializing
    pub fn is_initializing(&self) -> bool {
        self.connection_state == ConnectionState::Initializing
    }

    /// Check if the client is initialized
    pub fn is_initialized(&self) -> bool {
        self.connection_state == ConnectionState::Initialized
    }
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}
