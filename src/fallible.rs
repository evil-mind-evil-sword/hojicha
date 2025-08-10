//! Fallible model support for error handling in the update cycle
//!
//! This module provides the `FallibleModel` trait which extends the basic `Model`
//! trait with error handling capabilities. This allows models to handle errors
//! gracefully without panicking or silently ignoring failures.

use crate::{
    core::{Cmd, Model},
    error::{Error, Result},
    event::Event,
};

/// A model that can handle errors in its update cycle
///
/// This trait extends the basic `Model` trait with fallible update support,
/// allowing models to return errors from update operations and handle them
/// appropriately.
///
/// # Example
///
/// ```ignore
/// use hojicha::prelude::*;
/// use hojicha::fallible::FallibleModel;
///
/// struct MyApp {
///     data: Vec<String>,
///     error_message: Option<String>,
/// }
///
/// impl Model for MyApp {
///     type Message = Msg;
///
///     fn update(&mut self, event: Event<Msg>) -> Cmd<Msg> {
///         // Delegate to try_update for error handling
///         match self.try_update(event) {
///             Ok(cmd) => cmd,
///             Err(err) => self.handle_error(err),
///         }
///     }
///
///     fn view(&self, frame: &mut Frame, area: Rect) {
///         // Render UI including any error messages
///     }
/// }
///
/// impl FallibleModel for MyApp {
///     fn try_update(&mut self, event: Event<Msg>) -> Result<Cmd<Msg>> {
///         match event {
///             Event::User(Msg::LoadData) => {
///                 // This operation might fail
///                 let data = load_data_from_file()?;
///                 self.data = data;
///                 Ok(Cmd::none())
///             }
///             _ => Ok(Cmd::none())
///         }
///     }
///
///     fn handle_error(&mut self, error: Error) -> Cmd<Msg> {
///         // Store error for display
///         self.error_message = Some(error.to_string());
///         // Could also convert to a message
///         commands::custom(|| Some(Msg::ErrorOccurred(error.to_string())))
///     }
/// }
/// ```
pub trait FallibleModel: Model {
    /// Fallible update that can return errors
    ///
    /// This method performs the actual update logic and can return errors
    /// when operations fail. The default implementation delegates to the
    /// infallible `update` method.
    fn try_update(&mut self, event: Event<Self::Message>) -> Result<Cmd<Self::Message>> {
        Ok(self.update(event))
    }

    /// Handle errors that occur during update
    ///
    /// This method is called when `try_update` returns an error. It allows
    /// the model to handle the error appropriately, such as:
    /// - Logging the error
    /// - Storing it for display in the UI
    /// - Converting it to a message for further processing
    /// - Attempting recovery
    ///
    /// The default implementation logs the error and returns `Cmd::none()`.
    fn handle_error(&mut self, error: Error) -> Cmd<Self::Message> {
        eprintln!("Error in model update: {}", error);

        // Print error chain
        let mut current_error: &dyn std::error::Error = &error;
        while let Some(source) = current_error.source() {
            eprintln!("  Caused by: {}", source);
            current_error = source;
        }

        Cmd::none()
    }

    /// Handle a panic that occurred during update
    ///
    /// This method is called when a panic is caught during the update cycle.
    /// The default implementation converts it to an error and delegates to
    /// `handle_error`.
    fn handle_panic(&mut self, panic_info: String) -> Cmd<Self::Message> {
        let error = Error::Model(format!("Panic in update: {}", panic_info));
        self.handle_error(error)
    }
}

/// Helper trait to make it easier to use FallibleModel in the Program
pub trait FallibleModelExt: FallibleModel {
    /// Perform a fallible update with automatic error handling
    ///
    /// This method combines `try_update` and `handle_error` for convenience.
    fn update_with_error_handling(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match self.try_update(event) {
            Ok(cmd) => cmd,
            Err(err) => self.handle_error(err),
        }
    }

