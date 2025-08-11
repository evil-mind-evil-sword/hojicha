//! Command execution logic extracted from Program for testability

use super::error_handler::{DefaultErrorHandler, ErrorHandler};
use hojicha_core::core::Cmd;
use hojicha_core::event::Event;
use std::panic::{self, AssertUnwindSafe};
use crate::panic_utils;
use std::sync::{Arc, mpsc};
use tokio::runtime::Runtime;

/// Executes commands and sends resulting messages
#[derive(Clone)]
pub struct CommandExecutor<M = ()> {
    runtime: Arc<Runtime>,
    error_handler: Arc<dyn ErrorHandler<M> + Send + Sync>,
}

impl<M> CommandExecutor<M>
where
    M: Clone + Send + 'static,
{
    /// Create a new command executor with default error handler
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            runtime: Arc::new(Runtime::new()?),
            error_handler: Arc::new(DefaultErrorHandler),
        })
    }

    /// Create a new command executor with custom error handler
    pub fn with_error_handler<H>(error_handler: H) -> std::io::Result<Self>
    where
        H: ErrorHandler<M> + Send + Sync + 'static,
    {
        Ok(Self {
            runtime: Arc::new(Runtime::new()?),
            error_handler: Arc::new(error_handler),
        })
    }

    /// Execute a command and send the result through the channel
    pub fn execute(&self, cmd: Cmd<M>, tx: mpsc::SyncSender<Event<M>>) {
        if cmd.is_noop() {
            // NoOp command - do nothing
        } else if cmd.is_quit() {
            // Handle quit command by sending a special quit event
            let _ = tx.send(Event::Quit);
        } else if cmd.is_exec_process() {
            // Handle exec process commands specially
            if let Some((_program, _args, _callback)) = cmd.take_exec_process() {
                // For testing, we just send an ExecProcess event
                // The actual process execution would be handled by the Program
                let _ = tx.send(Event::ExecProcess);
            }
        } else if cmd.is_batch() {
            // Handle batch commands by executing them concurrently
            if let Some(cmds) = cmd.take_batch() {
                self.execute_batch(cmds, tx);
            }
        } else if cmd.is_sequence() {
            // Handle sequence commands by executing them in order
            if let Some(cmds) = cmd.take_sequence() {
                self.execute_sequence(cmds, tx);
            }
        } else if cmd.is_tick() {
            // Handle tick command with async delay
            if let Some((duration, callback)) = cmd.take_tick() {
                let tx_clone = tx.clone();
                self.runtime.spawn(async move {
                    tokio::time::sleep(duration).await;
                    // Wrap callback execution in panic recovery
                    let result = panic::catch_unwind(AssertUnwindSafe(|| callback()));
                    match result {
                        Ok(msg) => {
                            let _ = tx_clone.send(Event::User(msg));
                        }
                        Err(panic) => {
                            let panic_msg = panic_utils::format_panic_message(panic, "Tick callback panicked");
                            eprintln!("{}", panic_msg);
                            // Continue running - don't crash the application
                        }
                    }
                });
            }
        } else if cmd.is_every() {
            // Handle every command with recurring async timer
            if let Some((duration, callback)) = cmd.take_every() {
                let tx_clone = tx.clone();
                self.runtime.spawn(async move {
                    // Since callback is FnOnce, we can only call it once
                    // For now, just execute once after delay
                    tokio::time::sleep(duration).await;
                    // Wrap callback execution in panic recovery
                    let result = panic::catch_unwind(AssertUnwindSafe(|| callback(std::time::Instant::now())));
                    match result {
                        Ok(msg) => {
                            let _ = tx_clone.send(Event::User(msg));
                        }
                        Err(panic) => {
                            let panic_msg = panic_utils::format_panic_message(panic, "Every callback panicked");
                            eprintln!("{}", panic_msg);
                            // Continue running - don't crash the application
                        }
                    }
                });
            }
        } else if cmd.is_async() {
            // Handle async command using shared runtime
            if let Some(future) = cmd.take_async() {
                let tx_clone = tx.clone();
                self.runtime.spawn(async move {
                    // The future is already boxed, just need to pin it
                    use std::pin::Pin;
                    let mut future = future;
                    let future = unsafe { Pin::new_unchecked(&mut *future) };
                    
                    // Actually await the future
                    if let Some(msg) = future.await {
                        let _ = tx_clone.send(Event::User(msg));
                    }
                });
            }
        } else {
            // Spawn async task for regular command execution (like Bubbletea's goroutines)
            let tx_clone = tx.clone();
            let error_handler = self.error_handler.clone();
            self.runtime.spawn(async move {
                // Wrap command execution in panic recovery
                let result = panic::catch_unwind(AssertUnwindSafe(|| cmd.execute()));
                
                match result {
                    Ok(Ok(Some(msg))) => {
                        let _ = tx_clone.send(Event::User(msg));
                    }
                    Ok(Ok(None)) => {
                        // Command executed successfully but produced no message
                    }
                    Ok(Err(error)) => {
                        // Use the configured error handler
                        error_handler.handle_error(error, &tx_clone);
                    }
                    Err(panic) => {
                        // Command panicked - log and recover
                        let panic_msg = panic_utils::format_panic_message(panic, "Command execution panicked");
                        eprintln!("{}", panic_msg);
                        // Continue running - don't crash the application
                    }
                }
            });
        }
    }

    /// Execute a batch of commands concurrently
    pub fn execute_batch(&self, commands: Vec<Cmd<M>>, tx: mpsc::SyncSender<Event<M>>) {
        // Spawn all commands concurrently (like Bubbletea's batch)
        for cmd in commands {
            // Each command runs in its own async task
            self.execute(cmd, tx.clone());
        }
    }

    /// Execute a sequence of commands (one after another)
    pub fn execute_sequence(&self, commands: Vec<Cmd<M>>, tx: mpsc::SyncSender<Event<M>>) {
        // Spawn async task to execute commands in sequence
        let tx_clone = tx.clone();
        let error_handler = self.error_handler.clone();
        self.runtime.spawn(async move {
            for cmd in commands {
                let tx_inner = tx_clone.clone();

                // Execute the command through the regular execution path
                if cmd.is_tick() {
                    if let Some((duration, callback)) = cmd.take_tick() {
                        tokio::time::sleep(duration).await;
                        // Wrap callback execution in panic recovery
                        let result = panic::catch_unwind(AssertUnwindSafe(|| callback()));
                        match result {
                            Ok(msg) => {
                                let _ = tx_inner.send(Event::User(msg));
                            }
                            Err(panic) => {
                                let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
                                    s.clone()
                                } else if let Some(s) = panic.downcast_ref::<&str>() {
                                    s.to_string()
                                } else {
                                    "Unknown panic in tick callback".to_string()
                                };
                                eprintln!("Tick callback panicked: {}", panic_msg);
                                // Continue running - don't crash the application
                            }
                        }
                    }
                } else if cmd.is_every() {
                    if let Some((duration, callback)) = cmd.take_every() {
                        tokio::time::sleep(duration).await;
                        // Wrap callback execution in panic recovery
                        let result = panic::catch_unwind(AssertUnwindSafe(|| callback(std::time::Instant::now())));
                        match result {
                            Ok(msg) => {
                                let _ = tx_inner.send(Event::User(msg));
                            }
                            Err(panic) => {
                                let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
                                    s.clone()
                                } else if let Some(s) = panic.downcast_ref::<&str>() {
                                    s.to_string()
                                } else {
                                    "Unknown panic in every callback".to_string()
                                };
                                eprintln!("Every callback panicked: {}", panic_msg);
                                // Continue running - don't crash the application
                            }
                        }
                    }
                } else {
                    // Regular command execution with panic recovery
                    let result = panic::catch_unwind(AssertUnwindSafe(|| cmd.execute()));
                    match result {
                        Ok(Ok(Some(msg))) => {
                            let _ = tx_inner.send(Event::User(msg));
                        }
                        Ok(Ok(None)) => {}
                        Ok(Err(error)) => {
                            // Use the configured error handler
                            error_handler.handle_error(error, &tx_inner);
                        }
                        Err(panic) => {
                            let panic_msg = panic_utils::format_panic_message(panic, "Sequence command panicked");
                            eprintln!("{}", panic_msg);
                            // Continue running - don't crash the application
                        }
                    }
                }
            }
        });
    }

    /// Spawn a future on the runtime
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    /// Block on the runtime to ensure all tasks complete (for testing)
    pub fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}

