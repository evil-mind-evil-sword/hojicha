//! Stream subscription support for async event sources
//!
//! This module provides the `Subscription` type for managing subscriptions to
//! async streams (like WebSocket connections, file watchers, or timers). Subscriptions
//! automatically forward stream items as messages to your program's event loop.
//!
//! # Example
//!
//! ```ignore
//! use futures::stream;
//! use std::time::Duration;
//!
//! // Subscribe to a stream of periodic events
//! let stream = stream::repeat(MyMsg::Tick)
//!     .throttle(Duration::from_secs(1));
//!
//! let subscription = program.subscribe(stream);
//!
//! // The stream will send MyMsg::Tick every second
//! // Until the subscription is dropped or cancelled
//!
//! // Cancel when done
//! subscription.cancel().await;
//! ```
//!
//! # Automatic Cleanup
//!
//! Subscriptions are automatically cancelled when dropped, ensuring no resource
//! leaks from forgotten stream subscriptions.

use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// A handle to a running stream subscription
pub struct Subscription {
    handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl Subscription {
    /// Create a new subscription with the given task handle and cancellation token
    pub(crate) fn new(handle: JoinHandle<()>, cancel_token: CancellationToken) -> Self {
        Self {
            handle,
            cancel_token,
        }
    }

    /// Cancel the subscription
    ///
    /// This will stop the stream from sending more events to the program.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Check if the subscription is still active
    pub fn is_active(&self) -> bool {
        !self.handle.is_finished() && !self.cancel_token.is_cancelled()
    }

    /// Check if the subscription has completed
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Cancel the subscription when dropped
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_subscription_cancel() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            loop {
                if token_clone.is_cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        let subscription = Subscription::new(handle, token.clone());
        assert!(subscription.is_active());

        subscription.cancel();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(!subscription.is_active());
    }

    #[tokio::test]
    async fn test_subscription_drop_cancels() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = tokio::spawn(async move {
            loop {
                if token_clone.is_cancelled() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        {
            let _subscription = Subscription::new(handle, token.clone());
            // Subscription dropped here
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(token.is_cancelled());
    }
}
