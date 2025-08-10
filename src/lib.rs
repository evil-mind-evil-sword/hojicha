//! # Hojicha
//!
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
//!
//! A Rust framework that brings the Elm Architecture to terminal user interfaces,
//! built on top of [ratatui](https://github.com/ratatui-org/ratatui).
//!
//! ## Overview
//!
//! Hojicha provides a structured way to build terminal applications using:
//! - **Model**: Your application state
//! - **Message**: Events that trigger state changes  
//! - **Update**: Pure functions that handle messages and update state
//! - **View**: Pure functions that render your model to the terminal
//! - **Command**: Side effects that produce messages

pub mod async_handle;
pub mod commands;
pub mod components;
pub mod core;
pub mod error;
pub mod event;
pub mod logging;
pub mod metrics;
pub mod priority_queue;
pub mod program;
pub mod queue_scaling;
pub mod subscription;

#[cfg(test)]
pub mod testing;

// Re-export core types
pub use core::{Cmd, Model};
pub use error::{Error, ErrorContext, ErrorHandler, Result};
pub use event::{Event, KeyEvent, MouseEvent};
pub use program::{MouseMode, Program, ProgramOptions};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::commands::{
        batch, clear_line, clear_screen, custom, custom_async, custom_fallible,
        disable_bracketed_paste, disable_focus_change, disable_mouse, enable_bracketed_paste,
        enable_focus_change, enable_mouse_all_motion, enable_mouse_cell_motion, enter_alt_screen,
        every, exec, exec_command, exit_alt_screen, hide_cursor, interrupt, quit, sequence,
        set_window_title, show_cursor, suspend, tick, window_size,
    };
    pub use crate::components::{
        KeyBinding, KeyMap, List, ListOptions, Spinner, SpinnerStyle, Table, TableOptions,
        TableRow, TextArea, TextAreaOptions, Viewport, ViewportOptions,
    };
    pub use crate::core::{Cmd, Model};
    pub use crate::error::{Error, ErrorContext, ErrorHandler, Result};
    pub use crate::event::{Event, Key, KeyEvent, KeyModifiers, MouseEvent, WindowSize};
    pub use crate::logging::{log_debug, log_error, log_info, log_warn};
    pub use crate::program::{MouseMode, Program, ProgramOptions};

    // Re-export error macros
    pub use crate::{bail, ensure};

    // Re-export ratatui types users will need
    pub use ratatui::prelude::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic sanity test
        assert_eq!(2 + 2, 4);
    }
}