    /// Perform an update with panic catching
    ///
    /// This method catches panics during update and converts them to errors.
    fn update_with_panic_catching(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        use std::panic;

        match panic::catch_unwind(panic::AssertUnwindSafe(|| self.try_update(event))) {
            Ok(Ok(cmd)) => cmd,
            Ok(Err(err)) => self.handle_error(err),
            Err(panic) => {
                let panic_info = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                self.handle_panic(panic_info)
            }
        }
    }
}

/// Automatically implement FallibleModelExt for all FallibleModel types
impl<T: FallibleModel> FallibleModelExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    enum TestMsg {
        Succeed,
        Fail,
        Panic,
        ErrorOccurred(String),
    }

    struct TestModel {
        success_count: usize,
        error_count: usize,
        panic_count: usize,
        last_error: Option<String>,
        errors: Arc<Mutex<Vec<String>>>,
    }

    impl Model for TestModel {
        type Message = TestMsg;

        fn update(&mut self, event: Event<TestMsg>) -> Cmd<TestMsg> {
            self.update_with_error_handling(event)
        }

        fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
    }

    impl FallibleModel for TestModel {
        fn try_update(&mut self, event: Event<TestMsg>) -> Result<Cmd<TestMsg>> {
            match event {
                Event::User(TestMsg::Succeed) => {
                    self.success_count += 1;
                    Ok(Cmd::none())
                }
                Event::User(TestMsg::Fail) => Err(Error::Model("Intentional failure".to_string())),
                Event::User(TestMsg::Panic) => {
                    panic!("Intentional panic!");
                }
                _ => Ok(Cmd::none()),
            }
        }

        fn handle_error(&mut self, error: Error) -> Cmd<TestMsg> {
            self.error_count += 1;
            let error_str = error.to_string();
            self.last_error = Some(error_str.clone());
            self.errors.lock().unwrap().push(error_str.clone());
            commands::custom(move || Some(TestMsg::ErrorOccurred(error_str)))
        }

        fn handle_panic(&mut self, panic_info: String) -> Cmd<TestMsg> {
            self.panic_count += 1;
            self.last_error = Some(panic_info.clone());
            self.errors.lock().unwrap().push(panic_info.clone());
            commands::custom(|| Some(TestMsg::ErrorOccurred(panic_info)))
        }
    }

    #[test]
    fn test_fallible_model_success() {
        let mut model = TestModel {
            success_count: 0,
            error_count: 0,
            panic_count: 0,
            last_error: None,
            errors: Arc::new(Mutex::new(Vec::new())),
        };

        let cmd = model.update(Event::User(TestMsg::Succeed));
        assert!(cmd.is_noop());
        assert_eq!(model.success_count, 1);
        assert_eq!(model.error_count, 0);
    }

    #[test]
    fn test_fallible_model_error() {
        let mut model = TestModel {
            success_count: 0,
            error_count: 0,
            panic_count: 0,
            last_error: None,
            errors: Arc::new(Mutex::new(Vec::new())),
        };

        let cmd = model.update(Event::User(TestMsg::Fail));
        assert!(!cmd.is_noop()); // Should return an error message command
        assert_eq!(model.error_count, 1);
        assert!(model.last_error.is_some());
        assert!(model.last_error.unwrap().contains("Intentional failure"));
    }

    #[test]
    fn test_fallible_model_panic_catching() {
        let mut model = TestModel {
            success_count: 0,
            error_count: 0,
            panic_count: 0,
            last_error: None,
            errors: Arc::new(Mutex::new(Vec::new())),
        };

        // Use panic catching version
        let cmd = model.update_with_panic_catching(Event::User(TestMsg::Panic));
        assert!(!cmd.is_noop());
        assert_eq!(model.panic_count, 1);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_default_error_handling() {
        struct DefaultModel;

        impl Model for DefaultModel {
            type Message = ();
            fn update(&mut self, _: Event<()>) -> Cmd<()> {
                Cmd::none()
            }
            fn view(&self, _: &mut ratatui::Frame, _: ratatui::layout::Rect) {}
        }

        impl FallibleModel for DefaultModel {}

        let mut model = DefaultModel;
        let cmd = model.handle_error(Error::Model("test".to_string()));
        assert!(cmd.is_noop());
    }
}
