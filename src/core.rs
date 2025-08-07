//! Core traits and types for the Elm Architecture

use crate::Event;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::Debug;

/// Type alias for exec process callback function
type ExecCallback<M> = Box<dyn Fn(Option<i32>) -> M + Send>;

/// Type alias for exec process details
type ExecDetails<M> = (String, Vec<String>, ExecCallback<M>);

/// A message that can be sent to update the model.
///
/// Messages are typically enums that represent different events
/// or state changes in your application.
pub trait Message: Send + 'static {}

// Blanket implementation for all types that meet the bounds
impl<T: Send + 'static> Message for T {}

/// The core trait for your application's model.
///
/// Your model should implement this trait to define how it:
/// - Initializes (`init`)
/// - Updates in response to messages (`update`)
/// - Renders to the screen (`view`)
pub trait Model: Sized {
    /// The type of messages this model can receive
    type Message: Message;

    /// Initialize the model and optionally return a command to run
    ///
    /// Returns:
    /// - `None` - Start the event loop without any initial command
    /// - `Some(cmd)` - Execute the command before starting the event loop
    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        None
    }

    /// Update the model based on a message, optionally returning a command
    ///
    /// Returns:
    /// - `None` - Continue running without executing any command
    /// - `Some(cmd)` - Execute the command (use `commands::quit()` to exit)
    fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>>;

    /// Render the model to the screen
    fn view(&self, frame: &mut Frame, area: Rect);
}

/// A command is an asynchronous operation that produces a message.
///
/// Commands are used for side effects like:
/// - HTTP requests
/// - File I/O
/// - Timers
/// - Any async operation
pub struct Cmd<M: Message> {
    inner: CmdInner<M>,
}

pub(crate) enum CmdInner<M: Message> {
    /// No operation - continue running without doing anything
    NoOp,
    /// A simple function command
    Function(Box<dyn FnOnce() -> Option<M> + Send>),
    /// A function command that can return errors
    Fallible(Box<dyn FnOnce() -> crate::Result<Option<M>> + Send>),
    /// An external process command
    ExecProcess {
        program: String,
        args: Vec<String>,
        callback: ExecCallback<M>,
    },
    /// Quit the program
    Quit,
    /// Execute multiple commands concurrently
    Batch(Vec<Cmd<M>>),
    /// Execute multiple commands sequentially
    Sequence(Vec<Cmd<M>>),
    /// Execute after a delay
    Tick {
        duration: std::time::Duration,
        callback: Box<dyn FnOnce() -> M + Send>,
    },
    /// Execute repeatedly at intervals
    Every {
        duration: std::time::Duration,
        callback: Box<dyn FnOnce(std::time::Instant) -> M + Send>,
    },
}