impl<M> Default for CommandExecutor<M>
where
    M: Clone + Send + 'static,
{
    fn default() -> Self {
        Self::new().expect("Failed to create runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{AsyncTestHarness, CmdTestExt};
    use hojicha_core::commands;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Inc,
        Dec,
        Text(String),
    }

    #[test]
    fn test_execute_custom_command() {
        // Using AsyncTestHarness for cleaner testing
        let harness = AsyncTestHarness::new();
        let cmd = commands::custom(|| Some(TestMsg::Inc));
        
        let messages = harness.execute_command(cmd);
        assert_eq!(messages, vec![TestMsg::Inc]);
    }
    
    #[test]
    fn test_execute_custom_command_raw() {
        // Keep raw test to verify CommandExecutor directly
        let executor = CommandExecutor::<TestMsg>::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let cmd = commands::custom(|| Some(TestMsg::Inc));
        executor.execute(cmd, tx);

        // Give async task time to execute
        std::thread::sleep(Duration::from_millis(10));

        let event = rx.try_recv().unwrap();
        assert_eq!(event, Event::User(TestMsg::Inc));
    }

    #[test]
    fn test_execute_quit_command() {
        let executor = CommandExecutor::<TestMsg>::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let cmd: Cmd<TestMsg> = commands::quit();
        executor.execute(cmd, tx);

        let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(event, Event::Quit);
    }

    #[test]
    fn test_execute_batch_commands() {
        // Using AsyncTestHarness for cleaner testing
        let harness = AsyncTestHarness::new();
        
        let batch = commands::batch(vec![
            commands::custom(|| Some(TestMsg::Inc)),
            commands::custom(|| Some(TestMsg::Dec)),
            commands::custom(|| Some(TestMsg::Text("test".to_string()))),
        ]);

        let messages = harness.execute_command(batch);
        
        assert_eq!(messages.len(), 3);
        assert!(messages.contains(&TestMsg::Inc));
        assert!(messages.contains(&TestMsg::Dec));
        assert!(messages.contains(&TestMsg::Text("test".to_string())));
    }

    #[test]
    fn test_execute_none_command() {
        let executor = CommandExecutor::<TestMsg>::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        // Cmd::none() returns Option<Cmd>, which is None
        // So we test with a cmd that does nothing
        let cmd: Cmd<TestMsg> = commands::custom(|| None);
        executor.execute(cmd, tx);

        // Give async task time to execute
        std::thread::sleep(Duration::from_millis(10));

        // Should not receive any event
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_execute_tick_command() {
        // Using AsyncTestHarness for cleaner testing
        let harness = AsyncTestHarness::new();
        let cmd = commands::tick(Duration::from_millis(10), || TestMsg::Inc);
        
        let messages = harness.execute_command(cmd);
        assert_eq!(messages, vec![TestMsg::Inc]);
    }
    
    #[test]
    fn test_execute_tick_command_raw() {
        // Keep raw test to verify CommandExecutor directly
        let executor = CommandExecutor::<TestMsg>::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let cmd = commands::tick(Duration::from_millis(10), || TestMsg::Inc);
        executor.execute(cmd, tx);

        // Wait for tick
        let event = rx.recv_timeout(Duration::from_millis(50)).unwrap();
        if let Event::User(msg) = event {
            assert_eq!(msg, TestMsg::Inc);
        } else {
            panic!("Expected User event");
        }
    }

    #[test]
    fn test_execute_sequence() {
        // Using AsyncTestHarness for cleaner testing
        let harness = AsyncTestHarness::new();
        
        let seq = commands::sequence(vec![
            commands::custom(|| Some(TestMsg::Inc)),
            commands::custom(|| Some(TestMsg::Dec)),
        ]);

        let messages = harness.execute_and_wait(seq, Duration::from_millis(50));
        
        // Sequence should maintain order
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], TestMsg::Inc);
        assert_eq!(messages[1], TestMsg::Dec);
    }

    #[test]
    fn test_multiple_executors() {
        let executor1 = CommandExecutor::<TestMsg>::new().unwrap();
        let executor2 = CommandExecutor::<TestMsg>::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        executor1.execute(commands::custom(|| Some(TestMsg::Inc)), tx.clone());
        executor2.execute(commands::custom(|| Some(TestMsg::Dec)), tx.clone());

        // Give async tasks time to execute
        std::thread::sleep(Duration::from_millis(50));

        let mut events = Vec::new();
        while let Ok(Event::User(msg)) = rx.try_recv() {
            events.push(msg);
        }

        assert_eq!(events.len(), 2);
        assert!(events.contains(&TestMsg::Inc));
        assert!(events.contains(&TestMsg::Dec));
    }
}


