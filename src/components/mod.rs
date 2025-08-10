//! UI Components for building terminal interfaces
//!
//! This module provides pre-built components similar to Bubbles for Bubbletea,
//! including text input, scrollable views, and more.

pub mod keybinding;
pub mod list;
pub mod spinner;
pub mod styled_list;
pub mod table;
pub mod text_input;
pub mod textarea;
pub mod viewport;

pub use keybinding::{KeyBinding, KeyMap};
pub use list::{List, ListOptions};
pub use spinner::{Spinner, SpinnerStyle};
pub use styled_list::{StyledList, ListItemTrait};
pub use table::{Table, TableOptions, TableRow};
pub use text_input::{TextInput, ValidationResult};
pub use textarea::{TextArea, TextAreaOptions};
pub use viewport::{Viewport, ViewportOptions};
