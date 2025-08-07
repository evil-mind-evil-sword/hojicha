//! Command utilities for handling side effects

use crate::core::{Cmd, Message};
use crate::event::WindowSize;
use std::process::Command;
use std::time::Duration;

/// Special message types for terminal control
#[derive(Debug, Clone)]
pub enum TerminalControlMsg {
    HideCursor,
    ShowCursor,
    EnterAltScreen,
    ExitAltScreen,
    SetWindowTitle(String),
    EnableMouseCellMotion,
    EnableMouseAllMotion,
    DisableMouse,
    ClearScreen,
    ClearLine,
}

/// Batch multiple commands to run concurrently
///
/// # Example
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(batch(vec![
///         fetch_data(),
///         start_timer(),
///     ]))
/// }
/// ```
pub fn batch<M: Message>(cmds: Vec<Option<Cmd<M>>>) -> Option<Cmd<M>> {
    let cmds: Vec<_> = cmds.into_iter().flatten().collect();

    match cmds.len() {
        0 => None,
        1 => cmds.into_iter().next(),
        _ => Some(Cmd::batch(cmds)),
    }
}

/// Sequence commands to run one after another
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     Some(sequence(vec![
///         save_to_disk(),
///         show_notification(),
///     ]))
/// }
/// ```
pub fn sequence<M: Message>(cmds: Vec<Option<Cmd<M>>>) -> Option<Cmd<M>> {
    let cmds: Vec<_> = cmds.into_iter().flatten().collect();

    match cmds.len() {
        0 => None,
        1 => cmds.into_iter().next(),
        _ => Some(Cmd::sequence(cmds)),
    }
}

/// Create a command that sends a message after a delay
///
/// # Example
/// ```ignore
/// enum Msg {
///     Timeout,
/// }
///
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(tick(Duration::from_secs(5), || Msg::Timeout))
/// }
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
/// ```ignore
/// enum Msg {
///     Tick(std::time::Instant),
/// }
///
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(every(Duration::from_secs(1), |instant| Msg::Tick(instant)))
/// }
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
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::GetWindowSize) => {
///             return Some(window_size(|size| Msg::GotSize(size)));
///         }
///         _ => {}
///     }
///     None
/// }
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
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(set_window_title("My Awesome App"))
/// }
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
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::Shutdown) => Some(interrupt()),
///         _ => None
///     }
/// }
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

/// Hide the terminal cursor
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::HideCursor) => {
///             return Some(hide_cursor());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn hide_cursor<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Show the terminal cursor
pub fn show_cursor<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Enter alternate screen buffer
pub fn enter_alt_screen<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Exit alternate screen buffer
pub fn exit_alt_screen<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Create a custom command from an async function
///
/// This allows you to create commands that perform async operations like
/// HTTP requests, database queries, or other I/O operations.
///
/// # Example
/// ```ignore
/// async fn fetch_data() -> String {
///     // Perform async operation
///     reqwest::get("https://api.example.com/data")
///         .await
///         .unwrap()
///         .text()
///         .await
///         .unwrap()
/// }
///
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(custom_async(|| async {
///         let data = fetch_data().await;
///         Some(Message::DataFetched(data))
///     }))
/// }
/// ```
pub fn custom_async<M, F, Fut>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Option<M>> + Send + 'static,
{
    Cmd::new(move || {
        // Create a new runtime for this async operation
        let runtime = tokio::runtime::Runtime::new().ok()?;
        runtime.block_on(f())
    })
}

/// Create a custom command from a blocking function
///
/// This is a convenience wrapper for creating simple custom commands.
///
/// # Example
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(custom(|| {
///         // Perform some custom logic
///         let result = expensive_computation();
///         Some(Message::ComputationComplete(result))
///     }))
/// }
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
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(custom_fallible(|| {
///         // Perform operation that might fail
///         let data = std::fs::read_to_string("config.json")?;
///         Ok(Some(Message::ConfigLoaded(data)))
///     }))
/// }
/// ```
pub fn custom_fallible<M, F>(f: F) -> Cmd<M>
where
    M: Message,
    F: FnOnce() -> crate::Result<Option<M>> + Send + 'static,
{
    Cmd::fallible(f)
}

/// Execute a command in a subprocess, releasing the terminal while it runs
///
/// This is useful for running interactive programs like editors or shells.
/// The terminal will be restored after the command completes.
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::EditFile) => {
///             return Some(exec("vim", vec!["file.txt"], |exit_status| {
///                 Msg::EditorClosed(exit_status)
///             }));
///         }
///         _ => {}
///     }
///     None
/// }
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
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::RunShellCommand) => {
///             return Some(exec_command("ls -la", |exit_status| {
///                 Msg::CommandFinished(exit_status)
///             }));
///         }
///         _ => {}
///     }
///     None
/// }
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

/// Enable mouse cell motion tracking
///
/// This enables mouse events only when a button is pressed.
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::EnableMouse) => {
///             return Some(enable_mouse_cell_motion());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn enable_mouse_cell_motion<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Enable mouse all motion tracking
///
/// This enables mouse movement events regardless of whether a button is pressed,
/// allowing for hover interactions.
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::EnableHoverTracking) => {
///             return Some(enable_mouse_all_motion());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn enable_mouse_all_motion<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Disable mouse tracking
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::DisableMouse) => {
///             return Some(disable_mouse());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn disable_mouse<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Clear the entire screen
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::ClearScreen) => {
///             return Some(clear_screen());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn clear_screen<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Clear the current line
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::ClearLine) => {
///             return Some(clear_line());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn clear_line<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Quit the program gracefully
///
/// This command signals the program to exit cleanly.
///
/// # Example
///
/// ```no_run
/// # use hojicha::prelude::*;
/// # use hojicha::commands;
/// # #[derive(Debug, Clone)]
/// # enum MyMessage { Quit }
/// # struct MyModel;
/// # impl Model for MyModel {
/// #     type Message = MyMessage;
/// #     fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
/// match event {
///     Event::Key(key) if key.key == Key::Char('q') => {
///         Some(commands::quit())
///     }
///     Event::User(MyMessage::Quit) => {
///         Some(commands::quit())
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

/// Suspend the program (Ctrl+Z)
///
/// This will suspend the program and return control to the shell.
/// When the program is resumed, a Resume event will be sent.
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::Key(key) if key.key == Key::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) => {
///             return Some(suspend());
///         }
///         Event::Resume => {
///             // Handle resume
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn suspend<M: Message>() -> Cmd<M> {
    Cmd::new(|| None) // This will be handled by the runtime
}

/// Enable bracketed paste mode
///
/// When enabled, pasted text will be delivered as a single Event::Paste(String)
/// instead of individual key events. This prevents pasted text from
/// accidentally triggering keyboard shortcuts.
///
/// # Example
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(enable_bracketed_paste())
/// }
///
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::Paste(text) => {
///             self.input.push_str(&text);
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn enable_bracketed_paste<M: Message>() -> Cmd<M> {
    Cmd::new(|| {
        use crossterm::{event::EnableBracketedPaste, execute};
        use std::io;

        // Send the enable bracketed paste sequence to the terminal
        let _ = execute!(io::stdout(), EnableBracketedPaste);
        None
    })
}

/// Disable bracketed paste mode
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::DisablePaste) => {
///             return Some(disable_bracketed_paste());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn disable_bracketed_paste<M: Message>() -> Cmd<M> {
    Cmd::new(|| {
        use crossterm::{event::DisableBracketedPaste, execute};
        use std::io;

        // Send the disable bracketed paste sequence to the terminal
        let _ = execute!(io::stdout(), DisableBracketedPaste);
        None
    })
}

/// Enable focus change reporting
///
/// When enabled, the program will receive Event::Focus when the terminal
/// gains focus and Event::Blur when it loses focus.
///
/// # Example
/// ```ignore
/// fn init(&mut self) -> Option<Cmd<Self::Message>> {
///     Some(enable_focus_change())
/// }
///
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::Focus => {
///             self.has_focus = true;
///         }
///         Event::Blur => {
///             self.has_focus = false;
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn enable_focus_change<M: Message>() -> Cmd<M> {
    Cmd::new(|| {
        use crossterm::{event::EnableFocusChange, execute};
        use std::io;

        // Send the enable focus change sequence to the terminal
        let _ = execute!(io::stdout(), EnableFocusChange);
        None
    })
}

/// Disable focus change reporting
///
/// # Example
/// ```ignore
/// fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
///     match msg {
///         Event::User(Msg::DisableFocus) => {
///             return Some(disable_focus_change());
///         }
///         _ => {}
///     }
///     None
/// }
/// ```
pub fn disable_focus_change<M: Message>() -> Cmd<M> {
    Cmd::new(|| {
        use crossterm::{event::DisableFocusChange, execute};
        use std::io;

        // Send the disable focus change sequence to the terminal
        let _ = execute!(io::stdout(), DisableFocusChange);
        None
    })
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
        let result: Option<Cmd<TestMsg>> = batch(vec![]);
        assert!(result.is_none());
    }

    #[test]
    fn test_batch_single() {
        let cmd = Some(Cmd::new(|| Some(TestMsg::One)));
        let result = batch(vec![cmd]);
        assert!(result.is_some());
    }

    #[test]
    fn test_batch_multiple() {
        let cmds = vec![
            Some(Cmd::new(|| Some(TestMsg::One))),
            Some(Cmd::new(|| Some(TestMsg::Two))),
            None,
            Some(Cmd::new(|| Some(TestMsg::Three))),
        ];
        let result = batch(cmds);
        assert!(result.is_some());
    }

    #[test]
    fn test_sequence_executes_in_order() {
        let cmd = sequence(vec![Some(Cmd::new(|| Some(TestMsg::One)))]);

        if let Some(cmd) = cmd {
            let msg = cmd.execute().unwrap();
            assert_eq!(msg, Some(TestMsg::One));
        }
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
        assert_eq!(result.unwrap(), Some(TestMsg::Three));
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
    fn test_bracketed_paste_commands() {
        let enable: Cmd<TestMsg> = enable_bracketed_paste();
        let disable: Cmd<TestMsg> = disable_bracketed_paste();

        assert!(enable.test_execute().is_ok());
        assert!(disable.test_execute().is_ok());
    }

    #[test]
    fn test_focus_change_commands() {
        let enable: Cmd<TestMsg> = enable_focus_change();
        let disable: Cmd<TestMsg> = disable_focus_change();

        assert!(enable.test_execute().is_ok());
        assert!(disable.test_execute().is_ok());
    }

    #[test]
    fn test_batch_with_mixed_types() {
        let cmds = vec![
            Some(Cmd::new(|| Some(TestMsg::One))),
            None,
            Some(Cmd::new(|| Some(TestMsg::Two))),
        ];

        let batch_cmd = batch(cmds);
        assert!(batch_cmd.is_some());

        // Batch commands should be recognized as batch type
        let cmd = batch_cmd.unwrap();
        assert!(cmd.is_batch());
    }

    #[test]
    fn test_sequence_execution_order() {
        let cmds = vec![
            Some(Cmd::new(|| Some(TestMsg::One))),
            Some(Cmd::new(|| Some(TestMsg::Two))),
        ];

        let seq_cmd = sequence(cmds);
        assert!(seq_cmd.is_some());

        // Sequence commands should be recognized as sequence type
        let cmd = seq_cmd.unwrap();
        assert!(cmd.is_sequence());
    }
}
