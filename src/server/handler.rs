//! Server message handler implementation

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use log::{debug, warn};
use dashmap::DashMap;

use crate::error::Error;
use crate::protocol::{
    RequestId, ProgressToken,
    JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification,
    json_rpc::JSONRPCErrorInfo,
};

use super::{ServerEvent, ClientCapabilities, ServerOptions, state::{ServerState, Connection, CapabilityState}};

/// Handles incoming messages for a server
pub(crate) struct ServerMessageHandler {
    /// Server state
    state: Arc<RwLock<ServerState>>,
    /// Client connections
    connections: Arc<DashMap<String, Connection>>,
    /// Event sender
    event_sender: mpsc::Sender<ServerEvent>,
    /// Server options
    options: ServerOptions,
}

impl ServerMessageHandler {
    /// Helper function to create a JSON-RPC error response
    fn create_error_response(&self, id: RequestId, code: i32, message: &str) -> Result<JSONRPCMessage, Error> {
        Ok(JSONRPCMessage::Error(JSONRPCError {
            jsonrpc: "2.0".to_string(),
            id,
            error: JSONRPCErrorInfo {
                code,
                message: message.to_string(),
                data: None,
            },
        }))
    }
    /// Create a new server message handler
    pub fn new(
        state: Arc<RwLock<ServerState>>,
        connections: Arc<DashMap<String, Connection>>,
        event_sender: mpsc::Sender<ServerEvent>,
        options: ServerOptions,
    ) -> Self {
        Self {
            state,
            connections,
            event_sender,
            options,
        }
    }

    /// Handle an incoming message from a client
    pub async fn handle_message(
        &self,
        client_id: &str,
        message: JSONRPCMessage,
    ) -> Result<Option<JSONRPCMessage>, Error> {
        match message {
            JSONRPCMessage::Request(request) => {
                self.handle_request(client_id, request).await.map(Some)
            }
            JSONRPCMessage::Notification(notification) => {
                self.handle_notification(client_id, notification).await.map(|_| None)
            }
            _ => Err(Error::ProtocolError("Unsupported message type".to_string())),
        }
    }

    /// Handle a request from a client
    async fn handle_request(&self, client_id: &str, request: JSONRPCRequest) -> Result<JSONRPCMessage, Error> {
        debug!("Handling request: {} from client {}", request.method, client_id);

        // Check if server is in the correct state
        let state = self.state.read().await;
        if !state.is_running() {
            return Ok(JSONRPCMessage::Error(JSONRPCError {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                error: JSONRPCErrorInfo {
                    code: -32603, // Internal error
                    message: "Server is not running".to_string(),
                    data: None,
                },
            }));
        }
        drop(state);

        match request.method.as_str() {
            "initialize" => self.handle_initialize(client_id, request).await,
            "ping" => self.handle_ping(client_id, request).await,
            "resources/list" => self.handle_list_resources(client_id, request).await,
            "resources/templates/list" => self.handle_list_resource_templates(client_id, request).await,
            "resources/read" => self.handle_read_resource(client_id, request).await,
            "resources/subscribe" => self.handle_subscribe_resource(client_id, request).await,
            "resources/unsubscribe" => self.handle_unsubscribe_resource(client_id, request).await,
            "prompts/list" => self.handle_list_prompts(client_id, request).await,
            "prompts/get" => self.handle_get_prompt(client_id, request).await,
            "tools/list" => self.handle_list_tools(client_id, request).await,
            "tools/call" => self.handle_call_tool(client_id, request).await,
            "logging/setLevel" => self.handle_set_level(client_id, request).await,
            "completion/complete" => self.handle_complete(client_id, request).await,
            _ => {
                // Unknown request
                Ok(JSONRPCMessage::Error(JSONRPCError {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    error: JSONRPCErrorInfo {
                        code: -32601, // Method not found
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    },
                }))
            }
        }
    }

    /// Handle an initialization request
    async fn handle_initialize(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling initialize request from client {}", client_id);

        // Get connection
        let mut connection = self.connections.get_mut(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in initialize request".to_string())
        })?;