impl<M: Message> Cmd<M> {
    /// Create a new command from a function
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<M> + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Function(Box::new(f)),
        }
    }

    /// Create a new fallible command that can return errors
    pub fn fallible<F>(f: F) -> Self
    where
        F: FnOnce() -> crate::Result<Option<M>> + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Fallible(Box::new(f)),
        }
    }

    /// Returns a no-op command that continues running without doing anything
    ///
    /// This is a convenience method that returns a command that does nothing
    /// but keeps the program running.
    pub fn none() -> Option<Self> {
        Some(Cmd {
            inner: CmdInner::NoOp,
        })
    }

    /// Create a command that executes an external process
    pub(crate) fn exec_process<F>(program: String, args: Vec<String>, callback: F) -> Self
    where
        F: Fn(Option<i32>) -> M + Send + 'static,
    {
        Cmd {
            inner: CmdInner::ExecProcess {
                program,
                args,
                callback: Box::new(callback),
            },
        }
    }

    /// Create a batch command that executes commands concurrently
    pub(crate) fn batch(cmds: Vec<Cmd<M>>) -> Self {
        Cmd {
            inner: CmdInner::Batch(cmds),
        }
    }

    /// Create a sequence command that executes commands in order
    pub(crate) fn sequence(cmds: Vec<Cmd<M>>) -> Self {
        Cmd {
            inner: CmdInner::Sequence(cmds),
        }
    }

    /// Create a quit command
    pub(crate) fn quit() -> Self {
        Cmd {
            inner: CmdInner::Quit,
        }
    }

    /// Create a tick command
    pub(crate) fn tick<F>(duration: std::time::Duration, callback: F) -> Self
    where
        F: FnOnce() -> M + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Tick {
                duration,
                callback: Box::new(callback),
            },
        }
    }

    /// Create an every command
    pub(crate) fn every<F>(duration: std::time::Duration, callback: F) -> Self
    where
        F: FnOnce(std::time::Instant) -> M + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Every {
                duration,
                callback: Box::new(callback),
            },
        }
    }

    /// Execute the command and return its message
    pub(crate) fn execute(self) -> crate::Result<Option<M>> {
        match self.inner {
            CmdInner::Function(func) => Ok(func()),
            CmdInner::Fallible(func) => func(),
            CmdInner::ExecProcess {
                program,
                args,
                callback,
            } => {
                // In the real implementation, this would be handled by the runtime
                // For now, we'll just run it directly
                use std::process::Command;
                let output = Command::new(&program).args(&args).status();
                let exit_code = output.ok().and_then(|status| status.code());
                Ok(Some(callback(exit_code)))
            }
            CmdInner::NoOp => {
                // NoOp commands don't produce messages, just continue running
                Ok(None)
            }
            CmdInner::Quit => {
                // Quit commands don't produce messages, they're handled specially
                Ok(None)
            }
            CmdInner::Batch(_) | CmdInner::Sequence(_) => {
                // These are handled specially by the CommandExecutor
                Ok(None)
            }
            CmdInner::Tick { .. } | CmdInner::Every { .. } => {
                // These are handled specially by the CommandExecutor with async delays
                Ok(None)
            }
        }
    }

    /// Execute the command and return its message (for testing only)
    #[doc(hidden)]
    pub fn test_execute(self) -> crate::Result<Option<M>> {
        self.execute()
    }

    /// Check if this is an exec process command
    pub(crate) fn is_exec_process(&self) -> bool {
        matches!(self.inner, CmdInner::ExecProcess { .. })
    }

    /// Check if this is a no-op command
    pub(crate) fn is_noop(&self) -> bool {
        matches!(self.inner, CmdInner::NoOp)
    }

    /// Check if this is a quit command
    pub(crate) fn is_quit(&self) -> bool {
        matches!(self.inner, CmdInner::Quit)
    }

    /// Extract exec process details if this is an exec process command
    #[allow(clippy::type_complexity)]
    pub(crate) fn take_exec_process(self) -> Option<ExecDetails<M>> {
        match self.inner {
            CmdInner::ExecProcess {
                program,
                args,
                callback,
            } => Some((program, args, callback)),
            _ => None,
        }
    }

    /// Check if this is a batch command
    pub(crate) fn is_batch(&self) -> bool {
        matches!(self.inner, CmdInner::Batch(_))
    }

    /// Take the batch commands (consumes the command)
    pub(crate) fn take_batch(self) -> Option<Vec<Cmd<M>>> {
        match self.inner {
            CmdInner::Batch(cmds) => Some(cmds),
            _ => None,
        }
    }

    /// Check if this is a sequence command
    pub(crate) fn is_sequence(&self) -> bool {
        matches!(self.inner, CmdInner::Sequence(_))
    }

    /// Take the sequence commands (consumes the command)
    pub(crate) fn take_sequence(self) -> Option<Vec<Cmd<M>>> {
        match self.inner {
            CmdInner::Sequence(cmds) => Some(cmds),
            _ => None,
        }
    }

    pub(crate) fn is_tick(&self) -> bool {
        matches!(self.inner, CmdInner::Tick { .. })
    }

    pub(crate) fn is_every(&self) -> bool {
        matches!(self.inner, CmdInner::Every { .. })
    }

    pub(crate) fn take_tick(self) -> Option<(std::time::Duration, Box<dyn FnOnce() -> M + Send>)> {
        match self.inner {
            CmdInner::Tick { duration, callback } => Some((duration, callback)),
            _ => None,
        }
    }

    pub(crate) fn take_every(
        self,
    ) -> Option<(
        std::time::Duration,
        Box<dyn FnOnce(std::time::Instant) -> M + Send>,
    )> {
        match self.inner {
            CmdInner::Every { duration, callback } => Some((duration, callback)),
            _ => None,
        }
    }
}

