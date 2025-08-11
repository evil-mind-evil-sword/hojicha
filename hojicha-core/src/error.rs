//! Error handling for the hojicha framework
//!
//! This module provides a structured approach to error handling throughout the framework.

use std::fmt;
use std::io;

/// Result type alias for hojicha operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the hojicha framework
#[derive(Debug)]
pub enum Error {
    /// I/O error (terminal operations, file access, etc.)
    Io(io::Error),

    /// Terminal-specific error
    Terminal(String),

    /// Event handling error
    Event(String),

    /// Command execution error
    Command(String),

    /// Component error
    Component(String),

    /// Model update error
    Model(String),

    /// Configuration error
    Config(String),

    /// Custom error for user-defined errors
    Custom(Box<dyn std::error::Error + Send + Sync>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {err}"),
            Error::Terminal(msg) => write!(f, "Terminal error: {msg}"),
            Error::Event(msg) => write!(f, "Event error: {msg}"),
            Error::Command(msg) => write!(f, "Command error: {msg}"),
            Error::Component(msg) => write!(f, "Component error: {msg}"),
            Error::Model(msg) => write!(f, "Model error: {msg}"),
            Error::Config(msg) => write!(f, "Configuration error: {msg}"),
            Error::Custom(err) => write!(f, "Custom error: {err}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Custom(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: std::sync::mpsc::RecvError) -> Self {
        Error::Event(format!("Channel receive error: {err}"))
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for Error {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        Error::Event(format!("Channel send error: {err}"))
    }
}

/// Error context trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context(self, msg: &str) -> Result<T>;

    /// Add context with a closure
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<Error>,
{
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|err| {
            let base_error = err.into();
            Error::Custom(Box::new(ContextError {
                context: msg.to_string(),
                source: base_error,
            }))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|err| {
            let base_error = err.into();
            Error::Custom(Box::new(ContextError {
                context: f(),
                source: base_error,
            }))
        })
    }
}

/// Error with additional context
#[derive(Debug)]
struct ContextError {
    context: String,
    source: Error,
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for ContextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Error handler trait for models
pub trait ErrorHandler {
    /// Handle an error, returning true if the error was handled
    fn handle_error(&mut self, error: Error) -> bool;
}

/// Default error handler that logs errors to stderr
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&mut self, error: Error) -> bool {
        eprintln!("Error: {error}");

        // Print error chain
        let mut current_error: &dyn std::error::Error = &error;
        while let Some(source) = current_error.source() {
            eprintln!("  Caused by: {source}");
            current_error = source;
        }

        false // Error not handled, program should exit
    }
}

/// Panic handler for converting panics to errors
pub fn set_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(location) = panic_info.location() {
            format!(
                " at {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            )
        } else {
            String::new()
        };

        eprintln!("Panic occurred: {msg}{location}");
    }));
}

/// Helper macro for creating errors with context
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err($crate::error::Error::Custom(
            format!($msg).into()
        ))
    };
    ($err:expr $(,)?) => {
        return Err($crate::error::Error::Custom(
            format!("{}", $err).into()
        ))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::Error::Custom(
            format!($fmt, $($arg)*).into()
        ))
    };
}

