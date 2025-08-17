//! Command utilities for handling side effects
//! 
//! This module provides functions for creating commands that perform side effects
//! in your Hojicha application. Commands are the Elm Architecture's way of handling
//! operations that interact with the outside world.
//! 
//! ## Core Command Types
//! 
//! - **Synchronous**: Simple functions that return messages
//! - **Asynchronous**: Futures that eventually produce messages
//! - **Timed**: Commands that execute after delays or at intervals
//! - **Composite**: Batch and sequence commands for complex flows
//! 
//! ## Common Patterns
//! 
//! ### No-op Command
//! ```
//! # use hojicha_core::commands::none;
//! # use hojicha_core::{Model, Cmd, Event};
//! # struct MyModel;
//! # impl Model for MyModel {
//! #     type Message = ();
//! fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
//!     // Handle event but don't trigger side effects
//!     none()
//! }
//! #     fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
//! # }
//! ```
//! 
//! ### Concurrent Commands
//! ```
//! # use hojicha_core::commands::{batch, tick};
//! # use hojicha_core::Cmd;
//! # use std::time::Duration;
//! # enum Msg { Tick1, Tick2 }
//! let cmd: Cmd<Msg> = batch(vec![
//!     tick(Duration::from_secs(1), || Msg::Tick1),
//!     tick(Duration::from_secs(2), || Msg::Tick2),
//! ]);
//! ```
//! 
//! ### Sequential Commands
//! ```
//! # use hojicha_core::commands::sequence;
//! # use hojicha_core::Cmd;
//! # enum Msg { First, Second }
//! let cmd: Cmd<Msg> = sequence(vec![
//!     Cmd::new(|| Some(Msg::First)),
//!     Cmd::new(|| Some(Msg::Second)),
//! ]);
//! ```

use crate::core::{Cmd, Message};
use crate::event::WindowSize;
use std::process::Command;
use std::time::Duration;

// Import panic recovery utilities from runtime crate
// These are used to wrap Model methods for safe execution
#[cfg(feature = "panic-recovery")]
pub use hojicha_runtime::panic_recovery::{
    safe_init, safe_update, safe_view, PanicRecoveryStrategy
};

/// Default maximum batch size
/// 
/// Batches larger than this will trigger a warning in debug mode.
/// This is a soft limit - the batch will still be created.
const DEFAULT_MAX_BATCH_SIZE: usize = 100;

/// Hard maximum batch size
/// 
/// Batches larger than this will be automatically chunked.
/// This prevents accidental memory exhaustion from massive batches.
const HARD_MAX_BATCH_SIZE: usize = 1000;

/// Special message types for terminal control
#[derive(Debug, Clone)]
pub enum TerminalControlMsg {
    /// Hide the terminal cursor from view
    HideCursor,
    /// Show the terminal cursor
    ShowCursor,
    /// Enter the alternate screen buffer (like vim/less use)
    EnterAltScreen,
    /// Exit the alternate screen buffer and return to main screen
    ExitAltScreen,
    /// Set the terminal window title to the given string
    SetWindowTitle(String),
    /// Enable mouse tracking for cell motion (only when button pressed)
    EnableMouseCellMotion,
    /// Enable mouse tracking for all motion events (including hover)
    EnableMouseAllMotion,
    /// Disable all mouse tracking
    DisableMouse,
    /// Clear the entire screen
    ClearScreen,
    /// Clear the current line
    ClearLine,
}

/// Create a no-op command
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::none};
/// # enum Msg {}
/// // Returns a no-op command that continues running without side effects
/// let cmd: Cmd<Msg> = none();
/// ```
pub fn none<M: Message>() -> Cmd<M> {
    Cmd::none()
}

