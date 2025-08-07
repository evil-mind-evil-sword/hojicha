//! Mock terminal for testing without a real terminal

use ratatui::{
    backend::{Backend, WindowSize},
    buffer::{Buffer, Cell},
    layout::{Rect, Size},
};
use std::io;

/// A mock terminal backend that captures output for testing
#[derive(Debug, Clone)]
pub struct MockTerminal {
    width: u16,
    height: u16,
    buffer: Buffer,
    cursor: (u16, u16),
    hidden_cursor: bool,
    /// Captured operations for verification
    operations: Vec<String>,
}

impl MockTerminal {
    /// Create a new mock terminal with given dimensions
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            cursor: (0, 0),
            hidden_cursor: false,
            operations: Vec::new(),
        }
    }

    /// Get the current buffer content as a string
    pub fn get_content(&self) -> String {
        let mut result = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.buffer[(x, y)];
                result.push_str(cell.symbol());
            }
            if y < self.height - 1 {
                result.push('\n');
            }
        }
        result
    }

    /// Get content of a specific line
    pub fn get_line(&self, y: u16) -> String {
        let mut result = String::new();
        for x in 0..self.width {
            let cell = &self.buffer[(x, y)];
            result.push_str(cell.symbol());
        }
        result.trim_end().to_string()
    }

    /// Check if a string appears anywhere in the terminal
    pub fn contains(&self, text: &str) -> bool {
        self.get_content().contains(text)
    }

    /// Get all recorded operations
    pub fn get_operations(&self) -> &[String] {
        &self.operations
    }

    /// Clear the terminal
    pub fn clear(&mut self) {
        self.buffer = Buffer::empty(Rect::new(0, 0, self.width, self.height));
        self.operations.push("clear".to_string());
    }
}

impl Backend for MockTerminal {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            if x < self.width && y < self.height {
                let idx = y * self.width + x;
                self.buffer.content[idx as usize] = cell.clone();
            }
        }
        self.operations.push("draw".to_string());
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.hidden_cursor = true;
        self.operations.push("hide_cursor".to_string());
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.hidden_cursor = false;
        self.operations.push("show_cursor".to_string());
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        Ok(ratatui::layout::Position {
            x: self.cursor.0,
            y: self.cursor.1,
        })
    }

    fn set_cursor_position<P: Into<ratatui::layout::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        let pos = position.into();
        self.cursor = (pos.x, pos.y);
        self.operations
            .push(format!("set_cursor({}, {})", pos.x, pos.y));
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.clear();
        Ok(())
    }

    fn size(&self) -> io::Result<ratatui::layout::Size> {
        Ok(ratatui::layout::Size {
            width: self.width,
            height: self.height,
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        self.operations.push("flush".to_string());
        Ok(())
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: Size {
                width: self.width,
                height: self.height,
            },
            pixels: Size {
                width: self.width * 8,
                height: self.height * 16,
            }, // Approximate pixel size
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{
        Terminal,
        widgets::{Block, Borders, Paragraph},
    };

    #[test]
    fn test_mock_terminal_basic() {
        let backend = MockTerminal::new(20, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                let paragraph =
                    Paragraph::new("Hello, World!").block(Block::default().borders(Borders::ALL));
                f.render_widget(paragraph, f.area());
            })
            .unwrap();

        let backend = terminal.backend();
        assert!(backend.contains("Hello, World!"));
        assert!(backend.get_operations().contains(&"draw".to_string()));
    }

    #[test]
    fn test_mock_terminal_cursor() {
        let mut backend = MockTerminal::new(20, 10);

        backend.hide_cursor().unwrap();
        assert!(backend.hidden_cursor);

        backend.show_cursor().unwrap();
        assert!(!backend.hidden_cursor);

        backend.set_cursor_position((5, 3)).unwrap();
        let pos = backend.get_cursor_position().unwrap();
        assert_eq!((pos.x, pos.y), (5, 3));
    }
}
