//! Client message handler implementation

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::protocol::{
    RequestId, ProgressToken,
    JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification,
    roots::ListRootsRequest,
    logging::LoggingLevel,
};

use super::{ClientEvent, ClientOptions, ServerCapabilities, state::ClientState, state::PendingRequest};
use dashmap::DashMap;

/// Handles incoming messages for a client
pub(crate) struct ClientMessageHandler {
    /// Client state
    state: Arc<RwLock<ClientState>>,
    /// Pending requests
    pending_requests: Arc<DashMap<RequestId, PendingRequest>>,
    /// Event sender
    event_sender: mpsc::Sender<ClientEvent>,
    /// Server capabilities
    server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Client options
    options: ClientOptions,
}

impl ClientMessageHandler {
    /// Create a new client message handler
    pub fn new(
        state: Arc<RwLock<ClientState>>,
        pending_requests: Arc<DashMap<RequestId, PendingRequest>>,
        event_sender: mpsc::Sender<ClientEvent>,
        server_capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
        options: ClientOptions,
    ) -> Self {
        Self {
            state,
            pending_requests,
            event_sender,
            server_capabilities,
            options,
        }
    }

    /// Handle an incoming message from the server
    pub async fn handle_message(&self, message: JSONRPCMessage) -> Result<(), Error> {
        match message {
            JSONRPCMessage::Request(request) => self.handle_request(request).await,
            JSONRPCMessage::Response(response) => self.handle_response(response).await,
            JSONRPCMessage::Error(error) => self.handle_error(error).await,
            JSONRPCMessage::Notification(notification) => {
                self.handle_notification(notification).await
            }
            _ => Err(Error::ProtocolError("Unsupported message type".to_string())),
        }
    }

    /// Handle a request from the server
    async fn handle_request(&self, request: JSONRPCRequest) -> Result<(), Error> {
        debug!("Handling request: {}", request.method);

        match request.method.as_str() {
            "ping" => {
                // Handle ping request - just respond with empty result
                // We don't need to forward this to the application
                self.respond_success(request.id, serde_json::json!({})).await
            }
            "roots/list" => {
                // Roots list request - need to emit an event for the application to handle
                if !self.options.capabilities.roots {
                    return self.respond_error(
                        request.id,
                        -32601, // Method not found
                        "Client does not support roots".to_string(),
                        None,
                    ).await;
                }

                // Forward to application via event
                self.event_sender
                    .send(ClientEvent::RootsChanged)
                    .await
                    .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

                // If auto-acknowledge is enabled, send empty roots list
                if self.options.auto_acknowledge_roots_changed {
                    self.respond_success(
                        request.id,
                        serde_json::json!({ "roots": [] }),
                    ).await?;
                }

                Ok(())
            }
            "sampling/createMessage" => {
                // Sampling request - need to emit an event for the application to handle
                if !self.options.capabilities.sampling {
                    return self.respond_error(
                        request.id,
                        -32601, // Method not found
                        "Client does not support sampling".to_string(),
                        None,
                    ).await;
                }

                // This is handled by the application, which should respond directly
                // For now, return error
                self.respond_error(
                    request.id,
                    -32601, // Method not found
                    "Sampling not implemented".to_string(),
                    None,
                ).await
            }
            _ => {
                // Unknown request
                self.respond_error(
                    request.id,
                    -32601, // Method not found
                    format!("Method not found: {}", request.method),
                    None,
                ).await
            }
        }
    }

    /// Handle a response from the server
    async fn handle_response(&self, response: JSONRPCResponse) -> Result<(), Error> {
        debug!("Handling response for request: {:?}", response.id);

        // Find the pending request
        if let Some((_, pending)) = self.pending_requests.remove(&response.id) {
            // Send the response back to the waiting call
            if let Err(_) = pending.sender.send(JSONRPCMessage::Response(response)) {
                error!("Failed to send response to requester");
                return Err(Error::InternalError(
                    "Failed to send response to requester".to_string(),
                ));
            }
            Ok(())
        } else {
            warn!("Received response for unknown request: {:?}", response.id);
            Err(Error::ProtocolError(format!(
                "Received response for unknown request: {:?}",
                response.id
            )))
        }
    }

    /// Handle an error from the server
    async fn handle_error(&self, error: JSONRPCError) -> Result<(), Error> {
        debug!("Handling error for request: {:?}", error.id);

        // Find the pending request
        if let Some((_, pending)) = self.pending_requests.remove(&error.id) {
            // Send the error back to the waiting call
            if let Err(_) = pending.sender.send(JSONRPCMessage::Error(error)) {
                error!("Failed to send error to requester");
                return Err(Error::InternalError(
                    "Failed to send error to requester".to_string(),
                ));
            }
            Ok(())
        } else {
            warn!("Received error for unknown request: {:?}", error.id);
            Err(Error::ProtocolError(format!(
                "Received error for unknown request: {:?}",
                error.id
            )))
        }
    }