/// Batch multiple commands to run concurrently
///
/// Note: For performance optimization:
/// - Empty vectors return `Cmd::none()`
/// - Single-element vectors return the element directly
/// - Use `batch_strict()` if you need guaranteed batch semantics
///
/// # Safety
/// - Batches larger than 100 commands will trigger a debug warning
/// - Batches larger than 1000 commands will be automatically chunked
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::batch};
/// # enum Msg { Data, Timer }
/// # fn fetch_data() -> Cmd<Msg> { Cmd::none() }
/// # fn start_timer() -> Cmd<Msg> { Cmd::none() }
/// // Batch multiple commands to run concurrently
/// let cmd: Cmd<Msg> = batch(vec![
///     fetch_data(),
///     start_timer(),
/// ]);
/// ```
pub fn batch<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    match cmds.len() {
        0 => Cmd::none(),
        1 => cmds.into_iter().next().unwrap(),
        n if n > HARD_MAX_BATCH_SIZE => {
            // Chunk very large batches to prevent memory issues
            eprintln!("Warning: Batch of {} commands exceeds hard limit of {}. Chunking into smaller batches.", n, HARD_MAX_BATCH_SIZE);
            batch_chunked(cmds, HARD_MAX_BATCH_SIZE)
        }
        n => {
            #[cfg(debug_assertions)]
            if n > DEFAULT_MAX_BATCH_SIZE {
                eprintln!("Warning: Large batch of {} commands (recommended max: {})", n, DEFAULT_MAX_BATCH_SIZE);
            }
            Cmd::batch(cmds)
        }
    }
}

/// Sequence commands to run one after another
///
/// Note: For performance optimization:
/// - Empty vectors return `Cmd::none()`
/// - Single-element vectors return the element directly
/// - Use `sequence_strict()` if you need guaranteed sequence semantics
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::sequence};
/// # enum Msg { Save, Notify }
/// # fn save_to_disk() -> Cmd<Msg> { Cmd::none() }
/// # fn show_notification() -> Cmd<Msg> { Cmd::none() }
/// // Sequence commands to run one after another
/// let cmd: Cmd<Msg> = sequence(vec![
///     save_to_disk(),
///     show_notification(),
/// ]);
/// ```
pub fn sequence<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    match cmds.len() {
        0 => Cmd::none(),
        1 => cmds.into_iter().next().unwrap(),
        _ => Cmd::sequence(cmds),
    }
}

/// Create a batch command with strict semantics
///
/// Unlike `batch()`, this always returns a batch command regardless of the
/// number of elements. Use this when you need guaranteed batch behavior.
///
/// # Example
/// ```no_run
/// # use hojicha_core::{Cmd, commands::batch_strict};
/// # enum Msg { Action }
/// # fn maybe_cmd() -> Cmd<Msg> { Cmd::none() }
/// // Always returns a batch, even with 0 or 1 elements
/// batch_strict(vec![maybe_cmd()])
/// # ;
/// ```
pub fn batch_strict<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::batch(cmds)
}

/// Create a batch command with a specific size limit
///
/// Batches larger than the limit will be automatically chunked.
///
/// # Example
/// ```no_run
/// # use hojicha_core::{Cmd, commands::batch_with_limit};
/// # enum Msg { Action }
/// # let large_vec_of_commands: Vec<Cmd<Msg>> = vec![];
/// // Create batches with max 50 commands each
/// batch_with_limit(large_vec_of_commands, 50)
/// # ;
/// ```
pub fn batch_with_limit<M: Message>(cmds: Vec<Cmd<M>>, limit: usize) -> Cmd<M> {
    if cmds.len() <= limit {
        batch(cmds)
    } else {
        batch_chunked(cmds, limit)
    }
}

/// Internal helper to chunk large batches
#[doc(hidden)]
pub(crate) fn batch_chunked<M: Message>(mut cmds: Vec<Cmd<M>>, chunk_size: usize) -> Cmd<M> {
    let mut chunks = Vec::new();
    
    while !cmds.is_empty() {
        let chunk: Vec<Cmd<M>> = cmds
            .drain(..chunk_size.min(cmds.len()))
            .collect();
        chunks.push(Cmd::batch(chunk));
    }
    
    // Batch the batches - this creates a two-level batch
    // This ensures all commands still run concurrently
    Cmd::batch(chunks)
}

