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
pub mod utils;
pub mod viewport;

// New unified components that replace duplicates
pub mod unified_list;
pub mod unified_table;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use help::{Help, HelpBuilder, HelpEntry, HelpMode};
pub use keybinding::{KeyBinding, KeyMap};
// Legacy components (deprecated - use unified versions instead)
#[deprecated(note = "Use unified_list::UnifiedList instead")]
pub use list::{List, ListOptions};
#[deprecated(note = "Use unified_table::UnifiedTable instead")]  
pub use table::{Table, TableOptions, TableRow};
#[deprecated(note = "Use unified_list::UnifiedList instead")]
pub use styled_list::{ListItemTrait, StyledList};
#[deprecated(note = "Use unified_table::UnifiedTable instead")]
pub use styled_table::{Column, SortDirection, SortState, StyledTable};

// New unified components (recommended)
pub use unified_list::{UnifiedList, ListItem, ListConfig};
pub use unified_table::{UnifiedTable, TableRow as UnifiedTableRow, Column as UnifiedColumn, SortDirection as UnifiedSortDirection, SortState as UnifiedSortState, TableConfig};

// Other components
pub use modal::{Modal, ModalSize};
pub use paginator::{Paginator, PaginatorStyle};
pub use progress_bar::{ProgressBar, ProgressStyle};
pub use spinner::{Spinner, SpinnerStyle};
pub use status_bar::{StatusBar, StatusBarBuilder, StatusBarPosition, StatusSegment};
pub use stopwatch::{Lap, Stopwatch, StopwatchFormat, StopwatchState};
pub use tabs::{Tab, TabPosition, TabStyle, Tabs, TabsBuilder};
pub use text_input::{TextInput, ValidationResult};
pub use textarea::{TextArea, TextAreaOptions};
pub use timer::{Timer, TimerFormat, TimerState};
pub use viewport::{Viewport, ViewportOptions};
