//! Style system for Hojicha
//!
//! This module provides a high-level styling API inspired by Charmbracelet's Lipgloss,
//! built on top of Ratatui's style primitives.

mod builder;
mod color;
mod floating;
mod gradient;
mod grid;
mod layout;
mod theme;

pub use builder::{BorderStyle, Margin, Padding, Style, StyleBuilder, TextAlign};
pub use color::{AdaptiveColor, BackgroundMode, Color, ColorProfile};
pub use floating::{
    AnchorPoint, Dropdown, FloatPosition, FloatingElement, LayerManager, Overlay, Tooltip,
    TooltipPosition,
};
pub use gradient::{
    render_gradient_background, DiagonalDirection, Gradient, GradientType, LinearDirection,
};
pub use grid::{Grid, GridBuilder, GridCell, GridTemplate};
pub use layout::{
    center, join_horizontal, join_vertical, place, place_horizontal, place_in_area, place_vertical,
    Alignment, Element, HAlign, LayoutBuilder, PositionedElement, StyledText, VAlign,
};
pub use theme::{ColorPalette, Theme, Themed};

// Re-export some useful Ratatui types for convenience
pub use ratatui::style::{Modifier, Stylize};
