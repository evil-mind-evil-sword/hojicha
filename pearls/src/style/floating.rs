//! Floating elements for overlays, tooltips, and dropdowns
//!
//! Provides support for rendering elements above the main content.

use super::{Color, ColorProfile, Style};
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Position for floating elements
#[derive(Clone, Debug, PartialEq)]
pub enum FloatPosition {
    /// Position at specific coordinates
    Absolute(u16, u16),
    /// Position relative to an anchor point
    Relative(AnchorPoint),
    /// Center in the parent area
    Center,
    /// Position at cursor location
    Cursor(u16, u16),
}

/// Anchor point for relative positioning
#[derive(Clone, Debug, PartialEq)]
pub enum AnchorPoint {
    /// Top left corner
    TopLeft,
    /// Top center
    TopCenter,
    /// Top right corner
    TopRight,
    /// Middle left
    MiddleLeft,
    /// Center
    Center,
    /// Middle right
    MiddleRight,
    /// Bottom left corner
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Bottom right corner
    BottomRight,
}

/// A tooltip that appears on hover or focus
#[derive(Clone)]
pub struct Tooltip {
    /// Content of the tooltip
    content: String,
    /// Style for the tooltip
    style: Style,
    /// Maximum width
    max_width: u16,
    /// Position relative to trigger
    position: TooltipPosition,
    /// Whether to wrap text
    wrap: bool,
}

/// Position of tooltip relative to trigger element
#[derive(Clone, Debug, PartialEq)]
pub enum TooltipPosition {
    /// Above the element
    Above,
    /// Below the element
    Below,
    /// To the left
    Left,
    /// To the right
    Right,
}

impl Tooltip {
    /// Create a new tooltip
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: Style::new()
                .bg(Color::rgb(40, 40, 40))
                .fg(Color::white())
                .border(super::BorderStyle::Rounded),
            max_width: 40,
            position: TooltipPosition::Above,
            wrap: true,
        }
    }

    /// Set the style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set maximum width
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Set position relative to trigger
    pub fn with_position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// Set whether to wrap text
    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Calculate the area for the tooltip
    pub fn calculate_area(&self, trigger_area: Rect, parent_area: Rect) -> Rect {
        let lines: Vec<&str> = self.content.lines().collect();
        let height = (lines.len() as u16 + 2).min(10); // +2 for borders
        let width = if self.wrap {
            self.max_width
        } else {
            lines
                .iter()
                .map(|line| line.len() as u16)
                .max()
                .unwrap_or(0)
                .min(self.max_width)
                + 2 // +2 for borders
        };

        let (x, y) = match self.position {
            TooltipPosition::Above => {
                let x = trigger_area.x + trigger_area.width.saturating_sub(width) / 2;
                let y = trigger_area.y.saturating_sub(height);
                (x, y)
            }
            TooltipPosition::Below => {
                let x = trigger_area.x + trigger_area.width.saturating_sub(width) / 2;
                let y = trigger_area.y + trigger_area.height;
                (x, y)
            }
            TooltipPosition::Left => {
                let x = trigger_area.x.saturating_sub(width);
                let y = trigger_area.y + trigger_area.height.saturating_sub(height) / 2;
                (x, y)
            }
            TooltipPosition::Right => {
                let x = trigger_area.x + trigger_area.width;
                let y = trigger_area.y + trigger_area.height.saturating_sub(height) / 2;
                (x, y)
            }
        };

        // Ensure tooltip stays within parent area
        let x = x.min(parent_area.x + parent_area.width.saturating_sub(width));
        let y = y.min(parent_area.y + parent_area.height.saturating_sub(height));

        Rect {
            x: x.max(parent_area.x),
            y: y.max(parent_area.y),
            width,
            height,
        }
    }

    /// Render the tooltip
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Clear the area first
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.style.get_border().to_ratatui())
            .style(self.style.to_ratatui(profile));

        let paragraph = Paragraph::new(self.content.as_str())
            .block(block)
            .wrap(if self.wrap {
                Wrap { trim: true }
            } else {
                Wrap { trim: false }
            })
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}

/// An overlay that covers the entire screen with a semi-transparent background
#[derive(Clone)]
pub struct Overlay {
    /// Background style (usually semi-transparent)
    background_style: Style,
    /// Content area style
    content_style: Style,
    /// Dimming level (0.0 to 1.0)
    dim_level: f32,
}

impl Overlay {
    /// Create a new overlay
    pub fn new() -> Self {
        Self {
            background_style: Style::new().bg(Color::black()),
            content_style: Style::new(),
            dim_level: 0.5,
        }
    }

    /// Set the background style
    pub fn with_background_style(mut self, style: Style) -> Self {
        self.background_style = style;
        self
    }

    /// Set the content style
    pub fn with_content_style(mut self, style: Style) -> Self {
        self.content_style = style;
        self
    }

    /// Set the dimming level
    pub fn with_dim_level(mut self, level: f32) -> Self {
        self.dim_level = level.clamp(0.0, 1.0);
        self
    }

    /// Render the overlay background
    pub fn render_background(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Create a dimmed background
        let block = Block::default().style(self.background_style.to_ratatui(profile));
        frame.render_widget(block, area);
    }

    /// Calculate centered content area
    pub fn content_area(&self, parent_area: Rect, width: u16, height: u16) -> Rect {
        let x = parent_area.x + parent_area.width.saturating_sub(width) / 2;
        let y = parent_area.y + parent_area.height.saturating_sub(height) / 2;

        Rect {
            x,
            y,
            width: width.min(parent_area.width),
            height: height.min(parent_area.height),
        }
    }
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new()
    }
}

