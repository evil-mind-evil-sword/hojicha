//! UI Components for building terminal interfaces
//!
//! This module provides pre-built components similar to Bubbles for Bubbletea,
//! including text input, scrollable views, and more.

pub mod button;
pub mod keybinding;
pub mod list;
pub mod modal;
pub mod progress_bar;
pub mod spinner;
pub mod styled_list;
pub mod styled_table;
pub mod table;
pub mod text_input;
pub mod textarea;
pub mod viewport;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use keybinding::{KeyBinding, KeyMap};
pub use list::{List, ListOptions};
pub use modal::{Modal, ModalSize};
pub use progress_bar::{ProgressBar, ProgressStyle};
pub use spinner::{Spinner, SpinnerStyle};
pub use styled_list::{ListItemTrait, StyledList};
pub use styled_table::{Column, SortDirection, SortState, StyledTable};
pub use table::{Table, TableOptions, TableRow};
pub use text_input::{TextInput, ValidationResult};
pub use textarea::{TextArea, TextAreaOptions};
pub use viewport::{Viewport, ViewportOptions};