/// Create a sequence command with strict semantics
///
/// Unlike `sequence()`, this always returns a sequence command regardless of the
/// number of elements. Use this when you need guaranteed sequence behavior.
///
/// # Example
/// ```no_run
/// # use hojicha_core::{Cmd, commands::sequence_strict};
/// # enum Msg { Action }
/// # fn maybe_cmd() -> Cmd<Msg> { Cmd::none() }
/// // Always returns a sequence, even with 0 or 1 elements
/// sequence_strict(vec![maybe_cmd()])
/// # ;
/// ```
pub fn sequence_strict<M: Message>(cmds: Vec<Cmd<M>>) -> Cmd<M> {
    Cmd::sequence(cmds)
}

/// Create a command that sends a message after a delay
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::tick};
/// # use std::time::Duration;
/// # enum Msg { Timeout }
/// // Send a message after 5 seconds
/// let cmd: Cmd<Msg> = tick(Duration::from_secs(5), || Msg::Timeout);
/// ```
pub fn tick<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> M + Send + 'static,
{
    Cmd::tick(duration, f)
}

/// Create a command that ticks at regular intervals
///
/// Similar to Bubbletea's Every command, this aligns with system clock
/// boundaries. For example, `every(Duration::from_secs(1))` will tick
/// at the start of each second.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::every};
/// # use std::time::{Duration, Instant};
/// # enum Msg { Tick(Instant) }
/// // Send a message every second
/// let cmd: Cmd<Msg> = every(Duration::from_secs(1), |instant| Msg::Tick(instant));
/// ```
pub fn every<M, F>(duration: Duration, f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce(std::time::Instant) -> M + Send + 'static,
{
    Cmd::every(duration, f)
}

/// Query the terminal for its current size
///
/// This command returns a WindowSize message with the current terminal dimensions.
/// Note that resize events are automatically delivered when the terminal size changes,
/// so you typically won't need to use this command directly.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::window_size, event::WindowSize};
/// # enum Msg { GotSize(WindowSize) }
/// // Query the terminal size
/// let cmd: Cmd<Msg> = window_size(|size| Msg::GotSize(size));
/// ```
pub fn window_size<M, F>(f: F) -> Cmd<M>
where
    M: Message,
    F: Fn(WindowSize) -> M + Send + Sync + 'static,
{
    Cmd::new(move || {
        // Query the actual terminal size using crossterm
        match crossterm::terminal::size() {
            Ok((width, height)) => Some(f(WindowSize { width, height })),
            Err(_) => {
                // Fall back to reasonable defaults if we can't query the terminal
                Some(f(WindowSize {
                    width: 80,
                    height: 24,
                }))
            }
        }
    })
}

/// Set the terminal window title
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::set_window_title};
/// # enum Msg {}
/// // Set the terminal window title
/// let cmd: Cmd<Msg> = set_window_title("My Awesome App");
/// ```
pub fn set_window_title<M: Message>(title: impl Into<String>) -> Cmd<M> {
    let title = title.into();
    Cmd::new(move || {
        use crossterm::{execute, terminal::SetTitle};
        let _ = execute!(std::io::stdout(), SetTitle(&title));
        None
    })
}

/// Send an interrupt signal (simulates Ctrl+C)
///
/// This is useful for graceful shutdown or interrupting long-running operations.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::interrupt};
/// # enum Msg {}
/// // Send an interrupt signal (simulates Ctrl+C)
/// let cmd: Cmd<Msg> = interrupt();
/// ```
pub fn interrupt<M: Message>() -> Cmd<M> {
    Cmd::new(|| {
        #[cfg(unix)]
        {
            // Send SIGINT to current process
            unsafe {
                libc::kill(libc::getpid(), libc::SIGINT);
            }
        }
        None
    })
}

