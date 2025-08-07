//! Scrollable viewport component
//!
//! A viewport provides a scrollable view of content that exceeds the visible area.

use crate::event::{Key, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};
use std::cmp::{max, min};

/// Scrollable viewport for displaying content
#[derive(Debug, Clone)]
pub struct Viewport {
    /// The content to display
    content: Vec<String>,
    /// Current vertical scroll offset
    y_offset: usize,
    /// Current horizontal scroll offset  
    x_offset: usize,
    /// Viewport dimensions
    width: u16,
    height: u16,
    /// Configuration options
    options: ViewportOptions,
    /// Whether the viewport has focus
    focused: bool,
}

/// Configuration options for Viewport
#[derive(Debug, Clone)]
pub struct ViewportOptions {
    /// Show vertical scrollbar
    pub show_scrollbar: bool,
    /// Scrollbar style
    pub scrollbar_style: Style,
    /// Scrollbar track style
    pub scrollbar_track_style: Style,
    /// Text style
    pub text_style: Style,
    /// Wrap long lines
    pub wrap_lines: bool,
    /// Scroll amount for mouse wheel
    pub scroll_amount: usize,
}

impl Default for ViewportOptions {
    fn default() -> Self {
        Self {
            show_scrollbar: true,
            scrollbar_style: Style::default().fg(Color::White),
            scrollbar_track_style: Style::default().fg(Color::DarkGray),
            text_style: Style::default(),
            wrap_lines: false,
            scroll_amount: 3,
        }
    }
}

