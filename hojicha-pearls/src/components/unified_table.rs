//! Unified table component with row selection, sorting, and styling
//!
//! This component combines the functionality of both Table and StyledTable
//! into a single, configurable implementation.

use crate::style::{BorderStyle, Color, ColorProfile, Style as HojichaStyle, Theme};
use hojicha_core::event::{Event, Key, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color as RatatuiColor, Modifier, Style as RatatuiStyle},
    widgets::{Block, Borders, Cell, Row, Table as RatatuiTable, TableState, Widget},
    Frame,
};
use std::cmp::{max, min};

/// Table column definition
#[derive(Clone)]
pub struct Column {
    /// Column header text
    pub header: String,
    /// Column width constraint
    pub width: Constraint,
    /// Whether this column can be sorted
    pub sortable: bool,
    /// Custom header style
    pub header_style: Option<HojichaStyle>,
    /// Custom cell style
    pub cell_style: Option<HojichaStyle>,
}

impl Column {
    /// Create a new column
    pub fn new(header: impl Into<String>, width: Constraint) -> Self {
        Self {
            header: header.into(),
            width,
            sortable: false,
            header_style: None,
            cell_style: None,
        }
    }

    /// Make the column sortable
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Set custom header style
    pub fn with_header_style(mut self, style: HojichaStyle) -> Self {
        self.header_style = Some(style);
        self
    }

    /// Set custom cell style
    pub fn with_cell_style(mut self, style: HojichaStyle) -> Self {
        self.cell_style = Some(style);
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Sort in ascending order
    Ascending,
    /// Sort in descending order
    Descending,
}

/// Table sort state
#[derive(Debug, Clone)]
pub struct SortState {
    /// Column index being sorted
    pub column: usize,
    /// Sort direction
    pub direction: SortDirection,
}

/// Configuration for table behavior and appearance
#[derive(Debug, Clone)]
pub struct TableConfig {
    /// Style for headers
    pub header_style: Option<HojichaStyle>,
    /// Style for normal rows
    pub row_style: Option<HojichaStyle>,
    /// Style for the selected row
    pub selected_style: Option<HojichaStyle>,
    /// Alternating row style (for zebra striping)
    pub alt_row_style: Option<HojichaStyle>,
    /// Container style
    pub container_style: Option<HojichaStyle>,
    /// Title style
    pub title_style: Option<HojichaStyle>,
    /// Whether to highlight the selected row
    pub highlight_selection: bool,
    /// Whether the table wraps around at the ends
    pub wrap_around: bool,
    /// Number of rows to skip when page up/down
    pub page_size: usize,
    /// Show row numbers
    pub show_row_numbers: bool,
    /// Show header
    pub show_header: bool,
    /// Show row selection
    pub show_selection: bool,
    /// Maximum height in rows
    pub max_height: Option<u16>,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            header_style: None,
            row_style: None,
            selected_style: None,
            alt_row_style: None,
            container_style: None,
            title_style: None,
            highlight_selection: true,
            wrap_around: false,
            page_size: 10,
            show_row_numbers: false,
            show_header: true,
            show_selection: true,
            max_height: None,
        }
    }
}

/// Trait for data that can be displayed in table rows
pub trait TableRow: Clone {
    /// Convert this row to string cells
    fn to_cells(&self) -> Vec<String>;
    
    /// Get a specific cell value for sorting
    fn get_cell(&self, column: usize) -> Option<String> {
        self.to_cells().get(column).cloned()
    }
}

impl TableRow for Vec<String> {
    fn to_cells(&self) -> Vec<String> {
        self.clone()
    }
}

impl TableRow for &[String] {
    fn to_cells(&self) -> Vec<String> {
        self.to_vec()
    }
}

impl<const N: usize> TableRow for [String; N] {
    fn to_cells(&self) -> Vec<String> {
        self.to_vec()
    }
}

/// A unified table component with selection, sorting, and styling
pub struct UnifiedTable<T: TableRow> {
    /// Column definitions
    columns: Vec<Column>,
    /// Table rows
    rows: Vec<T>,
    /// Original row order (for maintaining stability)
    original_indices: Vec<usize>,
    /// Currently selected row index
    selected: usize,
    /// Viewport offset (first visible row)
    offset: usize,
    /// Whether the table has focus
    focused: bool,
    /// Visible height (set during render)
    height: usize,
    /// Table configuration
    config: TableConfig,
    /// Optional block for borders/title
    block: Option<Block<'static>>,
    /// Table title
    title: Option<String>,
    /// Sort state
    sort_state: Option<SortState>,
}

