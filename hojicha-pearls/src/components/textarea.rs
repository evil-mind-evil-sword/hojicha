//! Multi-line text input component
//!
//! A TextArea provides multi-line text editing capabilities with features like:
//! - Cursor movement and selection
//! - Copy/paste support
//! - Placeholder text
//! - Line numbers
//! - Word wrapping

use hojicha_core::event::{Key, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};
use std::cmp::min;

/// Multi-line text input component
#[derive(Debug, Clone)]
pub struct TextArea {
    /// The text content split by lines
    lines: Vec<String>,
    /// Current cursor position (line, column)
    cursor: (usize, usize),
    /// Selection anchor if text is selected
    selection_anchor: Option<(usize, usize)>,
    /// Viewport offset for scrolling
    viewport_offset: (usize, usize),
    /// Component dimensions
    #[allow(dead_code)]
    width: u16,
    #[allow(dead_code)]
    height: u16,
    /// Configuration options
    options: TextAreaOptions,
    /// Whether the component has focus
    focused: bool,
}

/// Configuration options for TextArea
#[derive(Debug, Clone)]
pub struct TextAreaOptions {
    /// Show line numbers
    pub show_line_numbers: bool,
    /// Line number style
    pub line_number_style: Style,
    /// Placeholder text when empty
    pub placeholder: String,
    /// Placeholder style
    pub placeholder_style: Style,
    /// Cursor style
    pub cursor_style: Style,
    /// Selection style
    pub selection_style: Style,
    /// Text style
    pub text_style: Style,
    /// Enable word wrap
    pub word_wrap: bool,
    /// Tab size in spaces
    pub tab_size: usize,
}

impl Default for TextAreaOptions {
    fn default() -> Self {
        Self {
            show_line_numbers: false,
            line_number_style: Style::default().fg(Color::DarkGray),
            placeholder: String::new(),
            placeholder_style: Style::default().fg(Color::DarkGray),
            cursor_style: Style::default().bg(Color::White).fg(Color::Black),
            selection_style: Style::default().bg(Color::Blue),
            text_style: Style::default(),
            word_wrap: true,
            tab_size: 4,
        }
    }
}