/// Helper macro for ensuring conditions
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            $crate::bail!($msg);
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            $crate::bail!($err);
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail!($fmt, $($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display() {
        let err = Error::Terminal("Failed to initialize".to_string());
        assert_eq!(err.to_string(), "Terminal error: Failed to initialize");

        let err = Error::Io(io::Error::new(io::ErrorKind::NotFound, "File not found"));
        assert_eq!(err.to_string(), "I/O error: File not found");
    }

    #[test]
    fn test_error_context() {
        let result: Result<()> = Err(Error::Terminal("Base error".to_string()));
        let with_context = result.context("While initializing terminal");

        assert!(with_context.is_err());
        let err_str = with_context.unwrap_err().to_string();
        assert!(err_str.contains("While initializing terminal"));
        assert!(err_str.contains("Base error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let err: Error = io_err.into();

        match err {
            Error::Io(_) => (),
            _ => panic!("Expected Io error variant"),
        }
    }

    #[test]
    fn test_bail_macro() {
        fn test_fn() -> Result<()> {
            bail!("Test error");
        }

        assert!(test_fn().is_err());
        assert_eq!(
            test_fn().unwrap_err().to_string(),
            "Custom error: Test error"
        );
    }

    #[test]
    fn test_ensure_macro() {
        fn test_fn(value: i32) -> Result<i32> {
            ensure!(value > 0, "Value must be positive");
            Ok(value)
        }

        assert!(test_fn(5).is_ok());
        assert!(test_fn(-1).is_err());
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            Error::Terminal("terminal error".to_string()),
            Error::Event("event error".to_string()),
            Error::Command("command error".to_string()),
            Error::Component("component error".to_string()),
            Error::Model("model error".to_string()),
            Error::Config("config error".to_string()),
        ];

        for error in errors {
            let display_str = error.to_string();
            assert!(!display_str.is_empty());

            // Test that source returns None for string-based errors
            assert!(error.source().is_none());
        }
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let err = Error::Io(io_err);

        // Should have a source
        assert!(StdError::source(&err).is_some());

        let source = err.source().unwrap();
        assert_eq!(source.to_string(), "File not found");
    }

    #[test]
    fn test_custom_error() {
        let custom_err = Box::new(io::Error::other("custom"));
        let err = Error::Custom(custom_err);

        assert!(StdError::source(&err).is_some());
        assert!(err.to_string().contains("Custom error"));
    }

    #[test]
    fn test_channel_error_conversions() {
        let recv_err = std::sync::mpsc::RecvError;
        let err: Error = recv_err.into();

        match err {
            Error::Event(_) => (),
            _ => panic!("Expected Event error variant"),
        }

        let (tx, _rx) = std::sync::mpsc::channel::<i32>();
        drop(_rx); // Close receiver to cause send error

        let send_result = tx.send(42);
        if let Err(send_err) = send_result {
            let err: Error = send_err.into();
            match err {
                Error::Event(_) => (),
                _ => panic!("Expected Event error variant"),
            }
        }
    }

    #[test]
    fn test_with_context() {
        let result: Result<()> = Err(Error::Terminal("Base error".to_string()));
        let with_context = result.with_context(|| "Dynamic context".to_string());

        assert!(with_context.is_err());
        let err_str = with_context.unwrap_err().to_string();
        assert!(err_str.contains("Dynamic context"));
    }

    #[test]
    fn test_default_error_handler() {
        let mut handler = DefaultErrorHandler;
        let error = Error::Terminal("test error".to_string());

        // Should return false (error not handled)
        assert!(!handler.handle_error(error));
    }

    #[test]
    fn test_context_error() {
        let base_error = Error::Terminal("base".to_string());
        let context_error = ContextError {
            context: "context".to_string(),
            source: base_error,
        };

        let display_str = context_error.to_string();
        assert!(display_str.contains("context"));
        assert!(display_str.contains("base"));

        assert!(StdError::source(&context_error).is_some());
    }

    #[test]
    fn test_bail_macro_with_format() {
        fn test_fn(value: i32) -> Result<()> {
            bail!("Value {} is invalid", value);
        }

        let err = test_fn(42).unwrap_err();
        assert!(err.to_string().contains("Value 42 is invalid"));
    }

    #[test]
    fn test_ensure_macro_with_format() {
        fn test_fn(value: i32, min: i32) -> Result<i32> {
            ensure!(value >= min, "Value {} must be >= {}", value, min);
            Ok(value)
        }

        assert!(test_fn(10, 5).is_ok());

        let err = test_fn(3, 5).unwrap_err();
        assert!(err.to_string().contains("Value 3 must be >= 5"));
    }

    #[test]
    fn test_panic_handler() {
        // Test that we can set the panic handler without panicking
        set_panic_handler();

        // Reset to default handler
        let _ = std::panic::take_hook();
    }
}