impl Viewport {
    /// Create a new Viewport
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            y_offset: 0,
            x_offset: 0,
            width: 0,
            height: 0,
            options: ViewportOptions::default(),
            focused: false,
        }
    }

    /// Create a new Viewport with options
    pub fn with_options(options: ViewportOptions) -> Self {
        Self {
            options,
            ..Self::new()
        }
    }

    /// Set the content
    pub fn set_content(&mut self, content: impl Into<String>) {
        let text = content.into();
        self.content = text.lines().map(String::from).collect();
        self.y_offset = 0;
        self.x_offset = 0;
    }

    /// Append content
    pub fn append_content(&mut self, content: impl Into<String>) {
        let text = content.into();
        self.content.extend(text.lines().map(String::from));
    }

    /// Clear content
    pub fn clear(&mut self) {
        self.content.clear();
        self.y_offset = 0;
        self.x_offset = 0;
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Get the total number of lines
    pub fn line_count(&self) -> usize {
        self.content.len()
    }

    /// Get the current scroll position
    pub fn scroll_position(&self) -> (usize, usize) {
        (self.y_offset, self.x_offset)
    }

    /// Scroll to a specific line
    pub fn scroll_to_line(&mut self, line: usize) {
        self.y_offset = min(line, self.max_y_offset());
    }

    /// Scroll to the top
    pub fn scroll_to_top(&mut self) {
        self.y_offset = 0;
    }

    /// Scroll to the bottom
    pub fn scroll_to_bottom(&mut self) {
        self.y_offset = self.max_y_offset();
    }

    /// Scroll up by the given amount
    pub fn scroll_up(&mut self, amount: usize) {
        self.y_offset = self.y_offset.saturating_sub(amount);
    }

    /// Scroll down by the given amount
    pub fn scroll_down(&mut self, amount: usize) {
        self.y_offset = min(self.y_offset + amount, self.max_y_offset());
    }

    /// Scroll left by the given amount
    pub fn scroll_left(&mut self, amount: usize) {
        self.x_offset = self.x_offset.saturating_sub(amount);
    }

    /// Scroll right by the given amount
    pub fn scroll_right(&mut self, amount: usize) {
        if !self.options.wrap_lines {
            self.x_offset = min(self.x_offset + amount, self.max_x_offset());
        }
    }

    /// Get maximum y offset
    fn max_y_offset(&self) -> usize {
        self.content.len().saturating_sub(self.height as usize)
    }

    /// Get maximum x offset
    fn max_x_offset(&self) -> usize {
        if self.options.wrap_lines {
            0
        } else {
            self.content
                .iter()
                .map(|line| line.len())
                .max()
                .unwrap_or(0)
                .saturating_sub(self.width as usize)
        }
    }

    /// Calculate scrollbar position and size
    fn scrollbar_info(&self, height: u16) -> Option<(u16, u16)> {
        if !self.options.show_scrollbar || self.content.len() <= height as usize {
            return None;
        }

        let total_lines = self.content.len();
        let visible_lines = height as usize;

        // Calculate thumb size (minimum 1)
        let thumb_size = max(1, (visible_lines * height as usize) / total_lines) as u16;

        // Calculate thumb position
        let thumb_pos = if self.y_offset == 0 {
            0
        } else if self.y_offset >= self.max_y_offset() {
            height - thumb_size
        } else {
            ((self.y_offset * height as usize) / total_lines) as u16
        };

        Some((thumb_pos, thumb_size))
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        if self.focused {
            self.handle_key_event(*key)
        } else {
            false
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: &MouseEvent) -> bool {
        self.handle_mouse_event(*mouse)
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.key {
            Key::Up => {
                self.scroll_up(1);
                true
            }
            Key::Down => {
                self.scroll_down(1);
                true
            }
            Key::Left if !self.options.wrap_lines => {
                self.scroll_left(1);
                true
            }
            Key::Right if !self.options.wrap_lines => {
                self.scroll_right(1);
                true
            }
            Key::PageUp => {
                self.scroll_up(self.height as usize);
                true
            }
            Key::PageDown => {
                self.scroll_down(self.height as usize);
                true
            }
            Key::Home => {
                self.scroll_to_top();
                true
            }
            Key::End => {
                self.scroll_to_bottom();
                true
            }
            _ => false,
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::ScrollDown => {
                self.scroll_down(self.options.scroll_amount);
                true
            }
            MouseEventKind::ScrollUp => {
                self.scroll_up(self.options.scroll_amount);
                true
            }
            _ => false,
        }
    }

    /// Render the viewport to a frame
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Draw border if focused
        let block = if self.focused {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
        };

        let inner = block.inner(area);
        block.render(area, buf);

        // Calculate content area (leave space for scrollbar if shown)
        let content_width = if self.options.show_scrollbar {
            inner.width.saturating_sub(1)
        } else {
            inner.width
        };

        let content_area = Rect {
            x: inner.x,
            y: inner.y,
            width: content_width,
            height: inner.height,
        };

        // Render visible content
        let visible_lines = min(
            self.content.len().saturating_sub(self.y_offset),
            inner.height as usize,
        );

        for i in 0..visible_lines {
            let line_idx = self.y_offset + i;
            let line = &self.content[line_idx];
            let y = content_area.y + i as u16;

            // Handle line wrapping or horizontal scrolling
            let visible_text = if self.options.wrap_lines {
                // Wrap long lines
                line.clone()
            } else {
                // Horizontal scrolling
                line.chars()
                    .skip(self.x_offset)
                    .take(content_width as usize)
                    .collect()
            };

            let styled_line = Line::from(Span::styled(visible_text, self.options.text_style));
            buf.set_line(content_area.x, y, &styled_line, content_width);
        }

        // Render scrollbar
        if let Some((thumb_pos, thumb_size)) = self.scrollbar_info(inner.height) {
            let scrollbar_x = inner.x + inner.width - 1;

            // Draw scrollbar track
            for y in 0..inner.height {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('│')
                    .set_style(self.options.scrollbar_track_style);
            }

            // Draw scrollbar thumb
            for y in thumb_pos..(thumb_pos + thumb_size) {
                buf[(scrollbar_x, inner.y + y)]
                    .set_char('█')
                    .set_style(self.options.scrollbar_style);
            }
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::backend::TestBackend;

    use ratatui::Terminal;
    use ratatui::layout::Rect;

    #[test]
    fn test_viewport_creation() {
        let viewport = Viewport::new();
        assert_eq!(viewport.line_count(), 0);
        assert_eq!(viewport.scroll_position(), (0, 0));
        assert_eq!(viewport.height, 0);
        assert_eq!(viewport.x_offset, 0);
        assert_eq!(viewport.y_offset, 0);
    }

    #[test]
    fn test_viewport_default() {
        let viewport = Viewport::default();
        assert_eq!(viewport.line_count(), 0);
        assert_eq!(viewport.scroll_position(), (0, 0));
    }

    #[test]
    fn test_content_management() {
        let mut viewport = Viewport::new();
        viewport.set_content("Line 1\nLine 2\nLine 3");
        assert_eq!(viewport.line_count(), 3);

        viewport.append_content("\nLine 4\nLine 5");
        assert_eq!(viewport.line_count(), 6); // 3 original + 1 empty + 2 new

        viewport.clear();
        assert_eq!(viewport.line_count(), 0);
    }

    #[test]
    fn test_scrolling() {
        let mut viewport = Viewport::new();
        viewport.height = 10;
        viewport.set_content(
            (0..20)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        viewport.scroll_down(5);
        assert_eq!(viewport.y_offset, 5);

        viewport.scroll_to_bottom();
        assert_eq!(viewport.y_offset, 10);

        viewport.scroll_up(3);
        assert_eq!(viewport.y_offset, 7);

        viewport.scroll_to_top();
        assert_eq!(viewport.y_offset, 0);
    }

    #[test]
    fn test_mouse_scrolling() {
        let mut viewport = Viewport::new();
        viewport.height = 10;
        viewport.set_content(
            (0..20)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let mouse_event = MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: crate::event::KeyModifiers::empty(),
        };

        assert!(viewport.handle_mouse(&mouse_event));
        assert_eq!(viewport.y_offset, 3); // Default scroll amount
    }

    #[test]
    fn test_horizontal_scrolling() {
        let mut viewport = Viewport::new();
        viewport.set_content("Very long line that exceeds normal width");

        viewport.scroll_right(5);
        assert_eq!(viewport.x_offset, 5);

        viewport.scroll_left(2);
        assert_eq!(viewport.x_offset, 3);

        viewport.scroll_left(10); // Should clamp to 0
        assert_eq!(viewport.x_offset, 0);
    }

    #[test]
    fn test_scroll_bounds() {
        let mut viewport = Viewport::new();
        viewport.height = 5;
        viewport.set_content("Line 1\nLine 2\nLine 3"); // Only 3 lines

        // Should not scroll beyond content
        viewport.scroll_down(10);
        assert_eq!(viewport.y_offset, 0); // Can't scroll down with so few lines

        // Add more content
        viewport.set_content(
            (0..10)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        viewport.scroll_down(10);
        assert_eq!(viewport.y_offset, 5); // height - content should be max offset

        viewport.scroll_up(10);
        assert_eq!(viewport.y_offset, 0);
    }

    #[test]
    fn test_mouse_events() {
        let mut viewport = Viewport::new();
        viewport.height = 5;
        viewport.set_content(
            (0..10)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        // Test scroll up
        viewport.y_offset = 3;
        let scroll_up = MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 0,
            row: 0,
            modifiers: crate::event::KeyModifiers::empty(),
        };
        assert!(viewport.handle_mouse(&scroll_up));
        assert_eq!(viewport.y_offset, 0);

        // Test other mouse events (should return false)
        let click = MouseEvent {
            kind: MouseEventKind::Down(crate::event::MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: crate::event::KeyModifiers::empty(),
        };
        assert!(!viewport.handle_mouse(&click));
    }

    #[test]
    fn test_options() {
        let options = ViewportOptions {
            wrap_lines: false,
            show_scrollbar: true,
            text_style: Style::default(),
            scrollbar_style: Style::default().fg(Color::Blue),
            scrollbar_track_style: Style::default().fg(Color::Gray),
            scroll_amount: 3,
        };

        let viewport = Viewport::with_options(options.clone());
        assert!(!viewport.options.wrap_lines);
        assert!(viewport.options.show_scrollbar);
        assert_eq!(viewport.options.scroll_amount, 3);
    }

    #[test]
    fn test_scrollbar_info() {
        let mut viewport = Viewport::new();
        viewport.height = 5;
        viewport.set_content(
            (0..10)
                .map(|i| format!("Line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let info = viewport.scrollbar_info(5);
        assert!(info.is_some());

        let (thumb_pos, thumb_size) = info.unwrap();
        assert!(thumb_pos < 5);
        assert!(thumb_size > 0);
        assert!(thumb_pos + thumb_size <= 5);
    }

    #[test]
    fn test_empty_content() {
        let mut viewport = Viewport::new();
        assert_eq!(viewport.line_count(), 0);

        viewport.scroll_down(5);
        assert_eq!(viewport.y_offset, 0); // Can't scroll empty content

        viewport.set_content("");
        assert_eq!(viewport.line_count(), 0); // Empty string creates no lines
    }

    #[test]
    fn test_rendering() {
        let mut viewport = Viewport::new();
        viewport.set_content("Line 1\nLine 2\nLine 3");

        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 20, 10);
                viewport.render(area, frame.buffer_mut());
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content().is_empty());
    }
}
