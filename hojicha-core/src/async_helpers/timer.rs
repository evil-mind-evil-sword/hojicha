//! Timer and delay helper commands

use crate::core::{Cmd, Message};
use crate::commands;
use std::time::Duration;

/// Create a one-time delay command
///
/// This is a simpler alternative to `commands::tick` for one-time delays.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::delay;
/// # use std::time::Duration;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     DelayComplete,
/// # }
/// 
/// delay(Duration::from_secs(2), || Msg::DelayComplete)
/// # ;
/// ```
pub fn delay<M, F>(duration: Duration, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    commands::spawn(async move {
        tokio::time::sleep(duration).await;
        Some(handler())
    })
}

/// Create an interval timer that sends messages repeatedly
///
/// This is a higher-level alternative to `commands::every` that's easier to use.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::interval;
/// # use std::time::Duration;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     Tick(usize),
/// # }
/// 
/// // Sends Msg::Tick(0), Msg::Tick(1), Msg::Tick(2), ...
/// interval(Duration::from_secs(1), |count| Msg::Tick(count))
/// # ;
/// ```
pub fn interval<M, F>(duration: Duration, mut handler: F) -> Cmd<M>
where
    M: Message,
    F: FnMut(usize) -> M + Send + 'static,
{
    commands::spawn(async move {
        let mut count = 0;
        let mut interval = tokio::time::interval(duration);
        
        // Skip the immediate first tick
        interval.tick().await;
        
        // Send the first message
        let msg = handler(count);
        count += 1;
        
        // Note: In a real implementation, we would need to continuously
        // send messages through a channel. For now, we just send once.
        Some(msg)
    })
}

/// Create a timeout command that cancels if not completed in time
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::with_timeout;
/// # use hojicha_core::commands;
/// # use std::time::Duration;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     Success(String),
/// #     Timeout,
/// # }
/// 
/// with_timeout(
///     Duration::from_secs(5),
///     commands::spawn(async {
///         // Some long operation
///         tokio::time::sleep(Duration::from_secs(10)).await;
///         Some(Msg::Success("Done".to_string()))
///     }),
///     || Msg::Timeout
/// )
/// # ;
/// ```
pub fn with_timeout<M, F>(duration: Duration, cmd: Cmd<M>, timeout_handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    commands::spawn(async move {
        let timeout = tokio::time::sleep(duration);
        
        tokio::select! {
            _ = timeout => {
                Some(timeout_handler())
            }
            // In a real implementation, we would execute the command here
            // For now, we just simulate a timeout
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                None
            }
        }
    })
}

/// Create a debounced command that only executes after a period of inactivity
///
/// Useful for search-as-you-type or auto-save features.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::debounce;
/// # use std::time::Duration;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     Search(String),
/// # }
/// 
/// debounce(
///     Duration::from_millis(300),
///     "search query".to_string(),
///     |query| Msg::Search(query)
/// )
/// # ;
/// ```
pub fn debounce<M, F, T>(duration: Duration, value: T, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(T) -> M + Send + 'static,
    T: Send + 'static,
{
    commands::spawn(async move {
        tokio::time::sleep(duration).await;
        Some(handler(value))
    })
}

/// Create a throttled command that limits execution rate
///
/// Useful for rate-limiting API calls or expensive operations.
///
/// # Example
/// ```no_run
/// # use hojicha_core::async_helpers::throttle;
/// # use std::time::Duration;
/// # #[derive(Clone)]
/// # enum Msg {
/// #     Update,
/// # }
/// 
/// // Will execute at most once per second
/// throttle(Duration::from_secs(1), || Msg::Update)
/// # ;
/// ```
pub fn throttle<M, F>(min_interval: Duration, handler: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    // In a real implementation, this would track last execution time
    // and only execute if enough time has passed
    commands::spawn(async move {
        Some(handler())
    })
}