//! Stream builder utilities for common async patterns
//!
//! This module provides helper functions for creating common types of streams
//! that can be used with the `Program::subscribe()` method.

use futures::stream::{Stream, StreamExt};
use std::time::Duration;

/// Create a stream that emits at regular intervals
///
/// This is useful for creating timers, periodic updates, or any task that needs
/// to run at regular intervals.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::interval_stream;
/// use std::time::Duration;
///
/// let stream = interval_stream(Duration::from_secs(1))
///     .map(|_| Msg::Tick);
///
/// program.subscribe(stream);
/// ```
pub fn interval_stream(duration: Duration) -> impl Stream<Item = ()> {
    use tokio_stream::wrappers::IntervalStream;

    let interval = tokio::time::interval(duration);
    IntervalStream::new(interval).map(|_| ())
}

/// Create a stream from a channel receiver
///
/// This allows you to bridge between channel-based APIs and the stream-based
/// subscription system.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::channel_stream;
///
/// let (tx, rx) = tokio::sync::mpsc::channel(100);
///
/// // Spawn something that sends to the channel
/// tokio::spawn(async move {
///     loop {
///         tx.send(42).await.ok();
///         tokio::time::sleep(Duration::from_secs(1)).await;
///     }
/// });
///
/// let stream = channel_stream(rx).map(|val| Msg::Value(val));
/// program.subscribe(stream);
/// ```
pub fn channel_stream<T>(rx: tokio::sync::mpsc::Receiver<T>) -> impl Stream<Item = T> + Send + Unpin
where
    T: Send + 'static,
{
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    }))
}

/// Create a stream from an unbounded channel receiver
///
/// Similar to `channel_stream` but for unbounded channels.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::unbounded_channel_stream;
///
/// let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
///
/// let stream = unbounded_channel_stream(rx).map(|val| Msg::Value(val));
/// program.subscribe(stream);
/// ```
pub fn unbounded_channel_stream<T>(
    rx: tokio::sync::mpsc::UnboundedReceiver<T>,
) -> impl Stream<Item = T> + Send + Unpin
where
    T: Send + 'static,
{
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    }))
}

/// Create a stream that emits once after a delay
///
/// This is useful for delayed actions or one-time timeouts.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::delayed_stream;
/// use std::time::Duration;
///
/// let stream = delayed_stream(Duration::from_secs(5))
///     .map(|_| Msg::DelayedAction);
///
/// program.subscribe(stream);
/// ```
pub fn delayed_stream(duration: Duration) -> impl Stream<Item = ()> + Send + Unpin {
    Box::pin(futures::stream::once(async move {
        tokio::time::sleep(duration).await;
    }))
}

/// Create a stream that completes after a delay (alias for delayed_stream)
///
/// This is useful for timeouts or delayed actions.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::timeout_stream;
/// use std::time::Duration;
///
/// let stream = timeout_stream(Duration::from_secs(5))
///     .map(|_| Msg::Timeout);
///
/// program.subscribe(stream);
/// ```
pub fn timeout_stream(duration: Duration) -> impl Stream<Item = ()> + Send + Unpin {
    delayed_stream(duration)
}

/// Create a stream that merges multiple streams
///
/// This allows you to combine multiple event sources into a single stream.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::{interval_stream, merge_streams};
/// use std::time::Duration;
///
/// let stream1 = interval_stream(Duration::from_millis(100)).map(|_| Msg::Fast);
/// let stream2 = interval_stream(Duration::from_secs(1)).map(|_| Msg::Slow);
///
/// let merged = merge_streams(vec![stream1.boxed(), stream2.boxed()]);
/// program.subscribe(merged);
/// ```
pub fn merge_streams<T>(
    streams: Vec<Box<dyn Stream<Item = T> + Send + Unpin>>,
) -> impl Stream<Item = T>
where
    T: Send + 'static,
{
    use futures::stream::SelectAll;

    let mut select_all = SelectAll::new();
    for stream in streams {
        select_all.push(stream);
    }
    select_all
}

/// Create a stream from a watch channel
///
/// This creates a stream that emits whenever the watched value changes.
///
/// # Example
/// ```ignore
/// use hojicha::stream_builders::watch_stream;
///
/// let (tx, rx) = tokio::sync::watch::channel(0);
///
/// // Something updates the watched value
/// tokio::spawn(async move {
///     let mut count = 0;
///     loop {
///         count += 1;
///         tx.send(count).ok();
///         tokio::time::sleep(Duration::from_secs(1)).await;
///     }
/// });
///
/// let stream = watch_stream(rx).map(|val| Msg::CountChanged(val));
/// program.subscribe(stream);
/// ```
pub fn watch_stream<T>(rx: tokio::sync::watch::Receiver<T>) -> impl Stream<Item = T> + Send + Unpin
where
    T: Clone + Send + Sync + 'static,
{
    Box::pin(futures::stream::unfold(rx, |mut rx| async move {
        rx.changed().await.ok()?;
        let value = rx.borrow().clone();
        Some((value, rx))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use std::time::Duration;

    #[tokio::test]
    async fn test_interval_stream() {
        let mut stream = interval_stream(Duration::from_millis(10)).take(3);

        let mut count = 0;
        while stream.next().await.is_some() {
            count += 1;
        }

        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_timeout_stream() {
        let mut stream = timeout_stream(Duration::from_millis(10));

        let result = stream.next().await;
        assert!(result.is_some());

        let result = stream.next().await;
        assert!(result.is_none()); // Stream should be done
    }

    #[tokio::test]
    async fn test_channel_stream() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);

        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        tx.send(3).await.unwrap();
        drop(tx);

        let mut stream = channel_stream(rx);

        assert_eq!(stream.next().await, Some(1));
        assert_eq!(stream.next().await, Some(2));
        assert_eq!(stream.next().await, Some(3));
        assert_eq!(stream.next().await, None);
    }
}
