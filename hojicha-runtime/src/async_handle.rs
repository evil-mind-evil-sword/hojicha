//! Handle for managing cancellable async operations

//! Async operation handle with cancellation support
//!
//! This module provides `AsyncHandle<T>` for managing long-running async operations
//! with cooperative cancellation support. This is essential for building responsive
//! TUI applications that need to cancel operations when users navigate away or quit.
//!
//! # Example
//!
//! ```ignore
//! use hojicha::async_handle::AsyncHandle;
//!
//! // Spawn a cancellable operation
//! let handle = program.spawn_cancellable(|token| async move {
//!     loop {
//!         tokio::select! {
//!             _ = token.cancelled() => {
//!                 // Clean up and exit
//!                 return Ok("Cancelled");
//!             }
//!             result = fetch_data() => {
//!                 return Ok(result);
//!             }
//!         }
//!     }
//! });
//!
//! // Later, cancel if needed
//! handle.cancel().await;
//! ```

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// A handle to a cancellable async operation
///
/// `AsyncHandle` allows you to:
/// - Cancel long-running operations cooperatively
/// - Check if an operation is still running
/// - Wait for completion with `.await`
/// - Abort forcefully if needed
///
/// The handle automatically cancels the operation when dropped.
pub struct AsyncHandle<T> {
    handle: JoinHandle<T>,
    cancel_token: CancellationToken,
}

impl<T> AsyncHandle<T> {
    /// Create a new async handle
    pub(crate) fn new(handle: JoinHandle<T>, cancel_token: CancellationToken) -> Self {
        Self {
            handle,
            cancel_token,
        }
    }

    /// Cancel the operation
    ///
    /// This sends a cancellation signal to the async task. The task must
    /// cooperatively check for cancellation to actually stop.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Check if the operation is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Check if the operation is still running
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished() && !self.cancel_token.is_cancelled()
    }

    /// Check if the operation has finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }

    /// Abort the task immediately
    ///
    /// This is more forceful than cancel() - it immediately aborts the task
    /// without waiting for cooperative cancellation.
    pub fn abort(&self) {
        self.handle.abort();
    }

    /// Get the cancellation token for cooperative cancellation
    ///
    /// This can be cloned and passed to child tasks for hierarchical cancellation.
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancel_token
    }
}

impl<T> Drop for AsyncHandle<T> {
    fn drop(&mut self) {
        // Cancel the operation when the handle is dropped
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_async_handle_cancel() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            loop {
                if token_clone.is_cancelled() {
                    return "Cancelled";
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        let async_handle = AsyncHandle::new(handle, token);
        assert!(async_handle.is_running());

        async_handle.cancel();
        assert!(async_handle.is_cancelled());

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(async_handle.is_finished());
    }

    #[tokio::test]
    async fn test_async_handle_drop_cancels() {
        let token = CancellationToken::new();
        let token_clone = token.clone();
        let token_check = token.clone();

        let handle = tokio::spawn(async move {
            loop {
                if token_clone.is_cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        {
            let _async_handle = AsyncHandle::new(handle, token);
            // Handle dropped here
        }

        // Should be cancelled after drop
        assert!(token_check.is_cancelled());
    }

    #[tokio::test]
    async fn test_async_handle_abort() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            loop {
                if token_clone.is_cancelled() {
                    return "Cancelled";
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        let async_handle = AsyncHandle::new(handle, token);

        // Abort immediately
        async_handle.abort();

        // Should finish quickly
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(async_handle.is_finished());
    }
}