    /// Handle a notification from the server
    async fn handle_notification(&self, notification: JSONRPCNotification) -> Result<(), Error> {
        debug!("Handling notification: {}", notification.method);

        match notification.method.as_str() {
            "notifications/cancelled" => {
                // Handle cancellation notification
                self.handle_cancelled_notification(notification).await
            }
            "notifications/progress" => {
                // Handle progress notification
                self.handle_progress_notification(notification).await
            }
            "notifications/resources/list_changed" => {
                // Handle resources list changed notification
                self.handle_resources_changed_notification().await
            }
            "notifications/resources/updated" => {
                // Handle resource updated notification
                self.handle_resource_updated_notification(notification).await
            }
            "notifications/prompts/list_changed" => {
                // Handle prompts list changed notification
                self.handle_prompts_changed_notification().await
            }
            "notifications/tools/list_changed" => {
                // Handle tools list changed notification
                self.handle_tools_changed_notification().await
            }
            "notifications/message" => {
                // Handle logging message notification
                self.handle_logging_notification(notification).await
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

    /// Handle a cancelled notification
    async fn handle_cancelled_notification(
        &self,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        let params = notification
            .params
            .ok_or_else(|| Error::ProtocolError("Missing params in cancelled notification".to_string()))?;

        let request_id: RequestId = serde_json::from_value(params["requestId"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        // Find and remove the pending request
        if let Some((_, _)) = self.pending_requests.remove(&request_id) {
            debug!("Request cancelled: {:?}", request_id);
            // We don't need to notify the requester, as the server just won't send a response
            Ok(())
        } else {
            warn!("Received cancellation for unknown request: {:?}", request_id);
            Err(Error::ProtocolError(format!(
                "Received cancellation for unknown request: {:?}",
                request_id
            )))
        }
    }

    /// Handle a progress notification
    async fn handle_progress_notification(
        &self,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        let params = notification
            .params
            .ok_or_else(|| Error::ProtocolError("Missing params in progress notification".to_string()))?;

        let token: ProgressToken = serde_json::from_value(params["progressToken"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        let progress = params["progress"]
            .as_f64()
            .ok_or_else(|| Error::ParseError("Missing or invalid progress".to_string()))?;

        let total = params["total"].as_f64();

        let message = params["message"].as_str().map(|s| s.to_string());

        // We need to map the progress token to a request ID
        // For now, we'll use the token directly as the request ID
        let request_id = match &token {
            ProgressToken::String(s) => RequestId::String(s.clone()),
            ProgressToken::Integer(i) => RequestId::Integer(*i),
        };

        // Emit progress event
        self.event_sender
            .send(ClientEvent::Progress {
                request_id,
                token,
                progress,
                total,
                message,
            })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Handle a resources list changed notification
    async fn handle_resources_changed_notification(&self) -> Result<(), Error> {
        // Check if server supports resources list changed notifications
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.resources || !caps.resources_list_changed {
                return Err(Error::ProtocolError(
                    "Server sent resources/list_changed but doesn't support it".to_string(),
                ));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        // Emit resources changed event
        self.event_sender
            .send(ClientEvent::ResourcesChanged)
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Handle a resource updated notification
    async fn handle_resource_updated_notification(
        &self,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        let params = notification
            .params
            .ok_or_else(|| Error::ProtocolError("Missing params in resource updated notification".to_string()))?;

        let uri = params["uri"]
            .as_str()
            .ok_or_else(|| Error::ParseError("Missing or invalid uri".to_string()))?
            .to_string();

        // Emit resource updated event
        self.event_sender
            .send(ClientEvent::ResourceUpdated { uri })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Handle a prompts list changed notification
    async fn handle_prompts_changed_notification(&self) -> Result<(), Error> {
        // Check if server supports prompts list changed notifications
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.prompts || !caps.prompts_list_changed {
                return Err(Error::ProtocolError(
                    "Server sent prompts/list_changed but doesn't support it".to_string(),
                ));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        // Emit prompts changed event
        self.event_sender
            .send(ClientEvent::PromptsChanged)
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Handle a tools list changed notification
    async fn handle_tools_changed_notification(&self) -> Result<(), Error> {
        // Check if server supports tools list changed notifications
        let caps = self.server_capabilities.read().await;
        if let Some(caps) = &*caps {
            if !caps.tools || !caps.tools_list_changed {
                return Err(Error::ProtocolError(
                    "Server sent tools/list_changed but doesn't support it".to_string(),
                ));
            }
        } else {
            return Err(Error::NotInitialized);
        }

        // Emit tools changed event
        self.event_sender
            .send(ClientEvent::ToolsChanged)
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Handle a logging notification
    async fn handle_logging_notification(
        &self,
        notification: JSONRPCNotification,
    ) -> Result<(), Error> {
        let params = notification
            .params
            .ok_or_else(|| Error::ProtocolError("Missing params in logging notification".to_string()))?;

        let level: LoggingLevel = serde_json::from_value(params["level"].clone())
            .map_err(|e| Error::ParseError(e.to_string()))?;

        let logger = params["logger"].as_str().map(|s| s.to_string());

        let data = params["data"].clone();

        // Emit log message event
        self.event_sender
            .send(ClientEvent::LogMessage {
                level,
                logger,
                data,
            })
            .await
            .map_err(|_| Error::InternalError("Failed to send event".to_string()))?;

        Ok(())
    }

    /// Send a success response
    async fn respond_success(
        &self,
        id: RequestId,
        result: serde_json::Value,
    ) -> Result<(), Error> {
        // We would normally send a response through the transport
        // For now, just emit an error that this is not implemented
        Err(Error::InternalError("Response sending not implemented".to_string()))
    }

    /// Send an error response
    async fn respond_error(
        &self,
        id: RequestId,
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    ) -> Result<(), Error> {
        // We would normally send a response through the transport
        // For now, just emit an error that this is not implemented
        Err(Error::InternalError("Error sending not implemented".to_string()))
    }
}
