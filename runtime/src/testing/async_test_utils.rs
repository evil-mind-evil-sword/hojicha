//! Utilities for testing async and timing-dependent behavior

use hojicha_core::core::{Cmd, Message};
use hojicha_core::event::Event;
use std::sync::mpsc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// A test harness for async operations with controllable time
pub struct AsyncTestHarness {
    runtime: Runtime,
}

impl AsyncTestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().expect("Failed to create runtime"),
        }
    }

    /// Execute a command and collect all messages it produces
    pub fn execute_command<M: Message + Clone + Send + 'static>(
        &self,
        cmd: Cmd<M>,
    ) -> Vec<M> {
        let (tx, rx) = mpsc::sync_channel(100);
        
        // Import CommandExecutor
        use crate::program::CommandExecutor;
        let executor = CommandExecutor::new().expect("Failed to create executor");
        
        // Execute the command
        executor.execute(cmd, tx);
        
        // Collect messages with a timeout
        let mut messages = Vec::new();
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(1);
        
        while start.elapsed() < timeout {
            match rx.try_recv() {
                Ok(Event::User(msg)) => messages.push(msg),
                Ok(_) => {} // Ignore other events
                Err(mpsc::TryRecvError::Empty) => {
                    // Give async tasks time to complete
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }
        
        messages
    }

    /// Execute a tick command immediately (for testing)
    pub fn execute_tick_now<M: Message, F>(&self, _duration: Duration, f: F) -> M
    where
        F: FnOnce() -> M,
    {
        // In tests, ignore the duration and execute immediately
        f()
    }

    /// Execute an async future and wait for result
    pub fn block_on_async<M: Message + Send + 'static>(
        &self,
        future: impl std::future::Future<Output = Option<M>> + Send + 'static,
    ) -> Option<M> {
        self.runtime.block_on(future)
    }

    /// Execute a command and wait for completion
    pub fn execute_and_wait<M: Message + Clone + Send + 'static>(
        &self,
        cmd: Cmd<M>,
        wait_duration: Duration,
    ) -> Vec<M> {
        let (tx, rx) = mpsc::sync_channel(100);
        
        use crate::program::CommandExecutor;
        let executor = CommandExecutor::new().expect("Failed to create executor");
        
        // Start command execution
        executor.execute(cmd, tx);
        
        // Wait for operations to complete
        std::thread::sleep(wait_duration);
        
        // Collect all messages
        let mut messages = Vec::new();
        while let Ok(Event::User(msg)) = rx.try_recv() {
            messages.push(msg);
        }
        
        messages
    }
}

/// Extension trait for Cmd to make testing easier
pub trait CmdTestExt<M: Message> {
    /// Execute the command synchronously in tests
    fn execute_sync(self) -> Option<M>;
    
    /// Execute with a test harness
    fn execute_with_harness(self, harness: &AsyncTestHarness) -> Vec<M>;
}

impl<M: Message + Clone + Send + 'static> CmdTestExt<M> for Cmd<M> {
    fn execute_sync(self) -> Option<M> {
        // For simple commands, execute directly
        if !self.is_tick() && !self.is_every() && !self.is_async() {
            self.test_execute().ok().flatten()
        } else {
            // For async/timed commands, use a harness
            let harness = AsyncTestHarness::new();
            harness.execute_command(self).into_iter().next()
        }
    }
    
    fn execute_with_harness(self, harness: &AsyncTestHarness) -> Vec<M> {
        harness.execute_command(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hojicha_core::commands;
    
    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Tick,
        Every,
        Async,
    }
    
    #[test]
    fn test_async_harness() {
        let harness = AsyncTestHarness::new();
        
        // Test tick command
        let tick_cmd = commands::tick(Duration::from_millis(10), || TestMsg::Tick);
        let messages = harness.execute_command(tick_cmd);
        assert_eq!(messages, vec![TestMsg::Tick]);
    }
    
    #[test]
    fn test_execute_tick_now() {
        let harness = AsyncTestHarness::new();
        let msg = harness.execute_tick_now(Duration::from_secs(100), || TestMsg::Tick);
        assert_eq!(msg, TestMsg::Tick);
    }
    
    #[test]
    fn test_block_on_async() {
        let harness = AsyncTestHarness::new();
        let result = harness.block_on_async(async {
            tokio::time::sleep(Duration::from_millis(1)).await;
            Some(TestMsg::Async)
        });
        assert_eq!(result, Some(TestMsg::Async));
    }
    
    #[test]
    fn test_cmd_sync_execution() {
        // Test that simple commands can be executed synchronously
        let cmd = commands::custom(|| Some(TestMsg::Async));
        let result = cmd.execute_sync();
        assert_eq!(result, Some(TestMsg::Async));
    }
}