//! Style system for Hojicha
//! 
//! This module provides a high-level styling API inspired by Charmbracelet's Lipgloss,
//! built on top of Ratatui's style primitives.

mod builder;
mod color;
mod layout;
mod theme;

pub use builder::{Style, StyleBuilder, Padding, Margin, BorderStyle};
pub use color::{Color, AdaptiveColor, ColorProfile, BackgroundMode};
pub use layout::{
    join_horizontal, join_vertical, center, place,
    Alignment, HAlign, VAlign, Element, LayoutBuilder, StyledText
};
pub use theme::{Theme, ColorPalette, Themed};

// Re-export some useful Ratatui types for convenience
pub use ratatui::style::{Modifier, Stylize};