impl<T: TableRow> UnifiedTable<T> {
    /// Create a new unified table
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            original_indices: Vec::new(),
            selected: 0,
            offset: 0,
            focused: false,
            height: 10,
            config: TableConfig::default(),
            block: None,
            title: None,
            sort_state: None,
        }
    }

    /// Configure the table with custom config
    pub fn with_config(mut self, config: TableConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the table title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the block (borders/title)
    pub fn with_block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set the table rows
    pub fn with_rows(mut self, rows: Vec<T>) -> Self {
        self.original_indices = (0..rows.len()).collect();
        self.rows = rows;
        self
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: T) {
        self.original_indices.push(self.rows.len());
        self.rows.push(row);
    }

    /// Clear all rows
    pub fn clear_rows(&mut self) {
        self.rows.clear();
        self.original_indices.clear();
        self.selected = 0;
        self.offset = 0;
        self.sort_state = None;
    }

    /// Enable zebra striping with alternating row style
    pub fn with_zebra_stripes(mut self, alt_style: HojichaStyle) -> Self {
        self.config.alt_row_style = Some(alt_style);
        self
    }

    /// Set whether to show header
    pub fn with_header(mut self, show: bool) -> Self {
        self.config.show_header = show;
        self
    }

    /// Set maximum height
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.config.max_height = Some(height);
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

    /// Focus the table
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Remove focus from the table
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if the table is focused
    pub fn is_focused(&self) -> bool {
        self.focused
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
        } else if self.config.wrap_around {
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
        } else if self.config.wrap_around {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection up by page
    pub fn page_up(&mut self) {
        if self.selected > self.config.page_size {
            self.selected -= self.config.page_size;
        } else {
            self.selected = 0;
        }
        self.ensure_visible();
    }

    /// Move selection down by page
    pub fn page_down(&mut self) {
        self.selected = min(
            self.selected + self.config.page_size,
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

    /// Sort by the given column
    pub fn sort_by_column(&mut self, column: usize) {
        if column >= self.columns.len() || !self.columns[column].sortable {
            return;
        }

        let direction = match &self.sort_state {
            Some(state) if state.column == column => {
                match state.direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                }
            }
            _ => SortDirection::Ascending,
        };

        self.sort_state = Some(SortState { column, direction });

        // Create a vector of (index, row) pairs for stable sorting
        let mut indexed_rows: Vec<(usize, &T)> = self.rows.iter().enumerate().collect();

        indexed_rows.sort_by(|a, b| {
            let cell_a = a.1.get_cell(column).unwrap_or_default();
            let cell_b = b.1.get_cell(column).unwrap_or_default();
            
            let comparison = cell_a.cmp(&cell_b);
            
            match direction {
                SortDirection::Ascending => comparison,
                SortDirection::Descending => comparison.reverse(),
            }
        });

        // Rebuild the rows and indices
        let old_rows = std::mem::take(&mut self.rows);
        self.rows.clear();
        self.original_indices.clear();

        for (original_index, _) in indexed_rows {
            self.rows.push(old_rows[original_index].clone());
            self.original_indices.push(original_index);
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
            Key::Home | Key::Char('g') => {
                self.select_first();
                true
            }
            Key::End | Key::Char('G') => {
                self.select_last();
                true
            }
            _ => false,
        }
    }

    /// Handle generic events (for compatibility)
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused {
            return false;
        }

        match event {
            Event::Key(key_event) => self.handle_key(&key_event),
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

        // Check if click is within the table area
        if mouse.column < inner.x
            || mouse.column >= inner.x + inner.width
            || mouse.row < inner.y
            || mouse.row >= inner.y + inner.height
        {
            return false;
        }

        match mouse.kind {
            MouseEventKind::Down(_) => {
                // Calculate which row was clicked
                let header_offset = if self.config.show_header { 1 } else { 0 };
                if mouse.row >= inner.y + header_offset {
                    let clicked_row = (mouse.row - inner.y - header_offset) as usize + self.offset;
                    if clicked_row < self.rows.len() {
                        self.selected = clicked_row;
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

    /// Apply a theme to this table
    pub fn apply_theme(&mut self, theme: &Theme) {
        if let Some(style) = theme.get_style("table.header") {
            self.config.header_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("table.row") {
            self.config.row_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("table.selected") {
            self.config.selected_style = Some(style.clone());
        }

        if let Some(style) = theme.get_style("table.container") {
            self.config.container_style = Some(style.clone());
        }
    }

    /// Remove the selected row
    pub fn remove_selected(&mut self) -> Option<T> {
        if self.rows.is_empty() {
            return None;
        }

        let removed = self.rows.remove(self.selected);
        self.original_indices.remove(self.selected);

        // Adjust selection
        if self.selected >= self.rows.len() && !self.rows.is_empty() {
            self.selected = self.rows.len() - 1;
        }

        self.ensure_visible();
        Some(removed)
    }

    /// Get the rows as a slice
    pub fn rows(&self) -> &[T] {
        &self.rows
    }

    /// Get the rows as a mutable slice
    pub fn rows_mut(&mut self) -> &mut [T] {
        &mut self.rows
    }
}

// Rendering implementations
impl<T: TableRow> UnifiedTable<T> {
    /// Render using simple buffer approach (for compatibility with old Table)
    pub fn render_to_buffer(&mut self, area: Rect, buf: &mut Buffer) {
        // Draw block if present
        let inner = if let Some(ref block) = self.block {
            let widget = block.clone();
            widget.render(area, buf);
            block.inner(area)
        } else {
            area
        };

        // Update height for scrolling calculations
        let header_height = if self.config.show_header { 1 } else { 0 };
        self.height = (inner.height as usize).saturating_sub(header_height);

        // Render header if enabled
        let mut current_y = inner.y;
        if self.config.show_header {
            let header_style = RatatuiStyle::default()
                .fg(RatatuiColor::Yellow)
                .add_modifier(Modifier::BOLD);

            // Simple header rendering - just concatenate headers
            let header_text = self.columns.iter().map(|c| &c.header).cloned().collect::<Vec<_>>().join(" | ");
            let line = ratatui::text::Line::from(ratatui::text::Span::styled(header_text, header_style));
            buf.set_line(inner.x, current_y, &line, inner.width);
            current_y += 1;
        }

        // Calculate visible range
        let end = min(self.offset + self.height, self.rows.len());

        // Render visible rows
        for (i, row_index) in (self.offset..end).enumerate() {
            if let Some(row) = self.rows.get(row_index) {
                let y = current_y + i as u16;

                // Determine style
                let style = if row_index == self.selected && self.config.highlight_selection {
                    RatatuiStyle::default()
                        .bg(RatatuiColor::Blue)
                        .add_modifier(Modifier::BOLD)
                } else if let Some(_alt_style) = &self.config.alt_row_style {
                    if i % 2 == 1 {
                        RatatuiStyle::default().bg(RatatuiColor::DarkGray)
                    } else {
                        RatatuiStyle::default()
                    }
                } else {
                    RatatuiStyle::default()
                };

                // Render the row - simple concatenation for buffer rendering
                let cells = row.to_cells();
                let row_text = cells.join(" | ");
                let line = ratatui::text::Line::from(ratatui::text::Span::styled(row_text, style));
                buf.set_line(inner.x, y, &line, inner.width);
            }
        }
    }

    /// Render using styled frame approach (for compatibility with StyledTable)
    pub fn render_styled(&mut self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Create header
        let headers: Vec<Cell> = self.columns.iter().map(|col| {
            let style = col.header_style
                .as_ref()
                .or(self.config.header_style.as_ref())
                .map(|s| s.to_ratatui(profile))
                .unwrap_or_else(|| RatatuiStyle::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

            let mut header_text = col.header.clone();
            if let Some(sort_state) = &self.sort_state {
                if sort_state.column == self.columns.iter().position(|c| c.header == col.header).unwrap_or(usize::MAX) {
                    let arrow = match sort_state.direction {
                        SortDirection::Ascending => " ↑",
                        SortDirection::Descending => " ↓",
                    };
                    header_text.push_str(arrow);
                }
            }

            Cell::from(header_text).style(style)
        }).collect();

        // Create rows
        let rows: Vec<Row> = self.rows.iter().enumerate().map(|(i, row)| {
            let cells: Vec<Cell> = row.to_cells().into_iter().enumerate().map(|(col_idx, cell_text)| {
                let style = self.columns.get(col_idx)
                    .and_then(|col| col.cell_style.as_ref())
                    .or(if i == self.selected && self.config.highlight_selection {
                        self.config.selected_style.as_ref()
                    } else if i % 2 == 1 && self.config.alt_row_style.is_some() {
                        self.config.alt_row_style.as_ref()
                    } else {
                        self.config.row_style.as_ref()
                    })
                    .map(|s| s.to_ratatui(profile))
                    .unwrap_or_else(|| {
                        if i == self.selected && self.config.highlight_selection {
                            RatatuiStyle::default().bg(Color::blue().to_ratatui(profile)).add_modifier(Modifier::BOLD)
                        } else {
                            RatatuiStyle::default()
                        }
                    });

                Cell::from(cell_text).style(style)
            }).collect();

            Row::new(cells)
        }).collect();

        // Create the block
        let mut block = Block::default();
        if let Some(ref container_style) = self.config.container_style {
            if container_style.get_border() != &BorderStyle::None {
                block = block
                    .borders(Borders::ALL)
                    .border_type(container_style.get_border().to_ratatui());

                if let Some(border_color) = container_style.get_border_color() {
                    let mut border_style = RatatuiStyle::default().fg(border_color.to_ratatui(profile));

                    if self.focused {
                        border_style = border_style.add_modifier(Modifier::BOLD);
                    }

                    block = block.border_style(border_style);
                }
            }
        } else if self.block.is_some() {
            block = self.block.clone().unwrap();
        }

        // Add title
        if let Some(ref title) = self.title {
            let title_style = self.config.title_style
                .as_ref()
                .map(|s| s.to_ratatui(profile))
                .unwrap_or_else(|| RatatuiStyle::default().add_modifier(Modifier::BOLD));

            block = block
                .title(title.clone())
                .title_style(title_style);
        }

        // Get column constraints
        let constraints: Vec<Constraint> = self.columns.iter().map(|col| col.width).collect();

        // Create the table widget
        let mut table = RatatuiTable::new(rows, constraints).block(block);

        if self.config.show_header {
            table = table.header(Row::new(headers));
        }

        // Apply max height if set
        let render_area = if let Some(max_h) = self.config.max_height {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height.min(max_h),
            }
        } else {
            area
        };

        // Use TableState for stateful rendering
        let mut state = TableState::default();
        if !self.rows.is_empty() && self.config.show_selection {
            state.select(Some(self.selected));
        }

        // Render the table
        frame.render_stateful_widget(table, render_area, &mut state);
    }
}

// Type aliases for backward compatibility
pub type Table<T> = UnifiedTable<T>;
pub type StyledTable = UnifiedTable<Vec<String>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_table_creation() {
        let columns = vec![
            Column::new("Name", Constraint::Length(10)),
            Column::new("Age", Constraint::Length(5)),
        ];
        let table: UnifiedTable<Vec<String>> = UnifiedTable::new(columns);

        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }

    #[test]
    fn test_table_with_data() {
        let columns = vec![
            Column::new("Name", Constraint::Length(10)),
            Column::new("Age", Constraint::Length(5)),
        ];
        let rows = vec![
            vec!["Alice".to_string(), "25".to_string()],
            vec!["Bob".to_string(), "30".to_string()],
            vec!["Charlie".to_string(), "35".to_string()],
        ];
        let mut table = UnifiedTable::new(columns).with_rows(rows);

        assert_eq!(table.len(), 3);
        assert!(!table.is_empty());
        assert_eq!(table.selected(), 0);

        table.select_next();
        assert_eq!(table.selected(), 1);
        
        table.select_last();
        assert_eq!(table.selected(), 2);
    }

    #[test]
    fn test_table_navigation() {
        let columns = vec![Column::new("Data", Constraint::Length(10))];
        let rows = vec![
            vec!["A".to_string()],
            vec!["B".to_string()],
            vec!["C".to_string()],
        ];
        let mut table = UnifiedTable::new(columns).with_rows(rows);

        // Test navigation
        table.select_next();
        assert_eq!(table.selected(), 1);

        table.select_previous();
        assert_eq!(table.selected(), 0);

        // Test bounds
        table.select_previous();
        assert_eq!(table.selected(), 0); // Should stay at 0

        table.select_last();
        assert_eq!(table.selected(), 2);

        table.select_next();
        assert_eq!(table.selected(), 2); // Should stay at last
    }

    #[test]
    fn test_table_wrap_around() {
        let columns = vec![Column::new("Data", Constraint::Length(10))];
        let rows = vec![
            vec!["A".to_string()],
            vec!["B".to_string()],
            vec!["C".to_string()],
        ];
        let config = TableConfig {
            wrap_around: true,
            ..Default::default()
        };
        let mut table = UnifiedTable::new(columns).with_rows(rows).with_config(config);

        // Test wrap from beginning
        table.select_previous();
        assert_eq!(table.selected(), 2);

        // Test wrap from end
        table.select_next();
        assert_eq!(table.selected(), 0);
    }

    #[test]
    fn test_table_sorting() {
        let columns = vec![
            Column::new("Name", Constraint::Length(10)).sortable(),
            Column::new("Age", Constraint::Length(5)).sortable(),
        ];
        let rows = vec![
            vec!["Charlie".to_string(), "35".to_string()],
            vec!["Alice".to_string(), "25".to_string()],
            vec!["Bob".to_string(), "30".to_string()],
        ];
        let mut table = UnifiedTable::new(columns).with_rows(rows);

        // Sort by name (first column)
        table.sort_by_column(0);
        
        // Check that Alice is now first
        assert_eq!(table.rows()[0].to_cells()[0], "Alice");
        assert_eq!(table.rows()[1].to_cells()[0], "Bob");
        assert_eq!(table.rows()[2].to_cells()[0], "Charlie");

        // Sort by name descending
        table.sort_by_column(0);
        assert_eq!(table.rows()[0].to_cells()[0], "Charlie");
        assert_eq!(table.rows()[1].to_cells()[0], "Bob");
        assert_eq!(table.rows()[2].to_cells()[0], "Alice");
    }
}