//! # Hojicha Pearls
//!
//! Beautiful UI components and styling for Hojicha TUI applications.
//!
//! This crate provides pre-built components and styling utilities to help you
//! create polished terminal user interfaces quickly.
//!
//! ## Components
//!
//! - **Button**: Clickable button with customizable styling
//! - **Spinner**: Animated loading indicators
//! - **ProgressBar**: Visual progress indication
//! - **Tabs**: Tab-based navigation
//! - **Modal**: Overlay dialogs
//! - **Table**: Data display in tabular format
//! - **List**: Scrollable lists with selection
//! - **TextArea**: Multi-line text input
//! - **TextInput**: Single-line text input
//! - **Timer**: Countdown/countup timer
//! - **Stopwatch**: Time tracking component
//! - **StatusBar**: Application status display
//! - **Paginator**: Page navigation
//! - **Viewport**: Scrollable content area
//!
//! ## Styling
//!
//! - **Theme**: Consistent color schemes
//! - **Gradient**: Color gradients for visual effects
//! - **Grid**: Layout grid system
//! - **Floating**: Floating element positioning
//! - **Builder**: Fluent style builder API

#![warn(missing_docs)]

pub mod components;
pub mod style;

// Re-export commonly used components
pub use components::{
    Button, Help, List, Modal, Paginator, ProgressBar, Spinner, SpinnerStyle, StatusBar, Stopwatch,
    StyledList, StyledTable, Table, Tabs, TextArea, TextInput, Timer, Viewport,
};

// Re-export styling utilities
pub use style::{Color, ColorProfile, FloatingElement, Gradient, Grid, StyleBuilder, Theme};

/// Prelude for convenient imports
/// 
/// This module provides the most commonly used components and styling utilities.
/// Import everything with:
/// 
/// ```
/// use hojicha_pearls::prelude::*;
/// ```
/// 
/// ## Included Components
/// 
/// ### Input Components
/// - [`TextInput`] - Single-line text input
/// - [`TextArea`] - Multi-line text editor
/// - [`Button`] - Clickable button
/// 
/// ### Display Components
/// - [`List`] - Scrollable list with selection
/// - [`Table`] - Data table with headers
/// - [`Tabs`] - Tab navigation
/// - [`Modal`] - Overlay dialog
/// - [`ProgressBar`] - Progress indicator
/// - [`Spinner`] - Loading animation
/// 
/// ### Styling
/// - [`Theme`] - Color themes
/// - [`ColorProfile`] - Component color sets
/// - [`StyleBuilder`] - Fluent style API
pub mod prelude {
    // Input components
    pub use crate::components::{Button, TextArea, TextInput};
    
    // Display components
    pub use crate::components::{
        List, Modal, ProgressBar, Spinner, SpinnerStyle, Table, Tabs,
    };
    
    // Common additional components
    pub use crate::components::{StatusBar, Timer, Viewport};
    
    // Styling utilities
    pub use crate::style::{ColorProfile, StyleBuilder, Theme};
}
