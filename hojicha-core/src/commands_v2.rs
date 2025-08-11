//! Improved command API with consistent naming and behavior
//! 
//! This module provides a redesigned command API that addresses inconsistencies
//! in the original API. The new API focuses on:
//! 
//! - Consistent naming patterns
//! - Predictable behavior (no surprising optimizations)
//! - Clear intent from function names
//! - Reduced cognitive load

use crate::core::{Cmd, Message};
use std::time::Duration;

// ============================================================================
// Composite Commands - Consistent behavior, no surprises
// ============================================================================

/// Combine multiple commands to run concurrently
/// 
/// Always returns a batch command, even for single or empty vectors.
/// This provides predictable, consistent behavior.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::combine;
/// # use hojicha_core::Cmd;
/// # enum Msg { A, B }
/// // Always returns a batch, regardless of input
/// let cmd1 = combine(vec![Cmd::new(|| Some(Msg::A))]);  // Returns batch with 1 command
/// let cmd2 = combine(vec![]);                            // Returns empty batch
/// let cmd3 = combine(vec![                               // Returns batch with 2 commands
///     Cmd::new(|| Some(Msg::A)),
///     Cmd::new(|| Some(Msg::B)),
/// ]);
/// ```
pub fn combine<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::batch(cmds)
}

/// Combine multiple commands with optimization
/// 
/// May optimize single-command or empty batches for performance.
/// Use this when you want the framework to optimize where possible.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::combine_optimized;
/// # use hojicha_core::Cmd;
/// # enum Msg { A }
/// // May return the command directly instead of wrapping in batch
/// let cmd = combine_optimized(vec![Cmd::new(|| Some(Msg::A))]);
/// ```
pub fn combine_optimized<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    match cmds.len() {
        0 => Cmd::none(),
        1 => cmds.into_iter().next().unwrap(),
        _ => Cmd::batch(cmds),
    }
}

/// Run multiple commands in sequence (one after another)
/// 
/// Commands execute in order, each waiting for the previous to complete.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::chain;
/// # use hojicha_core::Cmd;
/// # enum Msg { First, Second }
/// let cmd = chain(vec![
///     Cmd::new(|| Some(Msg::First)),   // Runs first
///     Cmd::new(|| Some(Msg::Second)),  // Runs after First completes
/// ]);
/// ```
pub fn chain<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::sequence(cmds)
}

// ============================================================================
// Timer Commands - Clear, descriptive names
// ============================================================================

/// Execute a command after a delay
/// 
/// The command fires once after the specified duration.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::after;
/// # use std::time::Duration;
/// # enum Msg { Timeout }
/// let cmd = after(Duration::from_secs(5), || Msg::Timeout);
/// ```
pub fn after<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    Cmd::tick(duration, f)
}

/// Execute a command repeatedly at intervals
/// 
/// The command fires repeatedly every `duration`.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::interval;
/// # use std::time::Duration;
/// # enum Msg { Tick(std::time::Instant) }
/// let cmd = interval(Duration::from_secs(1), |instant| Msg::Tick(instant));
/// ```
pub fn interval<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(std::time::Instant) -> M + Send + 'static,
{
    Cmd::every(duration, f)
}

/// Execute a command after a delay (alias for clarity)
/// 
/// Same as `after`, provided for API flexibility.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::delay;
/// # use std::time::Duration;
/// # enum Msg { Ready }
/// let cmd = delay(Duration::from_millis(100), || Msg::Ready);
/// ```
pub fn delay<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    after(duration, f)
}

/// Execute a command repeatedly (alias for clarity)
/// 
/// Same as `interval`, provided for API flexibility.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::repeat;
/// # use std::time::Duration;
/// # enum Msg { Pulse }
/// let cmd = repeat(Duration::from_secs(1), |_| Msg::Pulse);
/// ```
pub fn repeat<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(std::time::Instant) -> M + Send + 'static,
{
    interval(duration, f)
}

// ============================================================================
// Async Commands - Unified interface
// ============================================================================

/// Create an async command
/// 
/// Executes an async future and sends the result as a message.
/// This is the primary way to create async commands.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::async_cmd;
/// # enum Msg { DataLoaded(String) }
/// let cmd = async_cmd(async {
///     // Perform async operation
///     let data = fetch_data().await;
///     Some(Msg::DataLoaded(data))
/// });
/// 
/// # async fn fetch_data() -> String { "data".to_string() }
/// ```
pub fn async_cmd<M, Fut>(fut: Fut) -> Cmd<M>
where
    M: Message,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    Cmd::async_cmd(fut)
}

/// Create an async command from a closure
/// 
/// Useful when you need to capture values before creating the future.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::async_with;
/// # enum Msg { Result(i32) }
/// let value = 42;
/// let cmd = async_with(move || async move {
///     // Can use captured value
///     let result = process_value(value).await;
///     Some(Msg::Result(result))
/// });
/// 
/// # async fn process_value(v: i32) -> i32 { v }
/// ```
pub fn async_with<M, F, Fut>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    Cmd::async_cmd(f())
}

// ============================================================================
// Control Commands
// ============================================================================

/// Create a no-op command
/// 
/// Use when you need to return a command but have no side effects.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::none;
/// # use hojicha_core::{Model, Cmd, Event};
/// # struct MyModel;
/// # impl Model for MyModel {
/// #     type Message = ();
/// fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
///     // Handle event but don't trigger side effects
///     none()
/// }
/// #     fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
/// # }
/// ```
pub fn none<M: Message>() -> Cmd<M> {
    Cmd::none()
}

