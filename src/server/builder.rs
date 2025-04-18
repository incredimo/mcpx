//! Builder for configuring and creating MCP servers

use std::collections::HashMap;
use crate::error::Error;
use super::{Server, ServerOptions, ServerCapabilities, ServerService};

/// Builder for creating and configuring MCP servers
pub struct ServerBuilder {
    /// Server options
    options: ServerOptions,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self {
            options: ServerOptions::default(),
        }
    }
}

impl ServerBuilder {
    /// Create a new server builder with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the server implementation name and version
    pub fn with_implementation(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.options.implementation.name = name.into();
        self.options.implementation.version = version.into();
        self
    }

    /// Set server instructions for clients
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.options.instructions = Some(instructions.into());
        self
    }

    /// Set whether to automatically acknowledge ping requests
    pub fn with_auto_acknowledge_ping(mut self, enable: bool) -> Self {
        self.options.auto_acknowledge_ping = enable;
        self
    }

    /// Set the default timeout for requests in milliseconds
    pub fn with_default_timeout(mut self, timeout_ms: u64) -> Self {
        self.options.default_timeout_ms = timeout_ms;
        self
    }

    /// Enable logging capability
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.options.capabilities.logging = enable;
        self
    }

    /// Enable completions capability
    pub fn with_completions(mut self, enable: bool) -> Self {
        self.options.capabilities.completions = enable;
        self
    }

    /// Enable prompts capability
    pub fn with_prompts(mut self, enable: bool) -> Self {
        self.options.capabilities.prompts = enable;
        self
    }

    /// Enable prompts list changed notifications capability
    pub fn with_prompts_list_changed(mut self, enable: bool) -> Self {
        self.options.capabilities.prompts_list_changed = enable;
        self
    }

    /// Enable resources capability
    pub fn with_resources(mut self, enable: bool) -> Self {
        self.options.capabilities.resources = enable;
        self
    }

    /// Enable resources list changed notifications capability
    pub fn with_resources_list_changed(mut self, enable: bool) -> Self {
        self.options.capabilities.resources_list_changed = enable;
        self
    }

    /// Enable resources subscribe capability
    pub fn with_resources_subscribe(mut self, enable: bool) -> Self {
        self.options.capabilities.resources_subscribe = enable;
        self
    }

    /// Enable tools capability
    pub fn with_tools(mut self, enable: bool) -> Self {
        self.options.capabilities.tools = enable;
        self
    }

    /// Enable tools list changed notifications capability
    pub fn with_tools_list_changed(mut self, enable: bool) -> Self {
        self.options.capabilities.tools_list_changed = enable;
        self
    }

    /// Add an experimental capability
    pub fn with_experimental(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.options.capabilities.experimental.insert(name.into(), value);
        self
    }

    /// Build the server with the provided service implementation
    pub fn build(
        self,
        service: Box<dyn ServerService + Send + Sync>,
    ) -> Result<(Server, tokio::sync::mpsc::Receiver<super::ServerEvent>), Error> {
        // Create server
        Ok(Server::new(self.options, service))
    }
}
