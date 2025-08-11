//! Render guard macro for safe component rendering
//!
//! Provides a macro to ensure components don't crash when areas have zero dimensions.

/// Macro to guard render functions against invalid areas
///
/// Usage:
/// ```
/// render_guard!(area);
/// ```
///
/// This will return early from the function if the area has zero width or height.
#[macro_export]
macro_rules! render_guard {
    ($area:expr) => {
        if $area.width == 0 || $area.height == 0 {
            return;
        }
    };

    // Version that returns a value
    ($area:expr, $return_value:expr) => {
        if $area.width == 0 || $area.height == 0 {
            return $return_value;
        }
    };
}

/// Trait for safe rendering with automatic area validation
pub trait SafeRender {
    /// Render with automatic area validation
    fn safe_render<F, R>(&self, area: ratatui::layout::Rect, render_fn: F) -> Option<R>
    where
        F: FnOnce() -> R,
    {
        if area.width > 0 && area.height > 0 {
            Some(render_fn())
        } else {
            None
        }
    }
}

// Implement for all types
impl<T> SafeRender for T {}
