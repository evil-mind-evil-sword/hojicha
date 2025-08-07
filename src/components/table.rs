//! Table component with row selection and scrolling
//!
//! A table provides a structured view of data with columns, headers, and row selection.

use crate::event::{Key, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Cell, Row, Table as RatatuiTable, Widget};
use std::cmp::{max, min};

/// Options for customizing table behavior
#[derive(Debug, Clone)]
pub struct TableOptions {
    /// Style for headers
    pub header_style: Style,
    /// Style for normal rows
    pub row_style: Style,
    /// Style for the selected row
    pub selected_style: Style,
    /// Whether to highlight the selected row
    pub highlight_selection: bool,
    /// Whether the table wraps around at the ends
    pub wrap_around: bool,
    /// Number of rows to skip when page up/down
    pub page_size: usize,
    /// Show row numbers
    pub show_row_numbers: bool,
    /// Column constraints
    pub column_constraints: Vec<Constraint>,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self {
            header_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            row_style: Style::default(),
            selected_style: Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
            highlight_selection: true,
            wrap_around: false,
            page_size: 10,
            show_row_numbers: false,
            column_constraints: vec![],
        }
    }
}

/// A table component with selection support
#[derive(Clone)]
pub struct Table<T> {
    /// Column headers
    headers: Vec<String>,
    /// Table data
    rows: Vec<T>,
    /// Currently selected row index
    selected: usize,
    /// Viewport offset (first visible row)
    offset: usize,
    /// Whether the table has focus
    focused: bool,
    /// Visible height (set during render)
    height: usize,
    /// Table options
    options: TableOptions,
    /// Optional block for borders/title
    block: Option<Block<'static>>,
}

