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
    Button,
    Help,
    List,
    Modal,
    Paginator,
    ProgressBar,
    Spinner, SpinnerStyle,
    StatusBar,
    Stopwatch,
    StyledList,
    StyledTable,
    Table,
    Tabs,
    TextInput,
    TextArea,
    Timer,
    Viewport,
};

// Re-export styling utilities
pub use style::{
    StyleBuilder,
    Color, ColorProfile,
    FloatingElement,
    Gradient,
    Grid,
    Theme,
};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::components::{
        Button,
        Spinner, SpinnerStyle,
        ProgressBar,
        Tabs,
        Modal,
        Table,
        List,
        TextInput,
        TextArea,
    };
    
    pub use crate::style::{
        Theme,
        ColorProfile,
        StyleBuilder,
    };
}