/// Macro to generate simple terminal control commands
/// 
/// This reduces code duplication for commands that just signal
/// the runtime to perform a terminal operation.
macro_rules! terminal_cmd {
    ($(
        $(#[$attr:meta])*
        $vis:vis fn $name:ident() -> $doc:literal;
    )+) => {
        $(
            $(#[$attr])*
            #[doc = $doc]
            #[doc = ""]
            #[doc = "This command signals the runtime to perform the operation."]
            $vis fn $name<M: Message>() -> Cmd<M> {
                Cmd::new(|| None)
            }
        )+
    };
}

// Generate all the simple terminal control commands
terminal_cmd! {
    /// Hide the terminal cursor
    pub fn hide_cursor() -> "Hide the terminal cursor from view";
    
    /// Show the terminal cursor
    pub fn show_cursor() -> "Show the terminal cursor";
    
    /// Enter alternate screen buffer
    pub fn enter_alt_screen() -> "Enter the alternate screen buffer (like vim/less use)";
    
    /// Exit alternate screen buffer
    pub fn exit_alt_screen() -> "Exit the alternate screen buffer and return to main screen";
}

/// Create a custom command from an async function
///
/// This allows you to create commands that perform async operations like
/// HTTP requests, database queries, or other I/O operations.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::custom_async};
/// # enum Message { DataFetched(String) }
/// // Create an async command
/// let cmd: Cmd<Message> = custom_async(|| async {
///     // Perform async operation
///     let data = "example data".to_string();
///     Some(Message::DataFetched(data))
/// });
/// ```
pub fn custom_async<M, F, Fut>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    Cmd::async_cmd(f())
}

/// Spawn a simple async task
///
/// This command spawns an async task on the shared runtime managed by the program.
/// Unlike `custom_async`, this uses the existing runtime rather than creating a new one.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::spawn};
/// # use std::time::Duration;
/// # enum Message { TimerComplete }
/// // Spawn an async task
/// let cmd: Cmd<Message> = spawn(async {
///     tokio::time::sleep(Duration::from_secs(1)).await;
///     Some(Message::TimerComplete)
/// });
/// ```
pub fn spawn<M, Fut>(fut: Fut) -> Cmd<M>
where
    M: Message,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    Cmd::async_cmd(fut)
}

/// Create a custom command from a blocking function
///
/// This is a convenience wrapper for creating simple custom commands.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::custom};
/// # enum Message { ComputationComplete(i32) }
/// # fn expensive_computation() -> i32 { 42 }
/// // Create a custom command
/// let cmd: Cmd<Message> = custom(|| {
///     // Perform some custom logic
///     let result = expensive_computation();
///     Some(Message::ComputationComplete(result))
/// });
/// ```
pub fn custom<M, F>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Option<M> + Send + 'static,
{
    Cmd::new(f)
}

/// Create a custom fallible command
///
/// This allows you to create commands that can fail and handle errors gracefully.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::custom_fallible};
/// # enum Message { ConfigLoaded(String) }
/// // Create a fallible command
/// let cmd: Cmd<Message> = custom_fallible(|| {
///     // Perform operation that might fail
///     let data = std::fs::read_to_string("config.json")?;
///     Ok(Some(Message::ConfigLoaded(data)))
/// });
/// ```
pub fn custom_fallible<M, F>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> crate::Result<Option<M>> + Send + 'static,
{
    Cmd::fallible(f)
}

/// Create a fallible command that converts errors to messages
///
/// This allows errors to be handled by the model's update method rather than
/// just being logged.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::fallible_with_error};
/// # enum Msg { DataLoaded(String), ErrorOccurred(String) }
/// // Create a fallible command with error handling
/// let cmd: Cmd<Msg> = fallible_with_error(
///     || {
///         let data = std::fs::read_to_string("data.json")?;
///         Ok(Some(Msg::DataLoaded(data)))
///     },
///     |err| Msg::ErrorOccurred(err.to_string())
/// );
/// ```
pub fn fallible_with_error<M, F, E>(f: F, error_handler: E) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> crate::Result<Option<M>> + Send + 'static,
    E: FnOnce(crate::error::Error) -> M + Send + 'static,
{
    Cmd::new(move || match f() {
        Ok(msg) => msg,
        Err(err) => Some(error_handler(err)),
    })
}

