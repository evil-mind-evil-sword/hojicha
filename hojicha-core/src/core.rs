//! Core traits and types for the Elm Architecture
//! 
//! This module contains the foundational traits and types that implement
//! The Elm Architecture (TEA) pattern in Hojicha.
//! 
//! ## The Elm Architecture
//! 
//! TEA is a pattern for organizing interactive applications with:
//! - **Unidirectional data flow**: Events flow through update to modify state
//! - **Pure functions**: Update and view are pure, side effects use commands
//! - **Clear separation**: Model (state), Update (logic), View (presentation)
//! 
//! ## Core Components
//! 
//! ### Model Trait
//! Your application struct implements this trait:
//! ```
//! # use hojicha_core::{Model, Cmd, Event};
//! # use ratatui::{Frame, layout::Rect};
//! struct MyApp {
//!     counter: i32,
//! }
//! 
//! impl Model for MyApp {
//!     type Message = MyMessage;
//!     
//!     fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
//!         // Handle events and return commands
//!         Cmd::none()
//!     }
//!     
//!     fn view(&self, frame: &mut Frame, area: Rect) {
//!         // Render the UI
//!     }
//! }
//! # enum MyMessage {}
//! ```
//! 
//! ### Commands
//! Commands represent side effects that produce messages:
//! ```
//! # use hojicha_core::Cmd;
//! # enum Msg { DataLoaded(String) }
//! let cmd: Cmd<Msg> = Cmd::new(|| {
//!     // Perform side effect
//!     let data = std::fs::read_to_string("data.txt").ok()?;
//!     Some(Msg::DataLoaded(data))
//! });
//! ```

use crate::event::Event;
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

    /// Initialize the model and return a command to run
    ///
    /// This method is called once when the program starts. Use it to:
    /// - Load initial data
    /// - Start timers
    /// - Perform initial setup
    ///
    /// Returns:
    /// - `Cmd::none()` - Start the event loop without any initial command
    /// - Any other command - Execute the command before starting the event loop
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::{Model, Cmd, commands};
    /// # use std::time::Duration;
    /// # struct MyApp;
    /// # enum Msg { Tick }
    /// # impl Model for MyApp {
    /// #     type Message = Msg;
    /// fn init(&mut self) -> Cmd<Self::Message> {
    ///     // Start a timer that ticks every second
    ///     commands::every(Duration::from_secs(1), |_| Msg::Tick)
    /// }
    /// #     fn update(&mut self, _: hojicha_core::Event<Self::Message>) -> Cmd<Self::Message> { Cmd::none() }
    /// #     fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
    /// # }
    /// ```
    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    /// Update the model based on a message and return a command
    ///
    /// This is the heart of your application logic. Handle events here and
    /// update your model's state accordingly.
    ///
    /// Returns:
    /// - `Cmd::none()` - Continue running without executing any command
    /// - `commands::quit()` - Exit the program
    /// - Any other command - Execute the command and continue
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::{Model, Cmd, Event, Key, commands};
    /// # struct Counter { value: i32 }
    /// # enum Msg { Increment, Decrement }
    /// # impl Model for Counter {
    /// #     type Message = Msg;
    /// fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
    ///     match event {
    ///         Event::Key(key) if key.key == Key::Char('q') => {
    ///             commands::quit()
    ///         }
    ///         Event::User(Msg::Increment) => {
    ///             self.value += 1;
    ///             Cmd::none()
    ///         }
    ///         Event::User(Msg::Decrement) => {
    ///             self.value -= 1;
    ///             Cmd::none()
    ///         }
    ///         _ => Cmd::none()
    ///     }
    /// }
    /// #     fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
    /// # }
    /// ```
    fn update(&mut self, msg: Event<Self::Message>) -> Cmd<Self::Message>;

    /// Render the model to the screen
    /// 
    /// This method is called after each update to render your UI.
    /// Use ratatui widgets to draw your interface.
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::Model;
    /// # use ratatui::{Frame, layout::Rect, widgets::{Block, Borders, Paragraph}};
    /// # struct MyApp { message: String }
    /// # impl Model for MyApp {
    /// #     type Message = ();
    /// #     fn update(&mut self, _: hojicha_core::Event<Self::Message>) -> hojicha_core::Cmd<Self::Message> { hojicha_core::Cmd::none() }
    /// fn view(&self, frame: &mut Frame, area: Rect) {
    ///     let widget = Paragraph::new(self.message.as_str())
    ///         .block(Block::default()
    ///             .title("My App")
    ///             .borders(Borders::ALL));
    ///     frame.render_widget(widget, area);
    /// }
    /// # }
    /// ```
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
    /// Execute an async future
    Async(Box<dyn std::future::Future<Output = Option<M>> + Send>),
}

