//! # Hojicha Core
//!
//! Core Elm Architecture (TEA) abstractions for building terminal user interfaces in Rust.
//!
//! This crate provides the fundamental building blocks of The Elm Architecture:
//!
//! ## The Elm Architecture
//!
//! Hojicha implements The Elm Architecture (TEA), a pattern for building interactive
//! applications with a clear separation of concerns:
//!
//! - **Model**: Your application state
//! - **Message**: Events that trigger state changes  
//! - **Update**: Pure functions that handle messages and update state
//! - **View**: Pure functions that render your model to the terminal
//! - **Command**: Side effects that produce messages
//!
//! ## Core Traits
//!
//! - [`Model`]: The main trait your application must implement
//! - [`Message`]: Marker trait for your application's message types
//! - [`Cmd`]: Commands for side effects and async operations
//!
//! ## Example
//!
//! ```no_run
//! use hojicha_core::{Model, Cmd};
//! use ratatui::Frame;
//!
//! struct App {
//!     counter: u32,
//! }
//!
//! enum Msg {
//!     Increment,
//!     Decrement,
//! }
//!
//! impl Model for App {
//!     type Message = Msg;
//!
//!     fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message> {
//!         match msg {
//!             Msg::Increment => self.counter += 1,
//!             Msg::Decrement => self.counter -= 1,
//!         }
//!         Cmd::none()
//!     }
//!
//!     fn view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
//!         // Render your UI here
//!     }
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

// Core TEA abstractions
pub mod async_helpers;
pub mod commands;
pub mod core;
pub mod debug;
pub mod error;
pub mod event;
pub mod fallible;
pub mod logging;

// Testing utilities (only in tests)
#[cfg(test)]
pub mod testing;

// Re-export core types
pub use core::{Cmd, Message, Model};
pub use error::{Error, ErrorContext, ErrorHandler, Result};
pub use event::{Event, Key, KeyEvent, KeyModifiers, MouseEvent};

// Re-export command constructors
pub use commands::{
    batch, custom, custom_async, custom_fallible, every, none, quit, sequence, spawn, tick,
};

/// Prelude module for convenient imports
/// 
/// This module provides the most commonly used types and functions for building
/// Hojicha applications. Import everything with:
/// 
/// ```
/// use hojicha_core::prelude::*;
/// ```
/// 
/// ## Included Items
/// 
/// ### Core Traits
/// - [`Model`] - The main trait your application implements
/// - [`Message`] - Marker trait for messages
/// - [`Cmd`] - Commands for side effects
/// 
/// ### Events
/// - [`Event`] - All event types (Key, Mouse, User, etc.)
/// - [`Key`], [`KeyEvent`], [`KeyModifiers`] - Keyboard handling
/// - [`MouseEvent`] - Mouse events
/// 
/// ### Commands
/// - [`none()`] - No-op command
/// - [`batch()`] - Run commands concurrently
/// - [`sequence()`] - Run commands in order
/// - [`tick()`] - Delayed command
/// - [`every()`] - Recurring command
/// - [`quit()`] - Exit the program
/// 
/// ### Error Handling
/// - [`Result`] - Hojicha's Result type
/// - [`Error`] - Hojicha's Error type
/// 
/// ### Ratatui Re-exports
/// - All of ratatui's prelude for building views
pub mod prelude {
    // Core traits and types
    pub use crate::core::{Cmd, Message, Model};
    
    // Events
    pub use crate::event::{Event, Key, KeyEvent, KeyModifiers, MouseEvent, WindowSize};
    
    // Essential commands
    pub use crate::commands::{
        batch, every, none, quit, sequence, tick,
    };
    
    // Error handling
    pub use crate::error::{Error, Result};
    
    // Re-export ratatui's prelude for views
    pub use ratatui::prelude::*;
}

// Users will directly import hojicha-runtime and hojicha-pearls as separate crates
