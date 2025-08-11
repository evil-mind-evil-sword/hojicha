//! High-level async helper commands for common operations
//!
//! This module provides ergonomic helper functions for common async operations
//! like HTTP requests, WebSocket connections, and file I/O.
//! 
//! ## Available Helpers
//! 
//! ### HTTP Operations
//! Simple HTTP requests with automatic JSON handling:
//! ```ignore
//! use hojicha_core::async_helpers::{http_get, http_post};
//! 
//! // GET request
//! let cmd = http_get("https://api.example.com/data", |result| {
//!     match result {
//!         Ok(body) => Msg::DataLoaded(body),
//!         Err(err) => Msg::Error(err.to_string()),
//!     }
//! });
//! 
//! // POST with JSON
//! let cmd = http_post(
//!     "https://api.example.com/users",
//!     json!({"name": "Alice"}),
//!     |result| Msg::UserCreated(result)
//! );
//! ```
//! 
//! ### WebSocket Connections
//! Real-time bidirectional communication:
//! ```ignore
//! use hojicha_core::async_helpers::{websocket, WebSocketEvent};
//! 
//! let cmd = websocket("wss://echo.websocket.org", |event| {
//!     match event {
//!         WebSocketEvent::Message(text) => Msg::WsMessage(text),
//!         WebSocketEvent::Error(err) => Msg::WsError(err),
//!         WebSocketEvent::Closed => Msg::WsDisconnected,
//!     }
//! });
//! ```
//! 
//! ### File Operations
//! Async file I/O and watching:
//! ```ignore
//! use hojicha_core::async_helpers::{read_file, write_file, watch_file};
//! 
//! // Read file
//! let cmd = read_file("config.json", |result| {
//!     result.map(Msg::ConfigLoaded)
//!           .unwrap_or_else(|e| Msg::Error(e.to_string()))
//! });
//! 
//! // Watch for changes
//! let cmd = watch_file("data.csv", |_| Msg::FileChanged);
//! ```
//! 
//! ### Timers
//! Delays and intervals:
//! ```ignore
//! use hojicha_core::async_helpers::{delay, interval};
//! use std::time::Duration;
//! 
//! // One-shot delay
//! let cmd = delay(Duration::from_secs(2), || Msg::TimerExpired);
//! 
//! // Repeating interval
//! let cmd = interval(Duration::from_secs(1), || Msg::Tick);
//! ```

pub mod http;
pub mod websocket;
pub mod file_io;
pub mod timer;

pub use http::{http_get, http_post, http_request, HttpMethod, HttpError};
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