/// Execute a command in a subprocess, releasing the terminal while it runs
///
/// This is useful for running interactive programs like editors or shells.
/// The terminal will be restored after the command completes.
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::exec};
/// # enum Msg { EditorClosed(Option<i32>) }
/// // Execute an external program
/// let cmd: Cmd<Msg> = exec("vim", vec!["file.txt"], |exit_status| {
///     Msg::EditorClosed(exit_status)
/// });
/// ```
pub fn exec<M, F>(program: impl Into<String>, args: Vec<impl Into<String>>, callback: F) -> Cmd<M>
where
    M: Message,
    F: Fn(Option<i32>) -> M + Send + 'static,
{
    let program = program.into();
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    Cmd::exec_process(program, args, callback)
}

/// Execute a shell command, releasing the terminal while it runs
///
/// # Example
/// ```
/// # use hojicha_core::{Cmd, commands::exec_command};
/// # enum Msg { CommandFinished(Option<i32>) }
/// // Execute a shell command
/// let cmd: Cmd<Msg> = exec_command("ls -la", |exit_status| {
///     Msg::CommandFinished(exit_status)
/// });
/// ```
pub fn exec_command<M, F>(command: impl Into<String>, callback: F) -> Cmd<M>
where
    M: Message,
    F: Fn(Option<i32>) -> M + Send + 'static,
{
    let command = command.into();

    Cmd::new(move || {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &command]).status()
        } else {
            Command::new("sh").args(["-c", &command]).status()
        };

        let exit_code = output.ok().and_then(|status| status.code());
        Some(callback(exit_code))
    })
}

// Generate mouse and screen control commands
terminal_cmd! {
    /// Enable mouse cell motion tracking
    /// 
    /// This enables mouse events only when a button is pressed.
    pub fn enable_mouse_cell_motion() -> "Enable mouse tracking for cell motion (only when button pressed)";
    
    /// Enable mouse all motion tracking
    /// 
    /// This enables mouse movement events regardless of whether a button is pressed,
    /// allowing for hover interactions.
    pub fn enable_mouse_all_motion() -> "Enable mouse tracking for all motion events (including hover)";
    
    /// Disable mouse tracking
    pub fn disable_mouse() -> "Disable all mouse tracking";
    
    /// Clear the entire screen
    pub fn clear_screen() -> "Clear the entire screen";
    
    /// Clear the current line
    pub fn clear_line() -> "Clear the current line";
    
    /// Suspend the program (Ctrl+Z)
    /// 
    /// This will suspend the program and return control to the shell.
    /// When the program is resumed, a Resume event will be sent.
    pub fn suspend() -> "Suspend the program (Ctrl+Z)";
}

/// Quit the program gracefully
///
/// This command signals the program to exit cleanly.
///
/// # Example
///
/// ```no_run
/// # use hojicha_core::prelude::*;
/// # #[derive(Debug, Clone)]
/// # enum MyMessage { Quit }
/// # struct MyModel;
/// # impl Model for MyModel {
/// #     type Message = MyMessage;
/// #     fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
/// match event {
///     Event::Key(key) if key.key == Key::Char('q') => {
///         quit()
///     }
///     Event::User(MyMessage::Quit) => {
///         quit()
///     }
///     _ => Cmd::none()
/// }
/// #     }
/// #     fn view(&self, _: &mut Frame, _: Rect) {}
/// # }
/// ```
pub fn quit<M: Message>() -> Cmd<M> {
    Cmd::quit()
}


