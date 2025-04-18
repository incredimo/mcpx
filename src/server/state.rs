//! Server state management

use std::collections::HashMap;
use crate::protocol::Implementation;

/// Current server operational state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationalState {
    /// Server is stopped
    Stopped,
    /// Server is starting up
    Starting,
    /// Server is running
    Running,
    /// Server is shutting down
    Stopping,
}

/// Server state
pub(crate) struct ServerState {
    /// Current operational state
    operational_state: OperationalState,
}

/// Connection state for tracking client capabilities
#[derive(Debug, Clone)]
pub(crate) struct CapabilityState {
    /// Whether this connection supports logging
    pub logging: bool,
    /// Whether this connection supports sampling
    pub sampling: bool,
    /// Whether this connection supports roots
    pub roots: bool,
    /// Whether this connection supports roots list changed notifications
    pub roots_list_changed: bool,
    /// Whether this connection supports prompts list changed notifications
    pub prompts_list_changed: bool,
    /// Whether this connection supports resources list changed notifications
    pub resources_list_changed: bool,
    /// Whether this connection supports resource subscriptions
    pub resources_subscribe: bool,
    /// Whether this connection supports tools list changed notifications
    pub tools_list_changed: bool,
    /// Experimental capabilities
    pub experimental: HashMap<String, serde_json::Value>,
}

impl Default for CapabilityState {
    fn default() -> Self {
        Self {
            logging: false,
            sampling: false,
            roots: false,
            roots_list_changed: false,
            prompts_list_changed: false,
            resources_list_changed: false,
            resources_subscribe: false,
            tools_list_changed: false,
            experimental: HashMap::new(),
        }
    }
}

/// Client connection state
#[derive(Debug)]
pub(crate) struct Connection {
    /// Unique client identifier
    pub id: String,
    /// Is the connection initialized?
    pub initialized: bool,
    /// Client implementation info (available after initialization)
    pub client_info: Option<Implementation>,
    /// Client protocol version (available after initialization)
    pub protocol_version: Option<String>,
    /// Client capabilities (available after initialization)
    pub capabilities: CapabilityState,
    /// Resources this client is subscribed to
    pub subscribed_resources: Vec<String>,
}

impl Connection {
    /// Create a new connection
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            initialized: false,
            client_info: None,
            protocol_version: None,
            capabilities: CapabilityState::default(),
            subscribed_resources: Vec::new(),
        }
    }

    /// Set the connection as initialized
    pub fn set_initialized(
        &mut self,
        client_info: Implementation,
        protocol_version: String,
        capabilities: CapabilityState,
    ) {
        self.initialized = true;
        self.client_info = Some(client_info);
        self.protocol_version = Some(protocol_version);
        self.capabilities = capabilities;
    }

    /// Subscribe to a resource
    pub fn subscribe_resource(&mut self, uri: &str) {
        if !self.subscribed_resources.contains(&uri.to_string()) {
            self.subscribed_resources.push(uri.to_string());
        }
    }

    /// Unsubscribe from a resource
    pub fn unsubscribe_resource(&mut self, uri: &str) {
        self.subscribed_resources.retain(|r| r != uri);
    }

    /// Check if subscribed to a resource
    pub fn is_subscribed_to(&self, uri: &str) -> bool {
        self.subscribed_resources.contains(&uri.to_string())
    }
}

impl ServerState {
    /// Create a new server state
    pub fn new() -> Self {
        Self {
            operational_state: OperationalState::Stopped,
        }
    }

    /// Get the current operational state
    pub fn operational_state(&self) -> OperationalState {
        self.operational_state
    }

    /// Set the server state to starting
    pub fn set_starting(&mut self) {
        self.operational_state = OperationalState::Starting;
    }

    /// Set the server state to running
    pub fn set_running(&mut self) {
        self.operational_state = OperationalState::Running;
    }

    /// Set the server state to stopping
    pub fn set_stopping(&mut self) {
        self.operational_state = OperationalState::Stopping;
    }

    /// Set the server state to stopped
    pub fn set_stopped(&mut self) {
        self.operational_state = OperationalState::Stopped;
    }

    /// Check if the server is stopped
    pub fn is_stopped(&self) -> bool {
        self.operational_state == OperationalState::Stopped
    }

    /// Check if the server is starting
    pub fn is_starting(&self) -> bool {
        self.operational_state == OperationalState::Starting
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.operational_state == OperationalState::Running
    }

    /// Check if the server is stopping
    pub fn is_stopping(&self) -> bool {
        self.operational_state == OperationalState::Stopping
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
