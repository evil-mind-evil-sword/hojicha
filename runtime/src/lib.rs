//! # Hojicha Runtime
//!
//! Event handling and async runtime for Hojicha TUI applications.
//!
//! This crate provides the runtime infrastructure for handling events,
//! managing async operations, and running the main application loop.
//!
//! ## Core Components
//!
//! - **Program**: Main application runtime and event loop
//! - **Event Processing**: Priority-based event handling with backpressure
//! - **Async Support**: Tokio-based async command execution
//! - **Subscriptions**: Stream-based event sources
//! - **Error Resilience**: Panic recovery and error handling
//!
//! ## Features
//!
//! - Priority event queue with automatic scaling
//! - Resilient input handling with panic recovery
//! - Safe mutex operations that recover from poison
//! - Metrics and monitoring support
//! - Terminal management and restoration

#![warn(missing_docs)]

// Note: Event types are in hojicha-core since they're fundamental to the Model trait

// Program and runtime
pub mod program;
pub use program::{MouseMode, Program, ProgramOptions};

// Async support
pub mod async_handle;
pub mod stream_builders;
pub mod subscription;

// Event processing infrastructure
pub mod metrics;
pub mod priority_queue;
pub mod queue_scaling;

// Testing utilities
pub mod safe_priority;
pub mod testing;

// Error resilience
pub mod panic_handler;
pub mod resilient_input;
pub mod safe_mutex;

// Re-export from core
pub use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers, MouseEvent, WindowSize};

// Re-export runtime types
pub use async_handle::AsyncHandle;
pub use subscription::Subscription;

// Re-export program components
pub use program::{
    CommandExecutor, EventProcessor, EventStats, FpsLimiter, PriorityConfig,
    PriorityEventProcessor, TerminalConfig, TerminalManager,
};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::async_handle::AsyncHandle;
    pub use crate::panic_handler;
    pub use crate::program::{MouseMode, Program, ProgramOptions};
    pub use crate::stream_builders::{delayed_stream, interval_stream, timeout_stream};
    pub use crate::subscription::Subscription;
    pub use hojicha_core::event::{Event, Key, KeyEvent, KeyModifiers, MouseEvent};
}