impl<T> Table<T> {
    /// Create a new table with headers
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
            selected: 0,
            offset: 0,
            focused: false,
            height: 10,
            options: TableOptions::default(),
            block: None,
        }
    }

    /// Set the table options
    pub fn with_options(mut self, options: TableOptions) -> Self {
        self.options = options;
        self
    }

    /// Set the block (borders/title)
    pub fn with_block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the rows
    pub fn with_rows(mut self, rows: Vec<T>) -> Self {
        self.rows = rows;
        self
    }

    /// Get the currently selected index
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Get a reference to the selected row
    pub fn selected_row(&self) -> Option<&T> {
        self.rows.get(self.selected)
    }

    /// Get a mutable reference to the selected row
    pub fn selected_row_mut(&mut self) -> Option<&mut T> {
        self.rows.get_mut(self.selected)
    }

    /// Get the number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Set whether the table has focus
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Select a specific row
    pub fn select(&mut self, index: usize) {
        if index < self.rows.len() {
            self.selected = index;
            self.ensure_visible();
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.rows.is_empty() {
            return;
        }

        if self.selected > 0 {
            self.selected -= 1;
        } else if self.options.wrap_around {
            self.selected = self.rows.len() - 1;
        }
        self.ensure_visible();
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.rows.is_empty() {
            return;
        }

        if self.selected + 1 < self.rows.len() {
            self.selected += 1;
        } else if self.options.wrap_around {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection up by page
    pub fn page_up(&mut self) {
        if self.selected > self.options.page_size {
            self.selected -= self.options.page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection down by page
    pub fn page_down(&mut self) {
        self.selected = min(
            self.selected + self.options.page_size,
            self.rows.len().saturating_sub(1),
        );
        self.ensure_visible();
    }

    /// Select the first row
    pub fn select_first(&mut self) {
        self.selected = 0;
        self.ensure_visible();
    }

    /// Select the last row
    pub fn select_last(&mut self) {
        if !self.rows.is_empty() {
            self.selected = self.rows.len() - 1;
            self.ensure_visible();
        }
    }

    /// Ensure the selected row is visible
    fn ensure_visible(&mut self) {
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + self.height {
            self.offset = self.selected.saturating_sub(self.height - 1);
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: &KeyEvent) -> bool {
        if !self.focused {
            return false;
        }

        match key.key {
            Key::Up | Key::Char('k') => {
                self.select_previous();
                true
            }
            Key::Down | Key::Char('j') => {
                self.select_next();
                true
            }
            Key::PageUp => {
                self.page_up();
                true
            }
            Key::PageDown => {
                self.page_down();
                true
            }
            Key::Home => {
                self.select_first();
                true
            }
            Key::End => {
                self.select_last();
                true
            }
            _ => false,
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: &MouseEvent, area: Rect) -> bool {
        if !self.focused {
            return false;
        }

        // Calculate inner area (accounting for borders)
        let inner = if self.block.is_some() {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        // Account for header row
        let content_y = inner.y + 2; // Header + separator

        // Check if click is within the table area
        if mouse.column < inner.x
            || mouse.column >= inner.x + inner.width
            || mouse.row < content_y
            || mouse.row >= inner.y + inner.height
        {
            return false;
        }

        match mouse.kind {
            MouseEventKind::Down(_) => {
                // Calculate which row was clicked
                if mouse.row >= content_y {
                    let clicked_index = (mouse.row - content_y) as usize + self.offset;
                    if clicked_index < self.rows.len() {
                        self.selected = clicked_index;
                        return true;
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                self.select_previous();
                return true;
            }
            MouseEventKind::ScrollDown => {
                self.select_next();
                return true;
            }
            _ => {}
        }

        false
    }

    /// Get the rows as a slice
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Get the rows as a mutable slice
    pub fn rows_mut(&mut self) -> &mut [T] {
        &mut self.rows
    }

    /// Add a row to the table
    pub fn push(&mut self, row: T) {
        self.rows.push(row);
    }

    /// Remove the selected row
    pub fn remove_selected(&mut self) -> Option<T> {
        if self.rows.is_empty() {
            return None;
        }

        let removed = self.rows.remove(self.selected);

        // Adjust selection
        if self.selected >= self.rows.len() && !self.rows.is_empty() {
            self.selected = self.rows.len() - 1;
        }

        self.ensure_visible();
        Some(removed)
    }

    /// Clear all rows
    pub fn clear(&mut self) {
        self.rows.clear();
        self.selected = 0;
        self.offset = 0;
    }
}

/// Trait for types that can be rendered as table rows
pub trait TableRow {
    /// Convert the item to table cells
    fn to_row(&self) -> Vec<String>;
}

impl<T: TableRow> Table<T> {
    /// Render the table to a buffer
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        // Draw block if present
        let inner = if let Some(ref block) = self.block {
            let widget = block.clone();
            widget.render(area, buf);
            block.inner(area)
        } else {
            area
        };

        // Calculate available height for rows (minus header and separator)
        self.height = inner.height.saturating_sub(2) as usize;

        // Calculate visible range
        let end = min(self.offset + self.height, self.rows.len());

        // Prepare headers
        let header_cells: Vec<Cell> = if self.options.show_row_numbers {
            std::iter::once(Cell::from("#"))
                .chain(self.headers.iter().map(|h| Cell::from(h.as_str())))
                .collect()
        } else {
            self.headers
                .iter()
                .map(|h| Cell::from(h.as_str()))
                .collect()
        };

        let header = Row::new(header_cells).style(self.options.header_style);

        // Prepare visible rows
        let rows: Vec<Row> = (self.offset..end)
            .map(|row_idx| {
                let row_data = &self.rows[row_idx];
                let cells_data = row_data.to_row();

                let cells: Vec<Cell> = if self.options.show_row_numbers {
                    std::iter::once(Cell::from(format!("{}", row_idx + 1)))
                        .chain(cells_data.into_iter().map(Cell::from))
                        .collect()
                } else {
                    cells_data.into_iter().map(Cell::from).collect()
                };

                let style = if row_idx == self.selected && self.options.highlight_selection {
                    self.options.selected_style
                } else {
                    self.options.row_style
                };

                Row::new(cells).style(style)
            })
            .collect();

        // Calculate constraints
        let constraints = if self.options.column_constraints.is_empty() {
            // Auto-size columns
            let col_count = if self.options.show_row_numbers {
                self.headers.len() + 1
            } else {
                self.headers.len()
            };
            vec![Constraint::Percentage((100 / col_count as u16).max(1)); col_count]
        } else {
            self.options.column_constraints.clone()
        };

        // Create and render table
        let table = RatatuiTable::new(rows, constraints)
            .header(header)
            .column_spacing(1);

        Widget::render(table, inner, buf);

        // Draw scrollbar if needed
        if self.rows.len() > self.height {
            let scrollbar_x = inner.x + inner.width - 1;
            let scrollbar_height = inner.height.saturating_sub(2); // Minus header
            let scrollbar_y = inner.y + 2; // After header

            // Calculate thumb size and position
            let thumb_height = max(
                1,
                (self.height * scrollbar_height as usize) / self.rows.len(),
            ) as u16;
            let thumb_pos = ((self.offset * scrollbar_height as usize) / self.rows.len()) as u16;

            // Draw scrollbar track
            for y in 0..scrollbar_height {
                buf[(scrollbar_x, scrollbar_y + y)]
                    .set_char('│')
                    .set_style(Style::default().fg(Color::DarkGray));
            }

            // Draw scrollbar thumb
            for y in thumb_pos..min(thumb_pos + thumb_height, scrollbar_height) {
                buf[(scrollbar_x, scrollbar_y + y)]
                    .set_char('█')
                    .set_style(Style::default().fg(Color::Gray));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::widgets::Borders;

    #[derive(Clone, Debug, PartialEq)]
    struct TestRow {
        id: u32,
        name: String,
        value: f64,
    }

    impl TableRow for TestRow {
        fn to_row(&self) -> Vec<String> {
            vec![
                self.id.to_string(),
                self.name.clone(),
                format!("{:.2}", self.value),
            ]
        }
    }

    #[test]
    fn test_table_creation() {
        let headers = vec!["ID".to_string(), "Name".to_string(), "Value".to_string()];
        let table: Table<TestRow> = Table::new(headers);

        assert_eq!(table.len(), 0);
        assert_eq!(table.selected(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_table_navigation() {
        let headers = vec!["ID".to_string(), "Name".to_string(), "Value".to_string()];
        let rows = vec![
            TestRow {
                id: 1,
                name: "A".to_string(),
                value: 1.0,
            },
            TestRow {
                id: 2,
                name: "B".to_string(),
                value: 2.0,
            },
            TestRow {
                id: 3,
                name: "C".to_string(),
                value: 3.0,
            },
            TestRow {
                id: 4,
                name: "D".to_string(),
                value: 4.0,
            },
        ];

        let mut table = Table::new(headers).with_rows(rows);

        // Test next
        table.select_next();
        assert_eq!(table.selected(), 1);
        assert_eq!(table.selected_row().unwrap().name, "B");

        // Test previous
        table.select_previous();
        assert_eq!(table.selected(), 0);

        // Test bounds without wrap
        table.select_previous();
        assert_eq!(table.selected(), 0); // Should stay at 0

        // Test last
        table.select_last();
        assert_eq!(table.selected(), 3);

        // Test bounds at end
        table.select_next();
        assert_eq!(table.selected(), 3); // Should stay at 3
    }

    #[test]
    fn test_table_wrap_around() {
        let headers = vec!["ID".to_string()];
        let rows = vec![
            TestRow {
                id: 1,
                name: "A".to_string(),
                value: 1.0,
            },
            TestRow {
                id: 2,
                name: "B".to_string(),
                value: 2.0,
            },
            TestRow {
                id: 3,
                name: "C".to_string(),
                value: 3.0,
            },
        ];

        let mut table = Table::new(headers)
            .with_rows(rows)
            .with_options(TableOptions {
                wrap_around: true,
                ..Default::default()
            });

        // Test wrap from beginning
        table.select_previous();
        assert_eq!(table.selected(), 2);

        // Test wrap from end
        table.select_next();
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn test_table_modification() {
        let headers = vec!["ID".to_string()];
        let mut table: Table<TestRow> = Table::new(headers);

        // Test push
        table.push(TestRow {
            id: 1,
            name: "A".to_string(),
            value: 1.0,
        });
        table.push(TestRow {
            id: 2,
            name: "B".to_string(),
            value: 2.0,
        });
        table.push(TestRow {
            id: 3,
            name: "C".to_string(),
            value: 3.0,
        });
        assert_eq!(table.len(), 3);

        // Test remove
        table.select(1); // Select "B"
        let removed = table.remove_selected();
        assert_eq!(removed.unwrap().name, "B");
        assert_eq!(table.len(), 2);
        assert_eq!(table.selected(), 1); // Should now point to "C"

        // Test clear
        table.clear();
        assert!(table.is_empty());
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn test_table_options() {
        let options = TableOptions {
            show_row_numbers: true,
            highlight_selection: false,
            wrap_around: true,
            page_size: 10,
            column_constraints: vec![Constraint::Length(10), Constraint::Min(20)],
            header_style: Style::default().fg(Color::Blue),
            row_style: Style::default().fg(Color::Green),
            selected_style: Style::default().bg(Color::Yellow),
        };

        let headers = vec!["ID".to_string()];
        let table: Table<TestRow> = Table::new(headers).with_options(options.clone());

        assert!(table.options.show_row_numbers);
        assert!(!table.options.highlight_selection);
        assert!(table.options.wrap_around);
        assert_eq!(table.options.column_constraints.len(), 2);
    }

    #[test]
    fn test_table_builder_pattern() {
        let headers = vec!["ID".to_string(), "Name".to_string()];
        let rows = vec![TestRow {
            id: 1,
            name: "Test".to_string(),
            value: 1.5,
        }];

        let block = Block::default().borders(Borders::ALL).title("My Table");

        let table = Table::new(headers)
            .with_rows(rows.clone())
            .with_block(block.clone())
            .with_options(TableOptions {
                show_row_numbers: true,
                ..Default::default()
            });

        assert_eq!(table.len(), 1);
        assert!(table.block.is_some());
        assert!(table.options.show_row_numbers);
    }

    #[test]
    fn test_table_selected_row() {
        let headers = vec!["ID".to_string()];
        let rows = vec![
            TestRow {
                id: 1,
                name: "A".to_string(),
                value: 1.0,
            },
            TestRow {
                id: 2,
                name: "B".to_string(),
                value: 2.0,
            },
        ];

        let mut table = Table::new(headers).with_rows(rows);

        // Test initial selection
        assert_eq!(table.selected_row().unwrap().id, 1);

        // Test after selection change
        table.select_next();
        assert_eq!(table.selected_row().unwrap().id, 2);

        // Test empty table
        table.clear();
        assert!(table.selected_row().is_none());
    }

    #[test]
    fn test_table_offset_management() {
        let headers = vec!["ID".to_string()];
        let rows: Vec<TestRow> = (1..=20)
            .map(|i| TestRow {
                id: i,
                name: format!("Item {i}"),
                value: i as f64,
            })
            .collect();

        let mut table = Table::new(headers).with_rows(rows);
        table.height = 5; // Simulate small viewport

        // Initial offset should be 0
        assert_eq!(table.offset, 0);

        // Navigate down past visible area
        for _ in 0..10 {
            table.select_next();
        }

        // Offset should have adjusted
        assert!(table.offset > 0);
        assert!(table.selected >= table.offset);
        assert!(table.selected < table.offset + table.height);
    }

    #[test]
    fn test_table_select_bounds() {
        let headers = vec!["ID".to_string()];
        let rows = vec![
            TestRow {
                id: 1,
                name: "A".to_string(),
                value: 1.0,
            },
            TestRow {
                id: 2,
                name: "B".to_string(),
                value: 2.0,
            },
            TestRow {
                id: 3,
                name: "C".to_string(),
                value: 3.0,
            },
        ];

        let mut table = Table::new(headers).with_rows(rows);

        // Test select with valid index
        table.select(1);
        assert_eq!(table.selected(), 1);

        // Test select with out-of-bounds index
        table.select(10);
        assert_eq!(table.selected(), 1); // Should remain at previous selection

        // Test select_first
        table.select_first();
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn test_table_rendering() {
        use ratatui::buffer::Buffer;

        let headers = vec!["ID".to_string(), "Name".to_string()];
        let rows = vec![TestRow {
            id: 1,
            name: "Test".to_string(),
            value: 1.0,
        }];

        let mut table = Table::new(headers).with_rows(rows);

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 10));
        table.render(Rect::new(0, 0, 30, 10), &mut buf);

        // Basic smoke test - buffer should have content
        assert!(!buf.content().is_empty());
    }

    #[test]
    fn test_table_push_batch() {
        let headers = vec!["ID".to_string()];
        let mut table: Table<TestRow> = Table::new(headers);

        let new_rows = vec![
            TestRow {
                id: 1,
                name: "A".to_string(),
                value: 1.0,
            },
            TestRow {
                id: 2,
                name: "B".to_string(),
                value: 2.0,
            },
            TestRow {
                id: 3,
                name: "C".to_string(),
                value: 3.0,
            },
        ];

        for row in new_rows.clone() {
            table.push(row);
        }

        assert_eq!(table.len(), 3);
        for (i, row) in table.rows.iter().enumerate() {
            assert_eq!(row.id, new_rows[i].id);
        }
    }

    #[test]
    fn test_table_remove_edge_cases() {
        let headers = vec!["ID".to_string()];
        let rows = vec![TestRow {
            id: 1,
            name: "A".to_string(),
            value: 1.0,
        }];

        let mut table = Table::new(headers).with_rows(rows);

        // Remove last item
        let removed = table.remove_selected();
        assert!(removed.is_some());
        assert!(table.is_empty());
        assert_eq!(table.selected(), 0);

        // Try to remove from empty table
        let removed_empty = table.remove_selected();
        assert!(removed_empty.is_none());
    }

    #[test]
    fn test_table_wrap_edge_cases() {
        let headers = vec!["ID".to_string()];
        let rows = vec![TestRow {
            id: 1,
            name: "Only".to_string(),
            value: 1.0,
        }];

        let mut table = Table::new(headers)
            .with_rows(rows)
            .with_options(TableOptions {
                wrap_around: true,
                ..Default::default()
            });

        // With single item, navigation should stay at 0
        table.select_next();
        assert_eq!(table.selected(), 0);

        table.select_previous();
        assert_eq!(table.selected(), 0);
    }
}