impl<M: Message> Cmd<M> {
    /// Create a new command from a function
    /// 
    /// Note: If the function returns `None`, consider using `Cmd::none()` instead
    /// for better performance and clearer intent.
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::Cmd;
    /// # enum Msg { DataLoaded(String) }
    /// let cmd: Cmd<Msg> = Cmd::new(|| {
    ///     // Perform a side effect
    ///     let data = std::fs::read_to_string("config.json").ok()?;
    ///     Some(Msg::DataLoaded(data))
    /// });
    /// ```
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Option<M> + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Function(Box::new(f)),
        }
    }

    /// Create a new fallible command that can return errors
    /// 
    /// Use this when your command might fail and you want to handle errors gracefully.
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::{Cmd, Result};
    /// # enum Msg { ConfigLoaded(String) }
    /// let cmd: Cmd<Msg> = Cmd::fallible(|| {
    ///     let data = std::fs::read_to_string("config.json")?;
    ///     Ok(Some(Msg::ConfigLoaded(data)))
    /// });
    /// ```
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
    /// This is the idiomatic way to return "no command" from update().
    /// The program will continue running without executing any side effects.
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::{Model, Cmd, Event};
    /// # use ratatui::{Frame, layout::Rect};
    /// # struct MyApp;
    /// # impl Model for MyApp {
    /// #     type Message = ();
    /// fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
    ///     match event {
    ///         Event::Tick => {
    ///             // Update internal state but don't trigger side effects
    ///             Cmd::none()
    ///         }
    ///         _ => Cmd::none()
    ///     }
    /// }
    /// #     fn view(&self, _: &mut Frame, _: Rect) {}
    /// # }
    /// ```
    pub fn none() -> Self {
        Cmd {
            inner: CmdInner::NoOp,
        }
    }

    /// Internal method
    #[doc(hidden)]
    pub fn exec_process<F>(program: String, args: Vec<String>, callback: F) -> Self
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
    /// Internal method
    #[doc(hidden)]
    pub fn batch(cmds: Vec<Cmd<M>>) -> Self {
        Cmd {
            inner: CmdInner::Batch(cmds),
        }
    }

    /// Create a sequence command that executes commands in order
    /// Internal method
    #[doc(hidden)]
    pub fn sequence(cmds: Vec<Cmd<M>>) -> Self {
        Cmd {
            inner: CmdInner::Sequence(cmds),
        }
    }

    /// Create a quit command
    /// Internal method
    #[doc(hidden)]
    pub fn quit() -> Self {
        Cmd {
            inner: CmdInner::Quit,
        }
    }

    /// Create a tick command
    /// Internal method
    #[doc(hidden)]
    pub fn tick<F>(duration: std::time::Duration, callback: F) -> Self
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
    /// Internal method
    #[doc(hidden)]
    pub fn every<F>(duration: std::time::Duration, callback: F) -> Self
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

    /// Create an async command
    /// Internal method
    #[doc(hidden)]
    pub fn async_cmd<Fut>(future: Fut) -> Self
    where
        Fut: std::future::Future<Output = Option<M>> + Send + 'static,
    {
        Cmd {
            inner: CmdInner::Async(Box::new(future)),
        }
    }

    /// Execute the command and return its message
    /// Internal method
    #[doc(hidden)]
    pub fn execute(self) -> crate::Result<Option<M>> {
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
            CmdInner::Tick { .. } | CmdInner::Every { .. } | CmdInner::Async(_) => {
                // These are handled specially by the CommandExecutor with async runtime
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
    /// Check if this is an exec process command
    pub fn is_exec_process(&self) -> bool {
        matches!(self.inner, CmdInner::ExecProcess { .. })
    }

    /// Check if this is a no-op command
    pub fn is_noop(&self) -> bool {
        matches!(self.inner, CmdInner::NoOp)
    }

    /// Check if this is a quit command
    pub fn is_quit(&self) -> bool {
        matches!(self.inner, CmdInner::Quit)
    }

    /// Extract exec process details if this is an exec process command
    #[allow(clippy::type_complexity)]
    /// Internal method
    #[doc(hidden)]
    pub fn take_exec_process(self) -> Option<ExecDetails<M>> {
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
    /// Internal method
    #[doc(hidden)]
    pub fn is_batch(&self) -> bool {
        matches!(self.inner, CmdInner::Batch(_))
    }

    /// Take the batch commands (consumes the command)
    /// Internal method
    #[doc(hidden)]
    pub fn take_batch(self) -> Option<Vec<Cmd<M>>> {
        match self.inner {
            CmdInner::Batch(cmds) => Some(cmds),
            _ => None,
        }
    }

    /// Check if this is a sequence command
    /// Internal method
    #[doc(hidden)]
    pub fn is_sequence(&self) -> bool {
        matches!(self.inner, CmdInner::Sequence(_))
    }

    /// Take the sequence commands (consumes the command)
    /// Internal method
    #[doc(hidden)]
    pub fn take_sequence(self) -> Option<Vec<Cmd<M>>> {
        match self.inner {
            CmdInner::Sequence(cmds) => Some(cmds),
            _ => None,
        }
    }

    /// Internal method
    #[doc(hidden)]
    pub fn is_tick(&self) -> bool {
        matches!(self.inner, CmdInner::Tick { .. })
    }

    /// Internal method
    #[doc(hidden)]
    pub fn is_every(&self) -> bool {
        matches!(self.inner, CmdInner::Every { .. })
    }

    /// Internal method
    #[doc(hidden)]
    pub fn take_tick(self) -> Option<(std::time::Duration, Box<dyn FnOnce() -> M + Send>)> {
        match self.inner {
            CmdInner::Tick { duration, callback } => Some((duration, callback)),
            _ => None,
        }
    }

    #[allow(clippy::type_complexity)]
    /// Internal method
    #[doc(hidden)]
    pub fn take_every(
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

    /// Internal method
    #[doc(hidden)]
    pub fn is_async(&self) -> bool {
        matches!(self.inner, CmdInner::Async(_))
    }

    /// Internal method
    #[doc(hidden)]
    pub fn take_async(self) -> Option<Box<dyn std::future::Future<Output = Option<M>> + Send>> {
        match self.inner {
            CmdInner::Async(future) => Some(future),
            _ => None,
        }
    }

    /// Inspect this command for debugging
    ///
    /// This allows you to observe command execution without modifying behavior.
    ///
    /// # Example
    /// ```no_run
    /// # use hojicha_core::Cmd;
    /// # #[derive(Clone)]
    /// # struct Msg;
    /// let cmd: Cmd<Msg> = Cmd::none()
    ///     .inspect(|cmd| println!("Executing command: {:?}", cmd));
    /// ```
    pub fn inspect<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }

    /// Conditionally inspect this command
    ///
    /// Only runs the inspection function if the condition is true.
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::Cmd;
    /// # enum Msg { Data(String) }
    /// # let debug_mode = true;
    /// let cmd: Cmd<Msg> = Cmd::new(|| Some(Msg::Data("test".into())))
    ///     .inspect_if(debug_mode, |cmd| {
    ///         eprintln!("Debug: executing {}", cmd.debug_name());
    ///     });
    /// ```
    pub fn inspect_if<F>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        if condition {
            f(&self);
        }
        self
    }

    /// Get a string representation of the command type for debugging
    /// 
    /// # Example
    /// ```
    /// # use hojicha_core::{Cmd, commands};
    /// # enum Msg { Tick }
    /// # use std::time::Duration;
    /// let cmd: Cmd<Msg> = commands::tick(Duration::from_secs(1), || Msg::Tick);
    /// assert_eq!(cmd.debug_name(), "Tick");
    /// 
    /// let noop: Cmd<Msg> = Cmd::none();
    /// assert_eq!(noop.debug_name(), "NoOp");
    /// ```
    pub fn debug_name(&self) -> &'static str {
        match self.inner {
            CmdInner::Function(_) => "Function",
            CmdInner::Fallible(_) => "Fallible",
            CmdInner::ExecProcess { .. } => "ExecProcess",
            CmdInner::NoOp => "NoOp",
            CmdInner::Quit => "Quit",
            CmdInner::Batch(_) => "Batch",
            CmdInner::Sequence(_) => "Sequence",
            CmdInner::Tick { .. } => "Tick",
            CmdInner::Every { .. } => "Every",
            CmdInner::Async(_) => "Async",
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

        fn init(&mut self) -> Cmd<Self::Message> {
            Cmd::new(|| Some(Msg::SetValue(0)))
        }

        fn update(&mut self, msg: Event<Self::Message>) -> Cmd<Self::Message> {
            if let Event::User(msg) = msg {
                match msg {
                    Msg::Increment => {
                        self.value += 1;
                        Cmd::new(move || Some(Msg::Noop))
                    }
                    Msg::Decrement => {
                        self.value -= 1;
                        Cmd::new(move || Some(Msg::Noop))
                    }
                    Msg::SetValue(v) => {
                        self.value = v;
                        Cmd::none()
                    }
                    Msg::Noop => Cmd::none(),
                }
            } else {
                Cmd::none()
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
        let cmd: Cmd<Msg> = Cmd::none();
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
        assert!(!cmd.is_noop());

        let result = cmd.test_execute().unwrap();
        assert_eq!(result, Some(Msg::SetValue(0)));
    }
}
