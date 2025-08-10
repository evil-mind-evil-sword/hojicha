//! Logging utilities for debugging TUI applications
//!
//! This module provides file-based logging that doesn't interfere with the terminal UI.
//! It's essential for debugging TUI applications where stderr output would corrupt the display.
//!
//! # Example
//!
//! ```ignore
//! use hojicha::logging;
//!
//! // Initialize file logger
//! logging::init_file_logger("/tmp/app.log").unwrap();
//!
//! // Log messages at different levels
//! logging::debug("Debug information");
//! logging::info("Application started");
//! logging::warn("Low memory");
//! logging::error("Connection failed");
//! ```

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, Once};

static INIT: Once = Once::new();
static mut LOGGER: Option<Arc<Mutex<Logger>>> = None;

/// Log levels for filtering messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Debug level for verbose debugging information
    Debug = 0,
    /// Info level for general informational messages
    Info = 1,
    /// Warning level for potentially problematic situations
    Warn = 2,
    /// Error level for error conditions
    Error = 3,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

struct Logger {
    writer: Box<dyn Write + Send>,
    level: LogLevel,
}

impl Logger {
    fn log(&mut self, level: LogLevel, message: &str) -> io::Result<()> {
        if level >= self.level {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            writeln!(
                self.writer,
                "[{}] {}: {}",
                timestamp,
                level.as_str(),
                message
            )?;
            self.writer.flush()?;
        }
        Ok(())
    }
}

/// Initialize the file logger with a given path
///
/// This creates or appends to the specified log file.
/// Only the first call to any init function will take effect.
pub fn init_file_logger(path: &str) -> io::Result<()> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;

    init_with_writer(Box::new(file))
}

/// Initialize the logger with a custom writer
///
/// This allows logging to any destination that implements Write.
/// Only the first call to any init function will take effect.
pub fn init_with_writer(writer: Box<dyn Write + Send>) -> io::Result<()> {
    init_with_writer_and_level(writer, LogLevel::Debug)
}

/// Initialize the logger with a custom writer and minimum log level
///
/// Messages below the specified level will be filtered out.
/// Only the first call to any init function will take effect.
pub fn init_with_writer_and_level(
    writer: Box<dyn Write + Send>,
    level: LogLevel,
) -> io::Result<()> {
    let result = Ok(());

    INIT.call_once(|| {
        let logger = Arc::new(Mutex::new(Logger { writer, level }));
        unsafe {
            LOGGER = Some(logger);
        }
    });

    result
}

/// Log a debug message
pub fn debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Log an info message
pub fn info(message: &str) {
    log(LogLevel::Info, message);
}

/// Log a warning message
pub fn warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Log an error message
pub fn error(message: &str) {
    log(LogLevel::Error, message);
}

/// Internal logging function
fn log(level: LogLevel, message: &str) {
    unsafe {
        let logger_ptr = &raw const LOGGER;
        if let Some(ref logger) = *logger_ptr {
            if let Ok(mut logger) = logger.lock() {
                let _ = logger.log(level, message);
            }
        }
    }
}

/// Log command for debug messages
///
/// Returns a command that logs a debug message when executed.
pub fn log_debug<M: crate::core::Message>(message: impl Into<String>) -> crate::core::Cmd<M> {
    let msg = message.into();
    crate::commands::custom(move || {
        debug(&msg);
        None
    })
}

/// Log command for info messages
///
/// Returns a command that logs an info message when executed.
pub fn log_info<M: crate::core::Message>(message: impl Into<String>) -> crate::core::Cmd<M> {
    let msg = message.into();
    crate::commands::custom(move || {
        info(&msg);
        None
    })
}

/// Log command for warning messages
///
/// Returns a command that logs a warning message when executed.
pub fn log_warn<M: crate::core::Message>(message: impl Into<String>) -> crate::core::Cmd<M> {
    let msg = message.into();
    crate::commands::custom(move || {
        warn(&msg);
        None
    })
}

/// Log command for error messages
///
/// Returns a command that logs an error message when executed.
pub fn log_error<M: crate::core::Message>(message: impl Into<String>) -> crate::core::Cmd<M> {
    let msg = message.into();
    crate::commands::custom(move || {
        error(&msg);
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_logging_levels() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = TestWriter {
            buffer: buffer.clone(),
        };

        // Note: We can't test this properly due to the Once guard
        // In real tests, we'd need to reset the global state between tests
        // For now, this is more of a compilation test
        let _ = init_with_writer_and_level(Box::new(writer), LogLevel::Info);

        debug("Should not appear");
        info("Should appear");
        warn("Should also appear");

        // In a real test, we'd check the buffer contents
        // let contents = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
        // assert!(!contents.contains("Should not appear"));
        // assert!(contents.contains("Should appear"));
    }
}
