//! UI Components for building terminal interfaces
//!
//! This module provides pre-built components similar to Bubbles for Bubbletea,
//! including text input, scrollable views, and more.

pub mod keybinding;
pub mod list;
pub mod spinner;
pub mod table;
pub mod textarea;
pub mod viewport;

pub use keybinding::{KeyBinding, KeyMap};
pub use list::{List, ListOptions};
pub use spinner::{Spinner, SpinnerStyle};
pub use table::{Table, TableOptions, TableRow};
pub use textarea::{TextArea, TextAreaOptions};
pub use viewport::{Viewport, ViewportOptions};
