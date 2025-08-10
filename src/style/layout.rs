//! Layout composition utilities
//!
//! Provides high-level functions for composing layouts, inspired by Lipgloss.

use super::{ColorProfile, Style};
use ratatui::{
    layout::{Alignment as RatatuiAlignment, Constraint, Direction, Layout as RatatuiLayout, Rect},
    widgets::{Block, Borders, Widget},
    Frame,
};

/// Horizontal alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

/// Vertical alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VAlign {
    Top,
    Center,
    Bottom,
}

/// Combined alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub horizontal: HAlign,
    pub vertical: VAlign,
}

impl Alignment {
    pub const TOP_LEFT: Self = Self {
        horizontal: HAlign::Left,
        vertical: VAlign::Top,
    };

    pub const TOP_CENTER: Self = Self {
        horizontal: HAlign::Center,
        vertical: VAlign::Top,
    };

    pub const TOP_RIGHT: Self = Self {
        horizontal: HAlign::Right,
        vertical: VAlign::Top,
    };

    pub const CENTER_LEFT: Self = Self {
        horizontal: HAlign::Left,
        vertical: VAlign::Center,
    };

    pub const CENTER: Self = Self {
        horizontal: HAlign::Center,
        vertical: VAlign::Center,
    };

    pub const CENTER_RIGHT: Self = Self {
        horizontal: HAlign::Right,
        vertical: VAlign::Center,
    };

    pub const BOTTOM_LEFT: Self = Self {
        horizontal: HAlign::Left,
        vertical: VAlign::Bottom,
    };

    pub const BOTTOM_CENTER: Self = Self {
        horizontal: HAlign::Center,
        vertical: VAlign::Bottom,
    };

    pub const BOTTOM_RIGHT: Self = Self {
        horizontal: HAlign::Right,
        vertical: VAlign::Bottom,
    };
}

/// An element that can be rendered
pub trait Element {
    /// Render the element to a frame
    fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile);

    /// Get the style of this element
    fn style(&self) -> &Style;

    /// Set the style of this element
    fn with_style(self, style: Style) -> Self
    where
        Self: Sized;
}

/// A layout builder for composing elements
pub struct LayoutBuilder {
    direction: Direction,
    elements: Vec<Box<dyn Element>>,
    constraints: Vec<Constraint>,
    margin: u16,
    alignment: RatatuiAlignment,
}

impl LayoutBuilder {
    /// Create a new horizontal layout
    pub fn horizontal() -> Self {
        Self {
            direction: Direction::Horizontal,
            elements: Vec::new(),
            constraints: Vec::new(),
            margin: 0,
            alignment: RatatuiAlignment::Left,
        }
    }

    /// Create a new vertical layout
    pub fn vertical() -> Self {
        Self {
            direction: Direction::Vertical,
            elements: Vec::new(),
            constraints: Vec::new(),
            margin: 0,
            alignment: RatatuiAlignment::Left,
        }
    }

    /// Add an element with a constraint
    pub fn add(mut self, element: impl Element + 'static, constraint: Constraint) -> Self {
        self.elements.push(Box::new(element));
        self.constraints.push(constraint);
        self
    }

    /// Set margin between elements
    pub fn margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    /// Set alignment
    pub fn align(mut self, alignment: RatatuiAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Build and render the layout
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        let chunks = RatatuiLayout::default()
            .direction(self.direction)
            .constraints(&self.constraints)
            .margin(self.margin)
            .split(area);

        for (element, &chunk) in self.elements.iter().zip(chunks.iter()) {
            element.render(frame, chunk, profile);
        }
    }
}

/// Join elements horizontally
pub fn join_horizontal<E: Element + 'static>(alignment: VAlign, elements: Vec<E>) -> LayoutBuilder {
    let mut builder = LayoutBuilder::horizontal();

    let ratatui_align = match alignment {
        VAlign::Top => RatatuiAlignment::Left,
        VAlign::Center => RatatuiAlignment::Center,
        VAlign::Bottom => RatatuiAlignment::Right,
    };

    builder = builder.align(ratatui_align);

    // Equal distribution by default
    let constraint = Constraint::Ratio(1, elements.len() as u32);

    for element in elements {
        builder = builder.add(element, constraint);
    }

    builder
}

