//! WebSocket transport implementation for MCP

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message as WsMessage,
    WebSocketStream,
    MaybeTlsStream,
};
use url::Url;
use log::{debug, error, info, warn};

use crate::error::Error;
use crate::protocol::JSONRPCMessage;

use super::Transport;

/// WebSocket transport for MCP
pub struct WebSocketTransport {
    /// WebSocket URL
    url: String,
    /// Whether the connection is active
    connected: Arc<AtomicBool>,
    /// Sender for outgoing messages
    sender: Arc<RwLock<Option<mpsc::Sender<WsMessage>>>>,
    /// Receiver for incoming messages
    receiver: Arc<RwLock<Option<mpsc::Receiver<Result<JSONRPCMessage, Error>>>>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(url: &str) -> Result<Self, Error> {
        // Validate URL
        let _ = Url::parse(url).map_err(Error::from)?;

        Ok(Self {
            url: url.to_string(),
            connected: Arc::new(AtomicBool::new(false)),
            sender: Arc::new(RwLock::new(None)),
            receiver: Arc::new(RwLock::new(None)),
        })
    }
}

#[async_trait]
impl Transport for WebSocketTransport {
    async fn connect(&self) -> Result<(), Error> {
        // Already connected?
        if self.connected.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Parse URL
        let url = Url::parse(&self.url).map_err(Error::from)?;

        // Connect to WebSocket server
        let (ws_stream, _) = connect_async(url.as_str())
            .await
            .map_err(|e| Error::TransportError(format!("WebSocket connection failed: {}", e)))?;

        info!("WebSocket connected");

        // Create channels for message passing
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<WsMessage>(100);
        let (incoming_tx, incoming_rx) = mpsc::channel::<Result<JSONRPCMessage, Error>>(100);

        // Store channels
        {
            let mut sender = self.sender.write().await;
            *sender = Some(outgoing_tx);
        }
        {
            let mut receiver = self.receiver.write().await;
            *receiver = Some(incoming_rx);
        }

        // Set connected flag
        self.connected.store(true, Ordering::SeqCst);

        // Start message handling task
        let connected = self.connected.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_websocket(ws_stream, outgoing_rx, incoming_tx, connected).await {
                error!("WebSocket error: {}", e);
            }
        });

        Ok(())
    }

    async fn disconnect(&self) -> Result<(), Error> {
        // Not connected?
        if !self.connected.load(Ordering::SeqCst) {
            return Ok(());
        }

        // Set disconnected flag to signal the handler to shut down
        self.connected.store(false, Ordering::SeqCst);

        // Send close message if sender is available
        let close_frame = WsMessage::Close(None);
        if let Some(sender) = &*self.sender.read().await {
            let _ = sender.send(close_frame).await;
        }

        // Clear channels
        {
            let mut sender = self.sender.write().await;
            *sender = None;
        }
        {
            let mut receiver = self.receiver.write().await;
            *receiver = None;
        }

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

        // Get sender
        let sender = self.sender.read().await;
        let sender = sender.as_ref().ok_or_else(|| {
            Error::ConnectionClosed("Connection not initialized".to_string())
        })?;

        // Send message
        sender
            .send(WsMessage::Text(json.into()))
            .await
            .map_err(|_| Error::ConnectionClosed("Failed to send message".to_string()))?;

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

/// Handle the WebSocket connection
async fn handle_websocket(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    outgoing_rx: mpsc::Receiver<WsMessage>,
    incoming_tx: mpsc::Sender<Result<JSONRPCMessage, Error>>,
    connected: Arc<AtomicBool>,
) -> Result<(), Error> {
    let (ws_sender, ws_receiver) = ws_stream.split();

    // Convert outgoing_rx to a stream
    let outgoing_stream = ReceiverStream::new(outgoing_rx);

    // Forward outgoing messages to WebSocket
    let outgoing_task = tokio::spawn(async move {
        let mut outgoing_stream = outgoing_stream;
        let mut ws_sender = ws_sender;

        while let Some(msg) = outgoing_stream.next().await {
            if let Err(e) = ws_sender.send(msg).await {
                error!("WebSocket send error: {}", e);
                break;
            }
        }

        // Try to close the connection
        let _ = ws_sender.close().await;
    });

    // Clone the connected flag for the incoming task
    let connected_clone = connected.clone();

    // Forward incoming WebSocket messages to incoming_tx
    let incoming_task = tokio::spawn(async move {
        let mut ws_receiver = ws_receiver;

        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    // Parse JSON message
                    let json_msg: Result<JSONRPCMessage, _> = serde_json::from_str(&text);
                    match json_msg {
                        Ok(msg) => {
                            if let Err(_) = incoming_tx.send(Ok(msg)).await {
                                // Receiver dropped, exit loop
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse JSON message: {}", e);
                            if let Err(_) = incoming_tx.send(Err(Error::ParseError(e.to_string()))).await {
                                // Receiver dropped, exit loop
                                break;
                            }
                        }
                    }
                }
                Ok(WsMessage::Binary(_)) => {
                    warn!("Received binary WebSocket message, ignoring");
                }
                Ok(WsMessage::Ping(_)) => {
                    // Handled automatically by tungstenite
                }
                Ok(WsMessage::Pong(_)) => {
                    // Handled automatically by tungstenite
                }
                Ok(WsMessage::Close(_)) => {
                    debug!("WebSocket connection closed by server");
                    break;
                }
                Ok(WsMessage::Frame(_)) => {
                    warn!("Received frame WebSocket message, ignoring");
                }
                Err(e) => {
                    error!("WebSocket receive error: {}", e);
                    if let Err(_) = incoming_tx
                        .send(Err(Error::TransportError(format!("WebSocket error: {}", e))))
                        .await
                    {
                        // Receiver dropped, exit loop
                        break;
                    }
                    break;
                }
            }
        }

        // Set disconnected flag
        connected_clone.store(false, Ordering::SeqCst);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = outgoing_task => {
            debug!("Outgoing WebSocket task completed");
        }
        _ = incoming_task => {
            debug!("Incoming WebSocket task completed");
        }
    }

    // Set disconnected flag
    connected.store(false, Ordering::SeqCst);

    Ok(())
}