        let protocol_version = params["protocolVersion"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid protocolVersion".to_string()))?
            .to_string();

        // TODO: Version compatibility check

        let client_info: crate::protocol::Implementation = serde_json::from_value(
            params["clientInfo"].clone(),
        )
        .map_err(|e| Error::ParseError(e.to_string()))?;

        // Parse client capabilities
        let mut capabilities = CapabilityState::default();

        if let Some(caps) = params["capabilities"].as_object() {
            // Check for sampling capability
            if caps.contains_key("sampling") {
                capabilities.sampling = true;
            }

            // Check for roots capability
            if let Some(roots) = caps.get("roots") {
                capabilities.roots = true;

                if let Some(list_changed) = roots.get("listChanged") {
                    capabilities.roots_list_changed = list_changed.as_bool().unwrap_or(false);
                }
            }

            // Check for experimental capabilities
            if let Some(experimental) = caps.get("experimental") {
                if let Some(obj) = experimental.as_object() {
                    for (key, value) in obj {
                        capabilities.experimental.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // Update connection state
        connection.set_initialized(client_info.clone(), protocol_version.clone(), capabilities.clone());

        // Emit client connected event
        self.event_sender
            .send(ServerEvent::ClientConnected {
                client_id: client_id.to_string(),
                client_info: client_info.clone(),
                protocol_version: protocol_version.clone(),
                capabilities: ClientCapabilities {
                    roots: capabilities.roots,
                    roots_list_changed: capabilities.roots_list_changed,
                    sampling: capabilities.sampling,
                    experimental: capabilities.experimental.clone(),
                },
            })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        // Create response
        let mut capabilities = serde_json::Map::new();
        capabilities.insert("logging".to_string(), serde_json::Value::Bool(self.options.capabilities.logging));
        capabilities.insert("completions".to_string(), serde_json::Value::Bool(self.options.capabilities.completions));

        if self.options.capabilities.prompts {
            let mut prompts_cap = serde_json::Map::new();
            prompts_cap.insert("listChanged".to_string(), serde_json::Value::Bool(self.options.capabilities.prompts_list_changed));
            capabilities.insert("prompts".to_string(), serde_json::Value::Object(prompts_cap));
        } else {
            capabilities.insert("prompts".to_string(), serde_json::Value::Bool(false));
        }

        if self.options.capabilities.resources {
            let mut resources_cap = serde_json::Map::new();
            resources_cap.insert("subscribe".to_string(), serde_json::Value::Bool(self.options.capabilities.resources_subscribe));
            resources_cap.insert("listChanged".to_string(), serde_json::Value::Bool(self.options.capabilities.resources_list_changed));
            capabilities.insert("resources".to_string(), serde_json::Value::Object(resources_cap));
        } else {
            capabilities.insert("resources".to_string(), serde_json::Value::Bool(false));
        }

        if self.options.capabilities.tools {
            let mut tools_cap = serde_json::Map::new();
            tools_cap.insert("listChanged".to_string(), serde_json::Value::Bool(self.options.capabilities.tools_list_changed));
            capabilities.insert("tools".to_string(), serde_json::Value::Object(tools_cap));
        } else {
            capabilities.insert("tools".to_string(), serde_json::Value::Bool(false));
        }

        let mut result_map = serde_json::Map::new();
        result_map.insert("protocolVersion".to_string(), serde_json::Value::String(crate::protocol::LATEST_PROTOCOL_VERSION.to_string()));
        result_map.insert("serverInfo".to_string(), serde_json::to_value(&self.options.implementation).unwrap());
        result_map.insert("capabilities".to_string(), serde_json::Value::Object(capabilities));
        result_map.insert("instructions".to_string(), serde_json::to_value(&self.options.instructions).unwrap());

        let result = serde_json::Value::Object(result_map);

        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result,
        }))
    }

    /// Handle a ping request
    async fn handle_ping(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling ping request from client {}", client_id);

        // Simply respond with an empty result
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({}),
        }))
    }

    /// Handle a resources/list request
    async fn handle_list_resources(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling list resources request from client {}", client_id);

        // Check if server supports resources
        if !self.options.capabilities.resources {
            return Ok(JSONRPCMessage::Error(JSONRPCError {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                error: JSONRPCErrorInfo {
                    code: -32601, // Method not found
                    message: "Resources not supported".to_string(),
                    data: None,
                },
            }));
        }

        // TODO: Forward to service implementation

        // For now, just return empty resources list
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "resources": []
            }),
        }))
    }

    /// Handle a resources/templates/list request
    async fn handle_list_resource_templates(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling list resource templates request from client {}", client_id);

        // Check if server supports resources
        if !self.options.capabilities.resources {
            return self.create_error_response(request.id, -32601, "Resources not supported");
        }

        // TODO: Forward to service implementation

        // For now, just return empty templates list
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "resourceTemplates": []
            }),
        }))
    }

    /// Handle a resources/read request
    async fn handle_read_resource(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling read resource request from client {}", client_id);

        // Check if server supports resources
        if !self.options.capabilities.resources {
            return self.create_error_response(request.id, -32601, "Resources not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in read resource request".to_string())
        })?;

        let _uri = params["uri"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid uri".to_string()))?
            .to_string();

        // TODO: Forward to service implementation

        // For now, just return empty contents
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "contents": []
            }),
        }))
    }

    /// Handle a resources/subscribe request
    async fn handle_subscribe_resource(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling subscribe resource request from client {}", client_id);

        // Check if server supports resource subscriptions
        if !self.options.capabilities.resources || !self.options.capabilities.resources_subscribe {
            return self.create_error_response(request.id, -32601, "Resource subscriptions not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in subscribe resource request".to_string())
        })?;

        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid uri".to_string()))?
            .to_string();

        // Update connection state
        let mut connection = self.connections.get_mut(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        connection.subscribe_resource(&uri);

        // TODO: Forward to service implementation

        // Return success
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({}),
        }))
    }

    /// Handle a resources/unsubscribe request
    async fn handle_unsubscribe_resource(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling unsubscribe resource request from client {}", client_id);

        // Check if server supports resource subscriptions
        if !self.options.capabilities.resources || !self.options.capabilities.resources_subscribe {
            return self.create_error_response(request.id, -32601, "Resource subscriptions not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in unsubscribe resource request".to_string())
        })?;

        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid uri".to_string()))?
            .to_string();

        // Update connection state
        let mut connection = self.connections.get_mut(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        connection.unsubscribe_resource(&uri);

        // TODO: Forward to service implementation

        // Return success
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({}),
        }))
    }

    /// Handle a prompts/list request
    async fn handle_list_prompts(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling list prompts request from client {}", client_id);

        // Check if server supports prompts
        if !self.options.capabilities.prompts {
            return self.create_error_response(request.id, -32601, "Prompts not supported");
        }

        // TODO: Forward to service implementation

        // For now, just return empty prompts list
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "prompts": []
            }),
        }))
    }

    /// Handle a prompts/get request
    async fn handle_get_prompt(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling get prompt request from client {}", client_id);

        // Check if server supports prompts
        if !self.options.capabilities.prompts {
            return self.create_error_response(request.id, -32601, "Prompts not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in get prompt request".to_string())
        })?;

        let _name = params["name"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid name".to_string()))?
            .to_string();

        let _arguments = params.get("arguments").and_then(|v| v.as_object()).map(|o| {
            o.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect::<HashMap<String, String>>()
        });

        // TODO: Forward to service implementation

        // For now, just return empty messages
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "messages": []
            }),
        }))
    }

    /// Handle a tools/list request
    async fn handle_list_tools(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling list tools request from client {}", client_id);

        // Check if server supports tools
        if !self.options.capabilities.tools {
            return self.create_error_response(request.id, -32601, "Tools not supported");
        }

        // TODO: Forward to service implementation

        // For now, just return empty tools list
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "tools": []
            }),
        }))
    }

    /// Handle a tools/call request
    async fn handle_call_tool(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling call tool request from client {}", client_id);

        // Check if server supports tools
        if !self.options.capabilities.tools {
            return self.create_error_response(request.id, -32601, "Tools not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in call tool request".to_string())
        })?;

        let _name = params["name"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid name".to_string()))?
            .to_string();

        let _arguments = params.get("arguments").cloned();

        // TODO: Forward to service implementation

        // For now, just return empty content
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "content": []
            }),
        }))
    }

    /// Handle a logging/setLevel request
    async fn handle_set_level(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling set level request from client {}", client_id);

        // Check if server supports logging
        if !self.options.capabilities.logging {
            return self.create_error_response(request.id, -32601, "Logging not supported");
        }

        // Parse request parameters
        let params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in set level request".to_string())
        })?;

        let _level: crate::protocol::logging::LoggingLevel = serde_json::from_value(params["level"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        // TODO: Forward to service implementation

        // Return success
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({}),
        }))
    }

    /// Handle a completion/complete request
    async fn handle_complete(
        &self,
        client_id: &str,
        request: JSONRPCRequest,
    ) -> Result<JSONRPCMessage, Error> {
        debug!("Handling complete request from client {}", client_id);

        // Check if server supports completions
        if !self.options.capabilities.completions {
            return self.create_error_response(request.id, -32601, "Completions not supported");
        }

        // Parse request parameters
        let _params = request.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in complete request".to_string())
        })?;

        // TODO: Forward to service implementation

        // For now, just return empty completions
        Ok(JSONRPCMessage::Response(JSONRPCResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: serde_json::json!({
                "completion": {
                    "values": []
                }
            }),
        }))
    }

    /// Handle a notification from a client
    async fn handle_notification(
        &self,
        client_id: &str,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        debug!("Handling notification: {} from client {}", notification.method, client_id);

        match notification.method.as_str() {
            "notifications/initialized" => {
                self.handle_initialized_notification(client_id, notification).await
            }
            "notifications/cancelled" => {
                self.handle_cancelled_notification(client_id, notification).await
            }
            "notifications/progress" => {
                self.handle_progress_notification(client_id, notification).await
            }
            "notifications/roots/list_changed" => {
                self.handle_roots_list_changed_notification(client_id, notification).await
            }
            _ => {
                warn!("Received unknown notification: {}", notification.method);
                Err(Error::ProtocolError(format!(
                    "Received unknown notification: {}",
                    notification.method
                )))
            }
        }
    }

    /// Handle an initialized notification
    async fn handle_initialized_notification(
        &self,
        client_id: &str,
        _notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        debug!("Handling initialized notification from client {}", client_id);

        // Get connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if connection is initialized
        if !connection.initialized {
            return Err(Error::ProtocolError("Client not initialized".to_string()));
        }

        // Initialization complete
        debug!("Client {} initialization complete", client_id);

        Ok(())
    }

    /// Handle a cancelled notification
    async fn handle_cancelled_notification(
        &self,
        client_id: &str,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        debug!("Handling cancelled notification from client {}", client_id);

        // Parse notification parameters
        let params = notification.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in cancelled notification".to_string())
        })?;

        let request_id: RequestId = serde_json::from_value(params["requestId"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        let reason = params["reason"].as_str().map(|s| s.to_string());

        // TODO: Forward to service implementation
        debug!(
            "Client {} cancelled request {:?}{}",
            client_id,
            request_id,
            reason.map_or("".to_string(), |r| format!(": {}", r))
        );

        Ok(())
    }

    /// Handle a progress notification
    async fn handle_progress_notification(
        &self,
        client_id: &str,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        debug!("Handling progress notification from client {}", client_id);

        // Parse notification parameters
        let params = notification.params.ok_or_else(|| {
            Error::ProtocolError("Missing params in progress notification".to_string())
        })?;

        let token: ProgressToken = serde_json::from_value(params["progressToken"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        let progress = params["progress"]
            .as_f64()
            .ok_or_else(|| Error::ParseError("Missing or invalid progress".to_string()))?;

        let total = params["total"].as_f64();

        let message = params["message"].as_str().map(|s| s.to_string());

        // TODO: Forward to service implementation
        debug!(
            "Client {} progress for token {:?}: {}/{} {}",
            client_id,
            token,
            progress,
            total.map_or("?".to_string(), |t| t.to_string()),
            message.unwrap_or_default()
        );

        Ok(())
    }

    /// Handle a roots list changed notification
    async fn handle_roots_list_changed_notification(
        &self,
        client_id: &str,
        _notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        debug!("Handling roots list changed notification from client {}", client_id);

        // Get connection
        let connection = self.connections.get(client_id).ok_or_else(|| {
            Error::InternalError(format!("Unknown client: {}", client_id))
        })?;

        // Check if client supports roots list changed notifications
        if !connection.capabilities.roots_list_changed {
            return Err(Error::ProtocolError(
                "Client sent roots/list_changed but doesn't support it".to_string(),
            ));
        }

        // Emit roots updated event
        self.event_sender
            .send(ServerEvent::RootsUpdated {
                client_id: client_id.to_string(),
            })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }
}
