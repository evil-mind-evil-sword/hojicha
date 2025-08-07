//! Command execution logic extracted from Program for testability

use crate::core::Cmd;
use crate::event::Event;
use std::sync::mpsc;
use tokio::runtime::Runtime;

/// Executes commands and sends resulting messages
#[derive(Clone)]
pub struct CommandExecutor {
    runtime: std::sync::Arc<Runtime>,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {
            runtime: std::sync::Arc::new(Runtime::new()?),
        })
    }

    /// Execute a command and send the result through the channel
    pub fn execute<M>(&self, cmd: Cmd<M>, tx: mpsc::SyncSender<Event<M>>)
    where
        M: Clone + Send + 'static,
    {
        if cmd.is_noop() {
            // NoOp command - do nothing
            return;
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
                    let msg = callback();
                    let _ = tx_clone.send(Event::User(msg));
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
                    let msg = callback(std::time::Instant::now());
                    let _ = tx_clone.send(Event::User(msg));
                });
            }
        } else {
            // Spawn async task for regular command execution (like Bubbletea's goroutines)
            let tx_clone = tx.clone();
            self.runtime.spawn(async move {
                match cmd.execute() {
                    Ok(Some(msg)) => {
                        let _ = tx_clone.send(Event::User(msg));
                    }
                    Ok(None) => {
                        // Command executed successfully but produced no message
                    }
                    Err(error) => {
                        // Log the error - in a real implementation, this might send an error event
                        eprintln!("Command execution error: {error}");
                    }
                }
            });
        }
    }

    /// Execute a batch of commands concurrently
    pub fn execute_batch<M>(&self, commands: Vec<Cmd<M>>, tx: mpsc::SyncSender<Event<M>>)
    where
        M: Clone + Send + 'static,
    {
        // Spawn all commands concurrently (like Bubbletea's batch)
        for cmd in commands {
            // Each command runs in its own async task
            self.execute(cmd, tx.clone());
        }
    }

    /// Execute a sequence of commands (one after another)
    pub fn execute_sequence<M>(&self, commands: Vec<Cmd<M>>, tx: mpsc::SyncSender<Event<M>>)
    where
        M: Clone + Send + 'static,
    {
        // Spawn async task to execute commands in sequence
        let tx_clone = tx.clone();
        self.runtime.spawn(async move {
            for cmd in commands {
                let tx_inner = tx_clone.clone();

                // Execute the command through the regular execution path
                if cmd.is_tick() {
                    if let Some((duration, callback)) = cmd.take_tick() {
                        tokio::time::sleep(duration).await;
                        let msg = callback();
                        let _ = tx_inner.send(Event::User(msg));
                    }
                } else if cmd.is_every() {
                    if let Some((duration, callback)) = cmd.take_every() {
                        tokio::time::sleep(duration).await;
                        let msg = callback(std::time::Instant::now());
                        let _ = tx_inner.send(Event::User(msg));
                    }
                } else {
                    // Regular command execution
                    match cmd.execute() {
                        Ok(Some(msg)) => {
                            let _ = tx_inner.send(Event::User(msg));
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("Sequence command execution error: {error}");
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

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create runtime")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    enum TestMsg {
        Inc,
        Dec,
        Text(String),
    }

    #[test]
    fn test_execute_custom_command() {
        let executor = CommandExecutor::new().unwrap();
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
        let executor = CommandExecutor::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let cmd: Cmd<TestMsg> = commands::quit();
        executor.execute(cmd, tx);

        let event = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        assert_eq!(event, Event::Quit);
    }

    #[test]
    fn test_execute_batch_commands() {
        let executor = CommandExecutor::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let commands = vec![
            commands::custom(|| Some(TestMsg::Inc)),
            commands::custom(|| Some(TestMsg::Dec)),
            commands::custom(|| Some(TestMsg::Text("test".to_string()))),
        ];

        executor.execute_batch(commands, tx);

        // Give async tasks time to execute
        std::thread::sleep(Duration::from_millis(50));

        // Collect all events
        let mut events = Vec::new();
        while let Ok(event) = rx.try_recv() {
            if let Event::User(msg) = event {
                events.push(msg);
            }
        }

        assert_eq!(events.len(), 3);
        assert!(events.contains(&TestMsg::Inc));
        assert!(events.contains(&TestMsg::Dec));
        assert!(events.contains(&TestMsg::Text("test".to_string())));
    }

    #[test]
    fn test_execute_none_command() {
        let executor = CommandExecutor::new().unwrap();
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
        let executor = CommandExecutor::new().unwrap();
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
        let executor = CommandExecutor::new().unwrap();
        let (tx, rx) = mpsc::sync_channel(10);

        let commands = vec![
            commands::custom(|| Some(TestMsg::Inc)),
            commands::custom(|| Some(TestMsg::Dec)),
        ];

        executor.execute_sequence(commands, tx);

        // Give async tasks time to execute
        std::thread::sleep(Duration::from_millis(50));

        let mut events = Vec::new();
        while let Ok(Event::User(msg)) = rx.try_recv() {
            events.push(msg);
        }

        // Sequence should maintain order
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], TestMsg::Inc);
        assert_eq!(events[1], TestMsg::Dec);
    }

    #[test]
    fn test_multiple_executors() {
        let executor1 = CommandExecutor::new().unwrap();
        let executor2 = CommandExecutor::new().unwrap();
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
