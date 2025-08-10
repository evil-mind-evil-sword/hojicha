//! Style system for Hojicha
//!
//! This module provides a high-level styling API inspired by Charmbracelet's Lipgloss,
//! built on top of Ratatui's style primitives.

mod builder;
mod color;
mod layout;
mod theme;

pub use builder::{BorderStyle, Margin, Padding, Style, StyleBuilder};
pub use color::{AdaptiveColor, BackgroundMode, Color, ColorProfile};
pub use layout::{
    center, join_horizontal, join_vertical, place, Alignment, Element, HAlign, LayoutBuilder,
    StyledText, VAlign,
};
pub use theme::{ColorPalette, Theme, Themed};

// Re-export some useful Ratatui types for convenience
pub use ratatui::style::{Modifier, Stylize};
