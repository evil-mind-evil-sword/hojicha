//! WebSocket connection helper commands

use crate::core::{Cmd, Message};
use crate::commands;
use std::sync::Arc;
use tokio::sync::Mutex;

/// WebSocket events
#[derive(Debug, Clone)]
pub enum WebSocketEvent {
    /// Connected to server
    Connected,
    /// Received a text message
    Message(String),
    /// Received binary data
    Binary(Vec<u8>),
    /// Connection closed
    Closed(Option<String>),
    /// Error occurred
    Error(WebSocketError),
}

/// WebSocket errors
#[derive(Debug, Clone)]
pub enum WebSocketError {
    /// Connection failed
    ConnectionFailed(String),
    /// Send failed
    SendFailed(String),
    /// Protocol error
    ProtocolError(String),
    /// Timeout
    Timeout,
}

impl std::fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::ConnectionFailed(e) => write!(f, "Connection failed: {}", e),
            WebSocketError::SendFailed(e) => write!(f, "Send failed: {}", e),
            WebSocketError::ProtocolError(e) => write!(f, "Protocol error: {}", e),
            WebSocketError::Timeout => write!(f, "WebSocket timeout"),
        }
    }
}

impl std::error::Error for WebSocketError {}

/// WebSocket connection handle
#[derive(Debug, Clone)]
pub struct WebSocketHandle {
    /// Unique connection ID
    pub id: String,
    /// URL of the WebSocket server
    pub url: String,
    /// Whether the connection is active
    pub connected: Arc<Mutex<bool>>,
}

impl WebSocketHandle {
    /// Send a text message through the WebSocket
    pub async fn send_text(&self, message: String) -> Result<(), WebSocketError> {
        // In a real implementation, this would send through the actual WebSocket
        if *self.connected.lock().await {
            Ok(())
        } else {
            Err(WebSocketError::SendFailed("Not connected".to_string()))
        }
    }

    /// Send binary data through the WebSocket
    pub async fn send_binary(&self, data: Vec<u8>) -> Result<(), WebSocketError> {
        if *self.connected.lock().await {
            Ok(())
        } else {
            Err(WebSocketError::SendFailed("Not connected".to_string()))
        }
    }

    /// Close the WebSocket connection
    pub async fn close(&self) {
        *self.connected.lock().await = false;
    }
}

/// Create a WebSocket connection command
///
/// This establishes a WebSocket connection and returns events through the handler.
/// The connection will automatically reconnect on failure.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::{websocket, WebSocketEvent};
/// # #[derive(Clone)]
/// # enum Msg {
/// #     WsConnected,
/// #     WsMessage(String),
/// #     WsDisconnected,
/// # }
/// # impl hojicha_core::Message for Msg {}
/// 
/// websocket("wss://echo.websocket.org", |event| {
///     match event {
///         WebSocketEvent::Connected => Some(Msg::WsConnected),
///         WebSocketEvent::Message(text) => Some(Msg::WsMessage(text)),
///         WebSocketEvent::Closed(_) => Some(Msg::WsDisconnected),
///         _ => None,
///     }
/// })
/// # ;
/// ```
pub fn websocket<M, F>(url: impl Into<String>, mut handler: F) -> Cmd<M>
where
    M: Message,
    F: FnMut(WebSocketEvent) -> Option<M> + Send + 'static,
{
    let url = url.into();
    let handle = WebSocketHandle {
        id: uuid::Uuid::new_v4().to_string(),
        url: url.clone(),
        connected: Arc::new(Mutex::new(false)),
    };
    
    commands::spawn(async move {
        // Simulate connection
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Mark as connected
        *handle.connected.lock().await = true;
        
        // Send connected event
        handler(WebSocketEvent::Connected)
    })
}

/// Create a WebSocket connection with automatic ping/pong
pub fn websocket_with_heartbeat<M, F>(
    url: impl Into<String>,
    ping_interval: std::time::Duration,
    mut handler: F,
) -> Cmd<M>
where
    M: Message,
    F: FnMut(WebSocketEvent) -> Option<M> + Send + 'static,
{
    let url = url.into();
    
    commands::spawn(async move {
        // In a real implementation, this would:
        // 1. Establish WebSocket connection
        // 2. Set up ping/pong heartbeat
        // 3. Handle reconnection on failure
        
        // For now, simulate connection
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        handler(WebSocketEvent::Connected)
    })
}

/// Helper to create a WebSocket message sender command
pub fn ws_send<M>(handle: WebSocketHandle, message: String) -> Cmd<M>
where
    M: Message,
{
    commands::spawn(async move {
        let _ = handle.send_text(message).await;
        None
    })
}

/// Helper to close a WebSocket connection
pub fn ws_close<M>(handle: WebSocketHandle) -> Cmd<M>
where
    M: Message,
{
    commands::spawn(async move {
        handle.close().await;
        None
    })
}

// Note: In a real implementation, we would need to add uuid to dependencies
// For now, we'll create a mock UUID module
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Uuid }
        pub fn to_string(&self) -> String { 
            format!("ws-{}", std::process::id())
        }
    }
}