/// A dropdown menu that appears below a trigger element
#[derive(Clone)]
pub struct Dropdown {
    /// Menu items
    items: Vec<String>,
    /// Selected item index
    selected: Option<usize>,
    /// Style for the dropdown
    style: Style,
    /// Style for selected item
    selected_style: Style,
    /// Maximum height
    max_height: u16,
    /// Width
    width: Option<u16>,
}

impl Dropdown {
    /// Create a new dropdown
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            selected: None,
            style: Style::new()
                .bg(Color::rgb(30, 30, 30))
                .fg(Color::white())
                .border(super::BorderStyle::Rounded),
            selected_style: Style::new().bg(Color::blue()).fg(Color::white()),
            max_height: 10,
            width: None,
        }
    }

    /// Set the selected item
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.items.len() {
                self.selected = Some(i);
            }
        } else {
            self.selected = None;
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        self.selected = match self.selected {
            None => Some(self.items.len().saturating_sub(1)),
            Some(0) => Some(self.items.len().saturating_sub(1)),
            Some(i) => Some(i - 1),
        };
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        self.selected = match self.selected {
            None => Some(0),
            Some(i) if i >= self.items.len() - 1 => Some(0),
            Some(i) => Some(i + 1),
        };
    }

    /// Get the selected item
    pub fn selected_item(&self) -> Option<&String> {
        self.selected.and_then(|i| self.items.get(i))
    }

    /// Set style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set selected style
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set maximum height
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = height;
        self
    }

    /// Set width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Calculate the area for the dropdown
    pub fn calculate_area(&self, trigger_area: Rect, parent_area: Rect) -> Rect {
        let height = (self.items.len() as u16 + 2).min(self.max_height); // +2 for borders
        let width = self.width.unwrap_or_else(|| {
            self.items
                .iter()
                .map(|item| item.len() as u16)
                .max()
                .unwrap_or(0)
                .max(trigger_area.width)
                + 2 // +2 for borders
        });

        let x = trigger_area.x;
        let y = trigger_area.y + trigger_area.height;

        // Adjust if dropdown would go off screen
        let y = if y + height > parent_area.y + parent_area.height {
            // Show above if not enough space below
            trigger_area.y.saturating_sub(height)
        } else {
            y
        };

        Rect {
            x: x.min(parent_area.x + parent_area.width.saturating_sub(width)),
            y,
            width,
            height,
        }
    }

    /// Render the dropdown
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Clear the area first
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(self.style.get_border().to_ratatui())
            .style(self.style.to_ratatui(profile));

        // Calculate inner area before rendering
        let inner = block.inner(area);
        frame.render_widget(block, area);
        for (i, item) in self.items.iter().enumerate() {
            if i >= inner.height as usize {
                break;
            }

            let item_area = Rect {
                x: inner.x,
                y: inner.y + i as u16,
                width: inner.width,
                height: 1,
            };

            let style = if Some(i) == self.selected {
                self.selected_style.to_ratatui(profile)
            } else {
                self.style.to_ratatui(profile)
            };

            let paragraph = Paragraph::new(item.as_str()).style(style);
            frame.render_widget(paragraph, item_area);
        }
    }
}

/// Layer management for floating elements
#[derive(Clone)]
pub struct LayerManager {
    /// Z-index sorted layers
    layers: Vec<(i32, LayerContent)>,
}

/// Content of a layer
#[derive(Clone)]
enum LayerContent {
    Tooltip(Tooltip, Rect),
    Dropdown(Dropdown, Rect),
    Custom(Box<dyn FloatingElement>),
}

/// Trait for custom floating elements
pub trait FloatingElement: Send + Sync {
    /// Render the floating element
    fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile);

    /// Get the z-index of this element
    fn z_index(&self) -> i32;

    /// Clone the element
    fn clone_box(&self) -> Box<dyn FloatingElement>;
}

impl Clone for Box<dyn FloatingElement> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl LayerManager {
    /// Create a new layer manager
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a tooltip to the layers
    pub fn add_tooltip(&mut self, tooltip: Tooltip, area: Rect, z_index: i32) {
        self.layers
            .push((z_index, LayerContent::Tooltip(tooltip, area)));
        self.sort_layers();
    }

    /// Add a dropdown to the layers
    pub fn add_dropdown(&mut self, dropdown: Dropdown, area: Rect, z_index: i32) {
        self.layers
            .push((z_index, LayerContent::Dropdown(dropdown, area)));
        self.sort_layers();
    }

    /// Add a custom floating element
    pub fn add_custom(&mut self, element: Box<dyn FloatingElement>) {
        let z_index = element.z_index();
        self.layers.push((z_index, LayerContent::Custom(element)));
        self.sort_layers();
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Sort layers by z-index
    fn sort_layers(&mut self) {
        self.layers.sort_by_key(|&(z, _)| z);
    }

    /// Render all layers
    pub fn render_all(&self, frame: &mut Frame, profile: &ColorProfile) {
        for (_, content) in &self.layers {
            match content {
                LayerContent::Tooltip(tooltip, area) => {
                    tooltip.render(frame, *area, profile);
                }
                LayerContent::Dropdown(dropdown, area) => {
                    dropdown.render(frame, *area, profile);
                }
                LayerContent::Custom(element) => {
                    // Custom elements handle their own area calculation
                    element.render(frame, frame.area(), profile);
                }
            }
        }
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}