impl<M: Message> Debug for Cmd<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cmd").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use ratatui::{layout::Rect, Frame};

    // Test model
    #[derive(Clone)]
    struct Counter {
        value: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    enum Msg {
        Increment,
        Decrement,
        SetValue(i32),
        Noop,
    }

    impl Model for Counter {
        type Message = Msg;

        fn init(&mut self) -> Option<Cmd<Self::Message>> {
            Some(Cmd::new(|| Some(Msg::SetValue(0))))
        }

        fn update(&mut self, msg: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
            if let Event::User(msg) = msg {
                match msg {
                    Msg::Increment => {
                        self.value += 1;
                        Some(Cmd::new(move || Some(Msg::Noop)))
                    }
                    Msg::Decrement => {
                        self.value -= 1;
                        Some(Cmd::new(move || Some(Msg::Noop)))
                    }
                    Msg::SetValue(v) => {
                        self.value = v;
                        None
                    }
                    Msg::Noop => None,
                }
            } else {
                None
            }
        }

        fn view(&self, frame: &mut Frame, area: Rect) {
            frame.render_widget(
                ratatui::widgets::Paragraph::new(format!("Count: {}", self.value)),
                area,
            );
        }
    }

    #[test]
    fn test_model_update() {
        let mut model = Counter { value: 0 };

        model.update(Event::User(Msg::Increment));
        assert_eq!(model.value, 1);

        model.update(Event::User(Msg::Decrement));
        assert_eq!(model.value, 0);
    }

    #[test]
    fn test_cmd_creation() {
        let cmd = Cmd::new(|| Some(Msg::Increment));
        let msg = cmd.test_execute().unwrap();
        assert!(matches!(msg, Some(Msg::Increment)));
    }

    #[test]
    fn test_cmd_none() {
        let cmd: Option<Cmd<Msg>> = Cmd::none();
        assert!(cmd.is_some());
        let cmd = cmd.unwrap();
        assert!(cmd.is_noop());
    }

    #[test]
    fn test_cmd_fallible_success() {
        let cmd = Cmd::fallible(|| Ok(Some(Msg::Increment)));
        let result = cmd.test_execute();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(Msg::Increment));
    }

    #[test]
    fn test_cmd_fallible_error() {
        let cmd: Cmd<Msg> =
            Cmd::fallible(|| Err(crate::Error::from(std::io::Error::other("test error"))));
        let result = cmd.test_execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_cmd_exec_process() {
        let cmd = Cmd::exec_process("echo".to_string(), vec!["test".to_string()], |_| {
            Msg::Increment
        });
        assert!(cmd.is_exec_process());

        let exec_details = cmd.take_exec_process();
        assert!(exec_details.is_some());
        let (program, args, _) = exec_details.unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["test"]);
    }

    #[test]
    fn test_cmd_debug() {
        let cmd = Cmd::new(|| Some(Msg::Increment));
        let debug_str = format!("{cmd:?}");
        assert!(debug_str.contains("Cmd"));
    }

    #[test]
    fn test_model_init() {
        let mut model = Counter { value: 5 };
        let cmd = model.init();
        assert!(cmd.is_some());

        let result = cmd.unwrap().test_execute().unwrap();
        assert_eq!(result, Some(Msg::SetValue(0)));
    }
}
