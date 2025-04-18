//! Builder for configuring and creating MCP clients


use crate::error::Error;
use crate::transport::{Transport, WebSocketTransport, HttpTransport};
use super::{Client, ClientOptions, ClientCapabilities};

/// Builder for creating and configuring MCP clients
pub struct ClientBuilder {
    /// Client options
    options: ClientOptions,
    /// WebSocket URL (for WebSocketTransport)
    websocket_url: Option<String>,
    /// HTTP URL (for HttpTransport)
    http_url: Option<String>,
    /// Custom transport implementation
    transport: Option<Box<dyn Transport + Send + Sync>>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            options: ClientOptions::default(),
            websocket_url: None,
            http_url: None,
            transport: None,
        }
    }
}

impl ClientBuilder {
    /// Create a new client builder with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the client implementation name and version
    pub fn with_implementation(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.options.implementation.name = name.into();
        self.options.implementation.version = version.into();
        self
    }

    /// Set the client capabilities
    pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> Self {
        self.options.capabilities = capabilities;
        self
    }

    /// Enable roots listing capability
    pub fn with_roots(mut self, enable: bool) -> Self {
        self.options.capabilities.roots = enable;
        self
    }

    /// Enable roots list changed notification capability
    pub fn with_roots_list_changed(mut self, enable: bool) -> Self {
        self.options.capabilities.roots_list_changed = enable;
        self
    }

    /// Enable sampling capability
    pub fn with_sampling(mut self, enable: bool) -> Self {
        self.options.capabilities.sampling = enable;
        self
    }

    /// Add an experimental capability
    pub fn with_experimental(mut self, capability: impl Into<String>) -> Self {
        self.options.capabilities.experimental.push(capability.into());
        self
    }

    /// Set whether to automatically acknowledge roots list changed notifications
    pub fn with_auto_acknowledge_roots_changed(mut self, enable: bool) -> Self {
        self.options.auto_acknowledge_roots_changed = enable;
        self
    }

    /// Set the default timeout for requests in milliseconds
    pub fn with_default_timeout(mut self, timeout_ms: u64) -> Self {
        self.options.default_timeout_ms = timeout_ms;
        self
    }

    /// Set the WebSocket URL to connect to
    pub fn with_websocket_url(mut self, url: impl Into<String>) -> Self {
        self.websocket_url = Some(url.into());
        self.http_url = None; // Clear HTTP URL if WebSocket is set
        self
    }

    /// Set the HTTP URL to connect to
    pub fn with_http_url(mut self, url: impl Into<String>) -> Self {
        self.http_url = Some(url.into());
        self.websocket_url = None; // Clear WebSocket URL if HTTP is set
        self
    }

    /// Set a custom transport implementation
    pub fn with_transport(mut self, transport: Box<dyn Transport + Send + Sync>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<(Client, tokio::sync::mpsc::Receiver<super::ClientEvent>), Error> {
        // Create transport
        let transport: Box<dyn Transport + Send + Sync> = if let Some(transport) = self.transport {
            transport
        } else if let Some(url) = self.websocket_url {
            Box::new(WebSocketTransport::new(&url)?)
        } else if let Some(url) = self.http_url {
            Box::new(HttpTransport::new(&url)?)
        } else {
            return Err(Error::ConfigError("No transport, WebSocket URL, or HTTP URL specified".to_string()));
        };

        // Create client
        Ok(Client::new(transport, self.options))
    }
}