/// Join elements vertically
pub fn join_vertical<E: Element + 'static>(alignment: HAlign, elements: Vec<E>) -> LayoutBuilder {
    let mut builder = LayoutBuilder::vertical();

    let ratatui_align = match alignment {
        HAlign::Left => RatatuiAlignment::Left,
        HAlign::Center => RatatuiAlignment::Center,
        HAlign::Right => RatatuiAlignment::Right,
    };

    builder = builder.align(ratatui_align);

    // Equal distribution by default
    let constraint = Constraint::Ratio(1, elements.len() as u32);

    for element in elements {
        builder = builder.add(element, constraint);
    }

    builder
}

/// Center an element in an area
pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let x = if width < area.width {
        area.x + (area.width - width) / 2
    } else {
        area.x
    };

    let y = if height < area.height {
        area.y + (area.height - height) / 2
    } else {
        area.y
    };

    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

/// Place an element with specific alignment
pub fn place(area: Rect, alignment: Alignment, width: u16, height: u16) -> Rect {
    let x = match alignment.horizontal {
        HAlign::Left => area.x,
        HAlign::Center => {
            if width < area.width {
                area.x + (area.width - width) / 2
            } else {
                area.x
            }
        }
        HAlign::Right => {
            if width < area.width {
                area.x + area.width - width
            } else {
                area.x
            }
        }
    };

    let y = match alignment.vertical {
        VAlign::Top => area.y,
        VAlign::Center => {
            if height < area.height {
                area.y + (area.height - height) / 2
            } else {
                area.y
            }
        }
        VAlign::Bottom => {
            if height < area.height {
                area.y + area.height - height
            } else {
                area.y
            }
        }
    };

    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

/// A styled text element
#[derive(Clone)]
pub struct StyledText {
    text: String,
    style: Style,
}

impl StyledText {
    /// Create a new styled text element
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
        }
    }
}

impl Element for StyledText {
    fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        use ratatui::widgets::Paragraph;

        let widget = Paragraph::new(self.text.as_str()).style(self.style.to_ratatui(profile));

        // Apply padding if specified
        let padding = self.style.get_padding();
        let padded_area =
            if padding.top > 0 || padding.right > 0 || padding.bottom > 0 || padding.left > 0 {
                Rect {
                    x: area.x + padding.left,
                    y: area.y + padding.top,
                    width: area.width.saturating_sub(padding.left + padding.right),
                    height: area.height.saturating_sub(padding.top + padding.bottom),
                }
            } else {
                area
            };

        // Apply border if specified
        let widget = if self.style.get_border() != &super::BorderStyle::None {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(self.style.get_border().to_ratatui());

            if let Some(border_color) = self.style.get_border_color() {
                block = block.border_style(
                    ratatui::style::Style::default().fg(border_color.to_ratatui(profile)),
                );
            }

            widget.block(block)
        } else {
            widget
        };

        frame.render_widget(widget, padded_area);
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// A container that can hold multiple elements
pub struct Container {
    layout: LayoutBuilder,
    style: Style,
}

impl Container {
    /// Create a new container with a layout
    pub fn new(layout: LayoutBuilder) -> Self {
        Self {
            layout,
            style: Style::default(),
        }
    }
}

impl Element for Container {
    fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Apply container style (background, border, etc.)
        if self.style.get_border() != &super::BorderStyle::None {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(self.style.get_border().to_ratatui());

            if let Some(border_color) = self.style.get_border_color() {
                block = block.border_style(
                    ratatui::style::Style::default().fg(border_color.to_ratatui(profile)),
                );
            }

            frame.render_widget(block, area);
        }

