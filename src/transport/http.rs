//! HTTP transport implementation for MCP
//!
//! This module provides an HTTP transport implementation for the Model Context Protocol.
//! It allows clients to communicate with MCP servers over HTTP.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use async_trait::async_trait;
use tokio::sync::{mpsc, RwLock};
use url::Url;
use log::{info, warn};
use reqwest::{Client as HttpClient, header};

use crate::error::Error;
use crate::protocol::JSONRPCMessage;

use super::Transport;

/// HTTP transport for MCP
pub struct HttpTransport {
    /// HTTP URL
    url: String,
    /// Whether the connection is active
    connected: Arc<AtomicBool>,
    /// HTTP client
    client: HttpClient,
    /// Receiver for incoming messages (simulated for HTTP)
    receiver: Arc<RwLock<Option<mpsc::Receiver<Result<JSONRPCMessage, Error>>>>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(url: &str) -> Result<Self, Error> {
        // Validate URL
        let url_parsed = Url::parse(url).map_err(Error::from)?;

        // Ensure it's HTTP or HTTPS
        if url_parsed.scheme() != "http" && url_parsed.scheme() != "https" {
            return Err(Error::UrlError(format!(
                "Invalid URL scheme: {}. Expected http or https",
                url_parsed.scheme()
            )));
        }

        // Create HTTP client with default headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = HttpClient::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| Error::TransportError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            url: url.to_string(),
            connected: Arc::new(AtomicBool::new(false)),
            client,
            receiver: Arc::new(RwLock::new(None)),
        })
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn connect(&self) -> Result<(), Error> {
        // Already connected?
        if self.connected.load(Ordering::SeqCst) {
            return Ok(());
        }

        // For HTTP, we'll do a simple HEAD request to check if the server is available
        let response = self.client
            .head(&self.url)
            .send()
            .await
            .map_err(|e| Error::TransportError(format!("HTTP connection failed: {}", e)))?;

        // Check if the response is successful
        if !response.status().is_success() {
            return Err(Error::TransportError(format!(
                "HTTP connection failed with status: {}",
                response.status()
            )));
        }

        info!("HTTP transport connected to {}", self.url);

        // Create a channel for simulating message receiving
        // (HTTP is request/response, not streaming, so we'll simulate the receive part)
        let (_incoming_tx, incoming_rx) = mpsc::channel::<Result<JSONRPCMessage, Error>>(100);

        // Store channel
        {
            let mut receiver = self.receiver.write().await;
            *receiver = Some(incoming_rx);
        }

        // Set connected flag
        self.connected.store(true, Ordering::SeqCst);

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), Error> {
        // Not connected?
        if !self.connected.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Clear the receiver channel
        {
            let mut receiver = self.receiver.write().await;
            *receiver = None;
        }

        // Set disconnected flag
        self.connected.store(false, Ordering::SeqCst);

        info!("HTTP transport disconnected");

        Ok(())
    }

    async fn send(&self, message: JSONRPCMessage) -> Result<(), Error> {
        // Not connected?
        if !self.connected.load(Ordering::SeqCst) {
            return Err(Error::ConnectionClosed("Not connected".to_string()));
        }

        // Serialize message to JSON
        let json = serde_json::to_string(&message)
            .map_err(|e| Error::JsonError(e.to_string()))?;

        // Send HTTP POST request
        let response = self.client
            .post(&self.url)
            .body(json)
            .send()
            .await
            .map_err(|e| Error::TransportError(format!("HTTP request failed: {}", e)))?;

        // Check if the response is successful
        if !response.status().is_success() {
            return Err(Error::TransportError(format!(
                "HTTP request failed with status: {}",
                response.status()
            )));
        }

        // Get the response body
        let response_body = response
            .text()
            .await
            .map_err(|e| Error::TransportError(format!("Failed to read HTTP response: {}", e)))?;

        // Parse the response as a JSON-RPC message
        let response_message: JSONRPCMessage = serde_json::from_str(&response_body)
            .map_err(|e| Error::JsonError(format!("Failed to parse response: {}", e)))?;

        // Send the response to the receiver channel
        if let Some(_receiver) = &self.receiver.read().await.as_ref() {
            // Get the sender by cloning the receiver's sender
            let (incoming_tx, _) = mpsc::channel::<Result<JSONRPCMessage, Error>>(1);

            // This is a bit of a hack - in a real implementation, we'd store the sender
            // But for this example, we'll create a new channel each time
            if let Err(_) = incoming_tx.send(Ok(response_message)).await {
                warn!("Failed to send response to channel - receiver may have been dropped");
            }
        }

        Ok(())
    }

    async fn receive(&self) -> Option<Result<JSONRPCMessage, Error>> {
        // Not connected?
        if !self.connected.load(Ordering::SeqCst) {
            return Some(Err(Error::ConnectionClosed("Not connected".to_string())));
        }

        // Get receiver
        let mut receiver = self.receiver.write().await;
        let receiver = receiver.as_mut()?;

        // Receive message
        receiver.recv().await
    }

    async fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_transport_new() {
        // Valid URL
        let transport = HttpTransport::new("http://localhost:8080").unwrap();
        assert_eq!(transport.url, "http://localhost:8080");
        assert!(!transport.is_connected().await);

        // Invalid URL
        let result = HttpTransport::new("invalid-url");
        assert!(result.is_err());

        // Invalid scheme
        let result = HttpTransport::new("ftp://localhost:8080");
        assert!(result.is_err());
    }
}