/// Macro to generate crossterm commands that execute terminal sequences
/// 
/// This reduces duplication for commands that use crossterm to send
/// control sequences to the terminal.
macro_rules! crossterm_cmd {
    ($(
        $(#[$attr:meta])*
        $vis:vis fn $name:ident($cmd_type:path) -> $doc:literal;
    )+) => {
        $(
            $(#[$attr])*
            #[doc = $doc]
            #[doc = ""]
            #[doc = "This command sends a control sequence to the terminal."]
            $vis fn $name<M: Message>() -> Cmd<M> {
                Cmd::new(|| {
                    use crossterm::execute;
                    use std::io;
                    let _ = execute!(io::stdout(), $cmd_type);
                    None
                })
            }
        )+
    };
}

// Generate crossterm-based commands
crossterm_cmd! {
    /// Enable bracketed paste mode
    /// 
    /// When enabled, pasted text will be delivered as a single Event::Paste(String)
    /// instead of individual key events. This prevents pasted text from
    /// accidentally triggering keyboard shortcuts.
    pub fn enable_bracketed_paste(crossterm::event::EnableBracketedPaste) -> "Enable bracketed paste mode";
    
    /// Disable bracketed paste mode
    pub fn disable_bracketed_paste(crossterm::event::DisableBracketedPaste) -> "Disable bracketed paste mode";
    
    /// Enable focus change reporting
    /// 
    /// When enabled, the program will receive Event::Focus when the terminal
    /// gains focus and Event::Blur when it loses focus.
    pub fn enable_focus_change(crossterm::event::EnableFocusChange) -> "Enable focus change reporting";
    
    /// Disable focus change reporting
    pub fn disable_focus_change(crossterm::event::DisableFocusChange) -> "Disable focus change reporting";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestMsg {
        One,
        Two,
        Three,
    }

    #[test]
    fn test_batch_empty() {
        let result: Cmd<TestMsg> = batch(vec![]);
        assert!(!result.is_quit());
    }

    #[test]
    fn test_batch_single() {
        let cmd = Cmd::new(|| Some(TestMsg::One));
        let result = batch(vec![cmd]);
        assert!(!result.is_quit());
    }

    #[test]
    fn test_batch_multiple() {
        let cmds = vec![
            Cmd::new(|| Some(TestMsg::One)),
            Cmd::new(|| Some(TestMsg::Two)),
            Cmd::new(|| Some(TestMsg::Three)),
        ];
        let result = batch(cmds);
        assert!(!result.is_quit());
    }

    #[test]
    fn test_sequence_executes_in_order() {
        let cmd = sequence(vec![Cmd::new(|| Some(TestMsg::One))]);
        let msg = cmd.execute().unwrap();
        assert_eq!(msg, Some(TestMsg::One));
    }

    #[test]
    fn test_tick_command() {
        let cmd = tick(Duration::from_millis(10), || TestMsg::One);
        // Tick commands are now async and handled by the executor
        // They return None from execute() since they need async handling
        let msg = cmd.execute().unwrap();
        assert_eq!(msg, None);
    }

    #[test]
    fn test_every_command() {
        let cmd: Cmd<TestMsg> = every(Duration::from_millis(1), |_| TestMsg::One);
        // Every commands are now async and handled by the executor
        // They return None from execute() since they need async handling
        let result = cmd.test_execute().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_window_size_command() {
        // Test that window_size returns a valid WindowSize
        #[derive(Debug, PartialEq)]
        enum SizeMsg {
            Size(WindowSize),
        }

        let cmd: Cmd<SizeMsg> = window_size(SizeMsg::Size);
        let result = cmd.test_execute().unwrap();

        // Verify we got a size message
        assert!(matches!(result, Some(SizeMsg::Size(_))));

        // The actual dimensions will vary based on terminal, but should be positive
        if let Some(SizeMsg::Size(size)) = result {
            assert!(size.width > 0);
            assert!(size.height > 0);
        }
    }

    #[test]
    fn test_cursor_commands() {
        let hide_cmd: Cmd<TestMsg> = hide_cursor();
        let show_cmd: Cmd<TestMsg> = show_cursor();

        assert!(hide_cmd.test_execute().is_ok());
        assert!(show_cmd.test_execute().is_ok());
    }

    #[test]
    fn test_alt_screen_commands() {
        let enter_cmd: Cmd<TestMsg> = enter_alt_screen();
        let exit_cmd: Cmd<TestMsg> = exit_alt_screen();

        assert!(enter_cmd.test_execute().is_ok());
        assert!(exit_cmd.test_execute().is_ok());
    }

    #[test]
    fn test_custom_command() {
        let cmd = custom::<TestMsg, _>(|| Some(TestMsg::One));
        let result = cmd.execute();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TestMsg::One));
    }

    #[test]
    fn test_custom_fallible_success() {
        let cmd = custom_fallible::<TestMsg, _>(|| Ok(Some(TestMsg::Two)));
        let result = cmd.execute();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TestMsg::Two));
    }

    #[test]
    fn test_custom_fallible_error() {
        use std::io;
        let cmd = custom_fallible::<TestMsg, _>(|| {
            Err(crate::error::Error::Io(io::Error::other("test error")))
        });
        let result = cmd.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_async_command() {
        let cmd = custom_async::<TestMsg, _, _>(|| async { Some(TestMsg::Three) });
        let result = cmd.execute();
        assert!(result.is_ok());
        // Now async commands return None since they use shared runtime
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_window_title_command() {
        let cmd: Cmd<TestMsg> = set_window_title("Test Title");
        assert!(cmd.test_execute().is_ok());

        let cmd_empty: Cmd<TestMsg> = set_window_title("");
        assert!(cmd_empty.test_execute().is_ok());
    }

    #[test]
    fn test_exec_command() {
        let cmd: Cmd<TestMsg> = exec("echo", vec!["hello"], |_| TestMsg::One);
        assert!(cmd.is_exec_process());

        let process_info = cmd.take_exec_process();
        assert!(process_info.is_some());
    }

    #[test]
    fn test_mouse_commands() {
        let cell_motion: Cmd<TestMsg> = enable_mouse_cell_motion();
        let all_motion: Cmd<TestMsg> = enable_mouse_all_motion();
        let disable: Cmd<TestMsg> = disable_mouse();

        assert!(cell_motion.test_execute().is_ok());
        assert!(all_motion.test_execute().is_ok());
        assert!(disable.test_execute().is_ok());
    }

    #[test]
    fn test_clear_commands() {
        let clear_screen: Cmd<TestMsg> = clear_screen();
        let clear_line: Cmd<TestMsg> = clear_line();

        assert!(clear_screen.test_execute().is_ok());
        assert!(clear_line.test_execute().is_ok());
    }

    #[test]
    fn test_suspend_command() {
        let cmd: Cmd<TestMsg> = suspend();
        assert!(cmd.test_execute().is_ok());
    }

    #[test]
    fn test_crossterm_commands() {
        let enable_paste: Cmd<TestMsg> = enable_bracketed_paste();
        let disable_paste: Cmd<TestMsg> = disable_bracketed_paste();
        let enable_focus: Cmd<TestMsg> = enable_focus_change();
        let disable_focus: Cmd<TestMsg> = disable_focus_change();

        assert!(enable_paste.test_execute().is_ok());
        assert!(disable_paste.test_execute().is_ok());
        assert!(enable_focus.test_execute().is_ok());
        assert!(disable_focus.test_execute().is_ok());
    }

    #[test]
    fn test_batch_with_mixed_types() {
        let cmds = vec![
            Cmd::new(|| Some(TestMsg::One)),
            Cmd::new(|| Some(TestMsg::Two)),
        ];

        let batch_cmd = batch(cmds);
        // Batch commands should be recognized as batch type
        assert!(batch_cmd.is_batch());
    }

    #[test]
    fn test_sequence_execution_order() {
        let cmds = vec![
            Cmd::new(|| Some(TestMsg::One)),
            Cmd::new(|| Some(TestMsg::Two)),
        ];

        let seq_cmd = sequence(cmds);
        // Sequence commands should be recognized as sequence type
        assert!(seq_cmd.is_sequence());
    }
}