        // Render contained elements
        let margin = self.style.get_margin();
        let content_area = Rect {
            x: area.x + margin.left,
            y: area.y + margin.top,
            width: area.width.saturating_sub(margin.left + margin.right),
            height: area.height.saturating_sub(margin.top + margin.bottom),
        };

        self.layout.render(frame, content_area, profile);
    }

    fn style(&self) -> &Style {
        &self.style
    }

    fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// Place content at a specific position within an area
///
/// This function positions content within a given area based on horizontal
/// and vertical alignment preferences.
///
/// # Arguments
/// * `area` - The area to place content within
/// * `content_width` - Width of the content to place
/// * `content_height` - Height of the content to place
/// * `h_align` - Horizontal alignment
/// * `v_align` - Vertical alignment
///
/// # Returns
/// A `Rect` representing the positioned content area
pub fn place_in_area(
    area: Rect,
    content_width: u16,
    content_height: u16,
    h_align: HAlign,
    v_align: VAlign,
) -> Rect {
    let x = match h_align {
        HAlign::Left => area.x,
        HAlign::Center => {
            if content_width < area.width {
                area.x + (area.width - content_width) / 2
            } else {
                area.x
            }
        }
        HAlign::Right => {
            if content_width < area.width {
                area.x + area.width - content_width
            } else {
                area.x
            }
        }
    };

    let y = match v_align {
        VAlign::Top => area.y,
        VAlign::Center => {
            if content_height < area.height {
                area.y + (area.height - content_height) / 2
            } else {
                area.y
            }
        }
        VAlign::Bottom => {
            if content_height < area.height {
                area.y + area.height - content_height
            } else {
                area.y
            }
        }
    };

    Rect {
        x,
        y,
        width: content_width.min(area.width),
        height: content_height.min(area.height),
    }
}

/// Place text horizontally within a given width
///
/// Centers, left-aligns, or right-aligns text within the specified width
/// by adding appropriate padding.
pub fn place_horizontal(text: &str, width: u16, align: HAlign) -> String {
    let text_width = text.chars().count();
    if text_width >= width as usize {
        return text.chars().take(width as usize).collect();
    }

    let padding = width as usize - text_width;
    match align {
        HAlign::Left => format!("{}{}", text, " ".repeat(padding)),
        HAlign::Center => {
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
        }
        HAlign::Right => format!("{}{}", " ".repeat(padding), text),
    }
}

/// Place text vertically within a given height
///
/// Positions text at the top, center, or bottom of the specified height
/// by adding appropriate empty lines.
pub fn place_vertical(lines: Vec<String>, height: u16, align: VAlign) -> Vec<String> {
    let content_height = lines.len();
    if content_height >= height as usize {
        return lines.into_iter().take(height as usize).collect();
    }

    let padding = height as usize - content_height;
    let empty_line = String::new();

    match align {
        VAlign::Top => {
            let mut result = lines;
            for _ in 0..padding {
                result.push(empty_line.clone());
            }
            result
        }
        VAlign::Center => {
            let top_pad = padding / 2;
            let bottom_pad = padding - top_pad;
            let mut result = Vec::with_capacity(height as usize);
            
            for _ in 0..top_pad {
                result.push(empty_line.clone());
            }
            result.extend(lines);
            for _ in 0..bottom_pad {
                result.push(empty_line.clone());
            }
            result
        }
        VAlign::Bottom => {
            let mut result = Vec::with_capacity(height as usize);
            for _ in 0..padding {
                result.push(empty_line.clone());
            }
            result.extend(lines);
            result
        }
    }
}

/// A positioned element that can be rendered at a specific location
pub struct PositionedElement<W: Widget> {
    widget: W,
    position: Rect,
}

impl<W: Widget> PositionedElement<W> {
    /// Create a new positioned element
    pub fn new(widget: W, position: Rect) -> Self {
        Self { widget, position }
    }

    /// Render the positioned element
    pub fn render(self, frame: &mut Frame) {
        frame.render_widget(self.widget, self.position);
    }
}
