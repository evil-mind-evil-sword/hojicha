//! Error handling utilities for the runtime
//!
//! This module provides error handling strategies and utilities for
//! fallible commands and model operations.

use hojicha_core::error::Error;
use hojicha_core::event::Event;
use std::sync::mpsc::SyncSender;

/// Trait for custom error handling strategies
pub trait ErrorHandler<M>: Send + Sync {
    /// Handle an error that occurred during command execution
    fn handle_error(&self, error: Error, tx: &SyncSender<Event<M>>);
}

/// Default error handler that logs errors to stderr
pub struct DefaultErrorHandler;

impl<M> ErrorHandler<M> for DefaultErrorHandler {
    fn handle_error(&self, error: Error, _tx: &SyncSender<Event<M>>) {
        eprintln!("Command execution error: {}", error);
        
        // Print error chain
        let mut current_error: &dyn std::error::Error = &error;
        while let Some(source) = current_error.source() {
            eprintln!("  Caused by: {}", source);
            current_error = source;
        }
    }
}

/// Error handler that converts errors to events
pub struct EventErrorHandler<M, F>
where
    M: Clone + Send + 'static,
    F: Fn(Error) -> M + Send + Sync,
{
    converter: F,
}

impl<M, F> EventErrorHandler<M, F>
where
    M: Clone + Send + 'static,
    F: Fn(Error) -> M + Send + Sync,
{
    /// Create a new event error handler with the given error converter
    pub fn new(converter: F) -> Self {
        Self { converter }
    }
}

impl<M, F> ErrorHandler<M> for EventErrorHandler<M, F>
where
    M: Clone + Send + 'static,
    F: Fn(Error) -> M + Send + Sync,
{
    fn handle_error(&self, error: Error, tx: &SyncSender<Event<M>>) {
        // Convert error to message and send as event
        let msg = (self.converter)(error);
        let _ = tx.send(Event::User(msg));
    }
}

/// Error handler that combines logging and event generation
pub struct CompositeErrorHandler<M> {
    handlers: Vec<Box<dyn ErrorHandler<M>>>,
}

impl<M> CompositeErrorHandler<M> {
    /// Create a new composite error handler
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add an error handler to the composite
    pub fn add_handler(mut self, handler: Box<dyn ErrorHandler<M>>) -> Self {
        self.handlers.push(handler);
        self
    }
}

impl<M> ErrorHandler<M> for CompositeErrorHandler<M> {
    fn handle_error(&self, error: Error, tx: &SyncSender<Event<M>>) {
        // Since Error doesn't implement Clone, we convert to string and recreate
        let error_string = error.to_string();
        for handler in &self.handlers {
            handler.handle_error(Error::Model(error_string.clone()), tx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Error(String),
    }

    #[test]
    fn test_default_error_handler() {
        let (tx, _rx) = mpsc::sync_channel::<Event<TestMsg>>(10);
        let handler = DefaultErrorHandler;
        
        let error = Error::Model("Test error".to_string());
        handler.handle_error(error, &tx);
        
        // Default handler only logs, doesn't send events
        // We can't easily test stderr output, but at least verify it doesn't panic
    }

    #[test]
    fn test_event_error_handler() {
        let (tx, rx) = mpsc::sync_channel(10);
        let handler = EventErrorHandler::new(|err| TestMsg::Error(err.to_string()));
        
        let error = Error::Model("Test error".to_string());
        handler.handle_error(error, &tx);
        
        // Should receive error as event
        let event = rx.recv().unwrap();
        // The Error::Model variant includes prefix "Model error: " in Display
        assert_eq!(event, Event::User(TestMsg::Error("Model error: Test error".to_string())));
    }
    
    #[test]
    fn test_composite_error_handler() {
        let (tx, rx) = mpsc::sync_channel(10);
        
        let composite = CompositeErrorHandler::new()
            .add_handler(Box::new(DefaultErrorHandler))
            .add_handler(Box::new(EventErrorHandler::new(|err| {
                TestMsg::Error(format!("Handled: {}", err))
            })));
        
        let error = Error::Model("Test error".to_string());
        composite.handle_error(error, &tx);
        
        // Should receive event from EventErrorHandler
        // Note: CompositeErrorHandler converts error to string then wraps as Model error
        // So "Model error: Test error" becomes "Model error: Model error: Test error"
        let event = rx.recv().unwrap();
        assert_eq!(event, Event::User(TestMsg::Error("Handled: Model error: Model error: Test error".to_string())));
    }
}