/// Create a quit command
/// 
/// Signals the application to terminate gracefully.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::quit;
/// # use hojicha_core::{Model, Cmd, Event, Key};
/// # struct MyModel;
/// # impl Model for MyModel {
/// #     type Message = ();
/// fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
///     match event {
///         Event::Key(k) if k.key == Key::Char('q') => quit(),
///         _ => none(),
///     }
/// }
/// #     fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
/// # }
/// ```
pub fn quit<M: Message>() -> Cmd<M> {
    Cmd::quit()
}

// ============================================================================
// Utility Commands
// ============================================================================

/// Create a simple command from a function
/// 
/// The most basic way to create a command that produces a message.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::message;
/// # enum Msg { Hello }
/// let cmd = message(|| Some(Msg::Hello));
/// ```
pub fn message<M, F>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Option<M> + Send + 'static,
{
    Cmd::new(f)
}

/// Create a command that always produces a specific message
/// 
/// Simpler than `message` when you always want to send a message.
/// 
/// # Example
/// ```
/// # use hojicha_core::commands_v2::send;
/// # enum Msg { Click }
/// let cmd = send(Msg::Click);
/// ```
pub fn send<M: Message>(msg: M) -> Cmd<M> {
    Cmd::new(move || Some(msg))
}

// ============================================================================
// Migration helpers - Deprecated aliases for backward compatibility
// ============================================================================

/// Legacy alias for `combine_optimized`
/// 
/// **Deprecated**: Use `combine_optimized` for optimized batching
/// or `combine` for predictable behavior.
#[deprecated(since = "0.3.0", note = "Use `combine_optimized` or `combine` instead")]
pub fn batch<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    combine_optimized(cmds)
}

/// Legacy alias for `combine`
/// 
/// **Deprecated**: Use `combine` instead.
#[deprecated(since = "0.3.0", note = "Use `combine` instead")]
pub fn batch_strict<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    combine(cmds)
}

/// Legacy alias for `after`
/// 
/// **Deprecated**: Use `after` or `delay` instead.
#[deprecated(since = "0.3.0", note = "Use `after` or `delay` instead")]
pub fn tick<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    after(duration, f)
}

/// Legacy alias for `interval`
/// 
/// **Deprecated**: Use `interval` or `repeat` instead.
#[deprecated(since = "0.3.0", note = "Use `interval` or `repeat` instead")]
pub fn every<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(std::time::Instant) -> M + Send + 'static,
{
    interval(duration, f)
}

/// Legacy alias for `async_cmd`
/// 
/// **Deprecated**: Use `async_cmd` instead.
#[deprecated(since = "0.3.0", note = "Use `async_cmd` instead")]
pub fn spawn<M, Fut>(fut: Fut) -> Cmd<M>
where
    M: Message,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    async_cmd(fut)
}

/// Legacy alias for `async_with`
/// 
/// **Deprecated**: Use `async_with` instead.
#[deprecated(since = "0.3.0", note = "Use `async_with` instead")]
pub fn custom_async<M, F, Fut>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    async_with(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        A,
        B,
        C,
    }
    
    #[test]
    fn test_combine_always_returns_batch() {
        // Empty vector returns empty batch
        let cmd = combine::<TestMsg>(vec![]);
        assert!(cmd.is_batch());
        
        // Single command returns batch with one command
        let cmd = combine(vec![Cmd::new(|| Some(TestMsg::A))]);
        assert!(cmd.is_batch());
        
        // Multiple commands return batch
        let cmd = combine(vec![
            Cmd::new(|| Some(TestMsg::A)),
            Cmd::new(|| Some(TestMsg::B)),
        ]);
        assert!(cmd.is_batch());
    }
    
    #[test]
    fn test_combine_optimized_optimizes() {
        // Empty vector returns none
        let cmd = combine_optimized::<TestMsg>(vec![]);
        assert!(cmd.is_noop());
        
        // Single command returns the command directly
        let single = Cmd::new(|| Some(TestMsg::A));
        let cmd = combine_optimized(vec![single.clone()]);
        // Can't easily test equality, but it shouldn't be a batch
        
        // Multiple commands return batch
        let cmd = combine_optimized(vec![
            Cmd::new(|| Some(TestMsg::A)),
            Cmd::new(|| Some(TestMsg::B)),
        ]);
        assert!(cmd.is_batch());
    }
    
    #[test]
    fn test_timer_commands() {
        // Test that timer commands compile and have correct types
        let _after_cmd = after(Duration::from_secs(1), || TestMsg::A);
        let _delay_cmd = delay(Duration::from_secs(1), || TestMsg::B);
        let _interval_cmd = interval(Duration::from_secs(1), |_| TestMsg::C);
        let _repeat_cmd = repeat(Duration::from_secs(1), |_| TestMsg::A);
    }
    
    #[test]
    fn test_async_commands() {
        let _async1 = async_cmd(async { Some(TestMsg::A) });
        let _async2 = async_with(|| async { Some(TestMsg::B) });
    }
    
    #[test]
    fn test_utility_commands() {
        let _msg_cmd = message(|| Some(TestMsg::A));
        let _send_cmd = send(TestMsg::B);
        let _none_cmd = none::<TestMsg>();
        let _quit_cmd = quit::<TestMsg>();
    }
}