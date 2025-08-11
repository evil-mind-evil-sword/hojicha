//! High-level async helper commands for common operations
//!
//! This module provides ergonomic helper functions for common async operations
//! like HTTP requests, WebSocket connections, and file I/O.
//! 
//! ## Available Helpers
//! 
//! ### HTTP Operations
//! Simple HTTP requests with automatic JSON handling:
//! ```no_run
//! # use hojicha_core::async_helpers::{http_get, http_post, HttpResponse, HttpError};
//! # use hojicha_core::Cmd;
//! # enum Msg { DataLoaded(String), Error(String), UserCreated(String) }
//! // GET request
//! let cmd: Cmd<Msg> = http_get("https://api.example.com/data", |result| {
//!     match result {
//!         Ok(response) => Msg::DataLoaded(response.body),
//!         Err(err) => Msg::Error(err.to_string()),
//!     }
//! });
//! 
//! // POST with JSON body (as string)
//! let json_body = r#"{"name": "Alice"}"#;
//! let cmd = http_post(
//!     "https://api.example.com/users",
//!     json_body,
//!     |result| match result {
//!         Ok(response) => Msg::UserCreated(response.body),
//!         Err(e) => Msg::Error(e.to_string()),
//!     }
//! );
//! ```
//! 
//! ### WebSocket Connections
//! Real-time bidirectional communication:
//! ```no_run
//! # use hojicha_core::async_helpers::{websocket, WebSocketEvent};
//! # use hojicha_core::Cmd;
//! # enum Msg { WsConnected, WsMessage(String), WsError(String), WsDisconnected, WsBinary(Vec<u8>) }
//! let cmd: Cmd<Msg> = websocket("wss://echo.websocket.org", |event| {
//!     Some(match event {
//!         WebSocketEvent::Connected => Msg::WsConnected,
//!         WebSocketEvent::Message(text) => Msg::WsMessage(text),
//!         WebSocketEvent::Binary(data) => Msg::WsBinary(data),
//!         WebSocketEvent::Error(err) => Msg::WsError(err.to_string()),
//!         WebSocketEvent::Closed(_) => Msg::WsDisconnected,
//!     })
//! });
//! ```
//! 
//! ### File Operations
//! Async file I/O and watching:
//! ```no_run
//! # use hojicha_core::async_helpers::{read_file, write_file, watch_file};
//! # use hojicha_core::Cmd;
//! # enum Msg { ConfigLoaded(String), Error(String), FileChanged }
//! // Read file
//! let cmd: Cmd<Msg> = read_file("config.json", |result| {
//!     result.map(Msg::ConfigLoaded)
//!           .unwrap_or_else(|e| Msg::Error(e.to_string()))
//! });
//! 
//! // Watch for changes
//! let cmd = watch_file("data.csv", |_| Some(Msg::FileChanged));
//! ```
//! 
//! ### Timers
//! Delays and intervals:
//! ```no_run
//! # use hojicha_core::async_helpers::{delay, interval};
//! # use hojicha_core::Cmd;
//! # use std::time::Duration;
//! # enum Msg { TimerExpired, Tick(usize) }
//! // One-shot delay
//! let cmd: Cmd<Msg> = delay(Duration::from_secs(2), || Msg::TimerExpired);
//! 
//! // Repeating interval
//! let cmd = interval(Duration::from_secs(1), |count| Msg::Tick(count));
//! ```

pub mod http;
pub mod websocket;
pub mod file_io;
pub mod timer;

pub use http::{http_get, http_post, http_request, HttpMethod, HttpError, HttpResponse};
pub use websocket::{websocket, WebSocketEvent, WebSocketError};
pub use file_io::{read_file, write_file, watch_file, FileError, FileEvent};
pub use timer::{delay, interval, with_timeout, debounce, throttle};

use crate::core::{Cmd, Message};

/// Result type for async operations
pub type AsyncResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Common configuration for async operations
#[derive(Debug, Clone)]
pub struct AsyncConfig {
    /// Timeout for operations
    pub timeout: Option<std::time::Duration>,
    /// Number of retries
    pub retries: u32,
    /// Backoff strategy for retries
    pub backoff: BackoffStrategy,
}

/// Backoff strategy for retries
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// No backoff
    None,
    /// Linear backoff (delay * attempt)
    Linear(std::time::Duration),
    /// Exponential backoff (delay * 2^attempt)
    Exponential(std::time::Duration),
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            timeout: Some(std::time::Duration::from_secs(30)),
            retries: 0,
            backoff: BackoffStrategy::None,
        }
    }
}

impl AsyncConfig {
    /// Create a config with a specific timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Create a config with retries
    pub fn with_retries(mut self, retries: u32, backoff: BackoffStrategy) -> Self {
        self.retries = retries;
        self.backoff = backoff;
        self
    }
}