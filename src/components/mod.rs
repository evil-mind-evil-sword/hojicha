//! UI Components for building terminal interfaces
//!
//! This module provides pre-built components similar to Bubbles for Bubbletea,
//! including text input, scrollable views, and more.

#[macro_use]
mod render_guard;
pub use render_guard::SafeRender;

pub mod button;
pub mod help;
pub mod keybinding;
pub mod list;
pub mod modal;
pub mod paginator;
pub mod progress_bar;
pub mod spinner;
pub mod status_bar;
pub mod stopwatch;
pub mod styled_list;
pub mod styled_table;
pub mod table;
pub mod tabs;
pub mod text_input;
pub mod textarea;
pub mod timer;
mod utils;
pub mod viewport;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use help::{Help, HelpBuilder, HelpEntry, HelpMode};
pub use keybinding::{KeyBinding, KeyMap};
pub use list::{List, ListOptions};
pub use modal::{Modal, ModalSize};
pub use paginator::{Paginator, PaginatorStyle};
pub use progress_bar::{ProgressBar, ProgressStyle};
pub use spinner::{Spinner, SpinnerStyle};
pub use status_bar::{StatusBar, StatusBarBuilder, StatusBarPosition, StatusSegment};
pub use stopwatch::{Lap, Stopwatch, StopwatchFormat, StopwatchState};
pub use styled_list::{ListItemTrait, StyledList};
pub use styled_table::{Column, SortDirection, SortState, StyledTable};
pub use table::{Table, TableOptions, TableRow};
pub use tabs::{Tab, TabPosition, TabStyle, Tabs, TabsBuilder};
pub use text_input::{TextInput, ValidationResult};
pub use textarea::{TextArea, TextAreaOptions};
pub use timer::{Timer, TimerFormat, TimerState};
pub use viewport::{Viewport, ViewportOptions};