impl TextArea {
    /// Create a new TextArea
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            selection_anchor: None,
            viewport_offset: (0, 0),
            width: 0,
            height: 0,
            options: TextAreaOptions::default(),
            focused: false,
        }
    }

    /// Create a new TextArea with options
    pub fn with_options(options: TextAreaOptions) -> Self {
        Self {
            options,
            ..Self::new()
        }
    }

    /// Set the placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.options.placeholder = placeholder.into();
        self
    }

    /// Enable or disable line numbers
    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.options.show_line_numbers = show;
        self
    }

    /// Set the text content
    pub fn set_value(&mut self, value: impl Into<String>) {
        let text = value.into();
        self.lines = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };
        self.cursor = (0, 0);
        self.selection_anchor = None;
    }

    /// Get the text content
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Check if the component has focus
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Insert text at the current cursor position
    pub fn insert_text(&mut self, text: &str) {
        self.delete_selection();

        let (line_idx, col_idx) = self.cursor;
        let current_line = &mut self.lines[line_idx];

        if text.contains('\n') {
            // Multi-line insert
            let mut new_lines: Vec<String> = text.lines().map(String::from).collect();
            if new_lines.is_empty() {
                return;
            }

            // Split current line at cursor
            let before = current_line[..col_idx].to_string();
            let after = current_line[col_idx..].to_string();

            // First line gets prepended with before
            new_lines[0] = before + &new_lines[0];

            // Last line gets appended with after
            let last_idx = new_lines.len() - 1;
            new_lines[last_idx].push_str(&after);

            // Replace current line with new lines
            self.lines.splice(line_idx..=line_idx, new_lines);

            // Update cursor to end of inserted text
            self.cursor = (
                line_idx + last_idx,
                self.lines[line_idx + last_idx].len() - after.len(),
            );
        } else {
            // Single line insert
            current_line.insert_str(col_idx, text);
            self.cursor.1 += text.len();
        }
    }

    /// Delete the character before the cursor (backspace)
    pub fn delete_backward(&mut self) {
        if self.delete_selection() {
            return;
        }

        let (line_idx, col_idx) = self.cursor;

        if col_idx > 0 {
            // Delete within line
            self.lines[line_idx].remove(col_idx - 1);
            self.cursor.1 -= 1;
        } else if line_idx > 0 {
            // Join with previous line
            let current_line = self.lines.remove(line_idx);
            let prev_line_len = self.lines[line_idx - 1].len();
            self.lines[line_idx - 1].push_str(&current_line);
            self.cursor = (line_idx - 1, prev_line_len);
        }
    }

    /// Delete the character at the cursor (delete)
    pub fn delete_forward(&mut self) {
        if self.delete_selection() {
            return;
        }

        let (line_idx, col_idx) = self.cursor;
        let current_line = &mut self.lines[line_idx];

        if col_idx < current_line.len() {
            // Delete within line
            current_line.remove(col_idx);
        } else if line_idx < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(line_idx + 1);
            self.lines[line_idx].push_str(&next_line);
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self) -> bool {
        if let Some(anchor) = self.selection_anchor {
            let (start, end) = self.get_selection_range(anchor);

            // Delete selected text
            if start.0 == end.0 {
                // Selection within single line
                let line = &mut self.lines[start.0];
                line.replace_range(start.1..end.1, "");
            } else {
                // Multi-line selection
                let mut new_line = String::new();
                new_line.push_str(&self.lines[start.0][..start.1]);
                new_line.push_str(&self.lines[end.0][end.1..]);

                self.lines.splice(start.0..=end.0, vec![new_line]);
            }

            self.cursor = start;
            self.selection_anchor = None;
            true
        } else {
            false
        }
    }

    /// Get normalized selection range
    fn get_selection_range(&self, anchor: (usize, usize)) -> ((usize, usize), (usize, usize)) {
        let cursor = self.cursor;
        if anchor.0 < cursor.0 || (anchor.0 == cursor.0 && anchor.1 < cursor.1) {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        }
    }

    /// Move cursor up
    pub fn move_cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = min(self.cursor.1, self.lines[self.cursor.0].len());
        }
    }

    /// Move cursor down
    pub fn move_cursor_down(&mut self) {
        if self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = min(self.cursor.1, self.lines[self.cursor.0].len());
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.lines[self.cursor.0].len();
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        let line_len = self.lines[self.cursor.0].len();
        if self.cursor.1 < line_len {
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    /// Move cursor to start of line
    pub fn move_cursor_home(&mut self) {
        self.cursor.1 = 0;
    }

    /// Move cursor to end of line
    pub fn move_cursor_end(&mut self) {
        self.cursor.1 = self.lines[self.cursor.0].len();
    }

    /// Insert a new line at cursor
    pub fn insert_newline(&mut self) {
        let (line_idx, col_idx) = self.cursor;
        let current_line = &mut self.lines[line_idx];

        let new_line = current_line[col_idx..].to_string();
        current_line.truncate(col_idx);

        self.lines.insert(line_idx + 1, new_line);
        self.cursor = (line_idx + 1, 0);
    }

    /// Handle input events
    pub fn handle_event(&mut self, event: &KeyEvent) -> bool {
        if !self.focused {
            return false;
        }

        self.handle_key_event(*event)
    }

    /// Handle paste events
    pub fn handle_paste(&mut self, text: &str) -> bool {
        if !self.focused {
            return false;
        }

        self.insert_text(text);
        true
    }

    /// Handle key events
    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Update selection anchor for shift+movement
        if shift
            && matches!(
                key.key,
                Key::Up | Key::Down | Key::Left | Key::Right | Key::Home | Key::End
            )
        {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.cursor);
            }
        } else if !shift && self.selection_anchor.is_some() {
            self.selection_anchor = None;
        }

        match key.key {
            Key::Char(c) if !ctrl => {
                self.insert_text(&c.to_string());
                true
            }
            Key::Enter => {
                self.insert_newline();
                true
            }
            Key::Backspace => {
                self.delete_backward();
                true
            }
            Key::Delete => {
                self.delete_forward();
                true
            }
            Key::Up => {
                self.move_cursor_up();
                true
            }
            Key::Down => {
                self.move_cursor_down();
                true
            }
            Key::Left => {
                self.move_cursor_left();
                true
            }
            Key::Right => {
                self.move_cursor_right();
                true
            }
            Key::Home => {
                self.move_cursor_home();
                true
            }
            Key::End => {
                self.move_cursor_end();
                true
            }
            Key::Tab => {
                self.insert_text(&" ".repeat(self.options.tab_size));
                true
            }
            _ => false,
        }
    }

    /// Render the TextArea to a frame
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

        // Calculate content area
        let line_number_width = if self.options.show_line_numbers {
            (self.lines.len().to_string().len() + 2) as u16
        } else {
            0
        };

        let content_area = Rect {
            x: inner.x + line_number_width,
            y: inner.y,
            width: inner.width.saturating_sub(line_number_width),
            height: inner.height,
        };

        // Render placeholder if empty
        if self.lines.len() == 1 && self.lines[0].is_empty() && !self.focused {
            if !self.options.placeholder.is_empty() {
                let placeholder_line = Line::from(Span::styled(
                    &self.options.placeholder,
                    self.options.placeholder_style,
                ));
                buf.set_line(
                    content_area.x,
                    content_area.y,
                    &placeholder_line,
                    content_area.width,
                );
            }
            return;
        }

        // Update viewport to follow cursor
        let visible_lines = inner.height as usize;
        let cursor_line = self.cursor.0;

        let viewport_start = if cursor_line < self.viewport_offset.0 {
            cursor_line
        } else if cursor_line >= self.viewport_offset.0 + visible_lines {
            cursor_line.saturating_sub(visible_lines - 1)
        } else {
            self.viewport_offset.0
        };

        // Render visible lines
        for (i, line_idx) in
            (viewport_start..min(viewport_start + visible_lines, self.lines.len())).enumerate()
        {
            let y = inner.y + i as u16;

            // Render line number
            if self.options.show_line_numbers {
                let line_num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = line_number_width as usize - 2
                );
                let line_num_span = Span::styled(line_num, self.options.line_number_style);
                buf.set_span(inner.x, y, &line_num_span, line_number_width);
            }

            // Render line content
            let line = &self.lines[line_idx];
            let mut spans = vec![];

            // Apply selection highlighting
            if let Some(anchor) = self.selection_anchor {
                let (start, end) = self.get_selection_range(anchor);

                if line_idx >= start.0 && line_idx <= end.0 {
                    let line_start = if line_idx == start.0 { start.1 } else { 0 };
                    let line_end = if line_idx == end.0 { end.1 } else { line.len() };

                    if line_start > 0 {
                        spans.push(Span::styled(&line[..line_start], self.options.text_style));
                    }
                    if line_end > line_start {
                        spans.push(Span::styled(
                            &line[line_start..line_end],
                            self.options.selection_style,
                        ));
                    }
                    if line_end < line.len() {
                        spans.push(Span::styled(&line[line_end..], self.options.text_style));
                    }
                } else {
                    spans.push(Span::styled(line, self.options.text_style));
                }
            } else {
                spans.push(Span::styled(line, self.options.text_style));
            }

            // Render cursor
            if self.focused && line_idx == cursor_line {
                let cursor_x = content_area.x + self.cursor.1.min(line.len()) as u16;
                if cursor_x < content_area.x + content_area.width {
                    let cursor_char = if self.cursor.1 < line.len() {
                        line.chars().nth(self.cursor.1).unwrap_or(' ')
                    } else {
                        ' '
                    };
                    buf[(cursor_x, y)]
                        .set_char(cursor_char)
                        .set_style(self.options.cursor_style);
                }
            }

            let line_widget = Line::from(spans);
            buf.set_line(content_area.x, y, &line_widget, content_area.width);
        }
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ratatui::backend::TestBackend;

    use ratatui::layout::Rect;
    use ratatui::Terminal;

    #[test]
    fn test_textarea_creation() {
        let textarea = TextArea::new();
        assert_eq!(textarea.value(), "");
        assert_eq!(textarea.cursor(), (0, 0));
        assert!(!textarea.focused);
        assert!(textarea.selection_anchor.is_none());
    }

    #[test]
    fn test_textarea_default() {
        let textarea = TextArea::default();
        assert_eq!(textarea.value(), "");
        assert_eq!(textarea.cursor(), (0, 0));
    }

    #[test]
    fn test_insert_text() {
        let mut textarea = TextArea::new();
        textarea.insert_text("Hello");
        assert_eq!(textarea.value(), "Hello");
        assert_eq!(textarea.cursor(), (0, 5));

        textarea.insert_text(" World");
        assert_eq!(textarea.value(), "Hello World");
        assert_eq!(textarea.cursor(), (0, 11));
    }

    #[test]
    fn test_multiline_insert() {
        let mut textarea = TextArea::new();
        textarea.insert_text("Line 1\nLine 2\nLine 3");
        assert_eq!(textarea.value(), "Line 1\nLine 2\nLine 3");
        assert_eq!(textarea.cursor(), (2, 6));
    }

    #[test]
    fn test_cursor_movement() {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello\nWorld");

        textarea.move_cursor_down();
        assert_eq!(textarea.cursor(), (1, 0));

        textarea.move_cursor_end();
        assert_eq!(textarea.cursor(), (1, 5));

        textarea.move_cursor_up();
        assert_eq!(textarea.cursor(), (0, 5));

        textarea.move_cursor_home();
        assert_eq!(textarea.cursor(), (0, 0));
    }

    #[test]
    fn test_delete_operations() {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello World");
        textarea.cursor = (0, 6); // Cursor after space

        // Delete backward should delete the space
        textarea.delete_backward();
        assert_eq!(textarea.value(), "HelloWorld");
        assert_eq!(textarea.cursor, (0, 5));

        // Delete forward at position 5 should delete 'W'
        textarea.delete_forward();
        assert_eq!(textarea.value(), "Helloorld");
    }

    #[test]
    fn test_set_value() {
        let mut textarea = TextArea::new();
        textarea.set_value("Test content");
        assert_eq!(textarea.value(), "Test content");
        assert_eq!(textarea.cursor(), (0, 0)); // Cursor resets to start
    }

    #[test]
    fn test_focus() {
        let mut textarea = TextArea::new();
        assert!(!textarea.focused);

        textarea.set_focused(true);
        assert!(textarea.focused);

        textarea.set_focused(false);
        assert!(!textarea.focused);
    }

    #[test]
    fn test_cursor_bounds() {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello\nWorld");

        // Test moving beyond bounds
        textarea.cursor = (0, 0);
        textarea.move_cursor_left(); // Should stay at (0, 0)
        assert_eq!(textarea.cursor(), (0, 0));

        textarea.move_cursor_up(); // Should stay at (0, 0)
        assert_eq!(textarea.cursor(), (0, 0));

        // Move to end and try to go further
        textarea.cursor = (1, 5);
        textarea.move_cursor_right(); // Should stay at (1, 5)
        assert_eq!(textarea.cursor(), (1, 5));

        textarea.move_cursor_down(); // Should stay at (1, 5)
        assert_eq!(textarea.cursor(), (1, 5));
    }

    #[test]
    fn test_delete_at_boundaries() {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello\nWorld");

        // Delete backward at start of document
        textarea.cursor = (0, 0);
        textarea.delete_backward();
        assert_eq!(textarea.value(), "Hello\nWorld"); // No change

        // Delete forward at end of document
        textarea.cursor = (1, 5);
        textarea.delete_forward();
        assert_eq!(textarea.value(), "Hello\nWorld"); // No change
    }

    #[test]
    fn test_multiline_cursor_movement() {
        let mut textarea = TextArea::new();
        textarea.set_value("Short\nVery long line\nEnd");

        // Start at end of first line
        textarea.cursor = (0, 5);
        textarea.move_cursor_down();
        assert_eq!(textarea.cursor(), (1, 5)); // Should maintain column

        textarea.move_cursor_down();
        assert_eq!(textarea.cursor(), (2, 3)); // Should clamp to line length

        textarea.move_cursor_up();
        assert_eq!(textarea.cursor(), (1, 3)); // Should maintain clamped position
    }

    #[test]
    fn test_empty_lines() {
        let mut textarea = TextArea::new();
        textarea.set_value("Line1\n\nLine3");

        textarea.cursor = (1, 0); // On empty line
        textarea.move_cursor_end();
        assert_eq!(textarea.cursor(), (1, 0)); // Should stay at start of empty line

        textarea.insert_text("Middle");
        assert_eq!(textarea.value(), "Line1\nMiddle\nLine3");
    }

    #[test]
    fn test_options() {
        let options = TextAreaOptions {
            show_line_numbers: true,
            line_number_style: Style::default(),
            placeholder: "Enter text...".to_string(),
            placeholder_style: Style::default(),
            word_wrap: true,
            tab_size: 4,
            text_style: Style::default(),
            cursor_style: Style::default().bg(Color::White),
            selection_style: Style::default().bg(Color::Blue),
        };

        let textarea = TextArea::with_options(options.clone());
        assert_eq!(textarea.options.placeholder, "Enter text...");
        assert!(textarea.options.show_line_numbers);
        assert!(textarea.options.word_wrap);
        assert_eq!(textarea.options.tab_size, 4);
    }

    #[test]
    fn test_rendering() {
        let mut textarea = TextArea::new();
        textarea.set_value("Hello\nWorld");
        textarea.set_focused(true);

        let backend = TestBackend::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 20, 10);
                textarea.render(area, frame.buffer_mut());
            })
            .unwrap();

        // Verify the terminal was drawn to (basic smoke test)
        let buffer = terminal.backend().buffer();
        assert!(!buffer.content().is_empty());
    }
}
