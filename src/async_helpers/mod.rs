//! High-level async helper commands for common operations
//!
//! This module provides ergonomic helper functions for common async operations
//! like HTTP requests, WebSocket connections, and file I/O.

pub mod http;
pub mod websocket;
pub mod file_io;
pub mod timer;

pub use http::{http_get, http_post, http_request, HttpMethod, HttpError};
pub use websocket::{websocket, WebSocketEvent, WebSocketError};
pub use file_io::{read_file, write_file, watch_file, FileError};
pub use timer::{interval, delay};

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