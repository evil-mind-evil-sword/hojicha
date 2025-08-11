//! Utility functions for components
//!
//! Common helper functions used across components.

use ratatui::layout::Rect;

/// Check if an area is valid for rendering (non-zero width and height)
#[inline]
pub fn is_valid_area(area: Rect) -> bool {
    area.width > 0 && area.height > 0
}
