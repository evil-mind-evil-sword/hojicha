//! Utilities for consistent panic handling across the runtime
//!
//! This module provides reusable panic handling utilities to reduce
//! code duplication and ensure consistent error reporting.

use hojicha_core::event::Event;
use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::sync::mpsc::SyncSender;

/// Format a panic payload into a readable error message
pub fn format_panic_message(panic: Box<dyn Any + Send>, context: &str) -> String {
    let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = panic.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    };
    
    format!("{}: {}", context, panic_msg)
}

/// Execute a closure with panic recovery and send the result as an event
pub fn execute_with_panic_recovery<M, F>(
    f: F,
    tx: &SyncSender<Event<M>>,
    context: &str,
) where
    F: FnOnce() -> Option<M> + std::panic::UnwindSafe,
    M: Send + 'static,
{
    let result = std::panic::catch_unwind(f);
    
    match result {
        Ok(Some(msg)) => {
            let _ = tx.send(Event::User(msg));
        }
        Ok(None) => {
            // No message to send
        }
        Err(panic) => {
            let panic_msg = format_panic_message(panic, context);
            eprintln!("{}", panic_msg);
        }
    }
}

/// Execute a fallible closure with panic recovery
pub fn execute_fallible_with_panic_recovery<M, F>(
    f: F,
    tx: &SyncSender<Event<M>>,
    context: &str,
) where
    F: FnOnce() -> Result<Option<M>, Box<dyn std::error::Error + Send + Sync>> + std::panic::UnwindSafe,
    M: Send + 'static,
{
    let result = std::panic::catch_unwind(AssertUnwindSafe(f));
    
    match result {
        Ok(Ok(Some(msg))) => {
            let _ = tx.send(Event::User(msg));
        }
        Ok(Ok(None)) => {
            // No message to send
        }
        Ok(Err(error)) => {
            eprintln!("{}: {}", context, error);
        }
        Err(panic) => {
            let panic_msg = format_panic_message(panic, context);
            eprintln!("{}", panic_msg);
        }
    }
}