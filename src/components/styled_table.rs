//! Styled table component with theme support
//!
//! A table with rich styling, sorting, and selection capabilities.

use crate::event::{Event, Key, KeyEvent};
use crate::style::{BorderStyle, Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::{Constraint, Rect},
    style::Modifier,
    widgets::{Block, Borders, Cell, Row, Table as RatatuiTable, TableState},
    Frame,
};

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
    pub header_style: Option<Style>,
    /// Custom cell style
    pub cell_style: Option<Style>,
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
    pub fn with_header_style(mut self, style: Style) -> Self {
        self.header_style = Some(style);
        self
    }

    /// Set custom cell style
    pub fn with_cell_style(mut self, style: Style) -> Self {
        self.cell_style = Some(style);
        self
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
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

/// A styled table component
pub struct StyledTable {
    /// Column definitions
    columns: Vec<Column>,
    /// Table rows (as string data)
    rows: Vec<Vec<String>>,
    /// Original row order (for maintaining stability)
    original_indices: Vec<usize>,
    /// Table state (selection)
    state: TableState,
    /// Sort state
    sort_state: Option<SortState>,
    /// Table title
    title: Option<String>,
    /// Whether the table is focused
    focused: bool,
    /// Show row selection
    show_selection: bool,
    /// Header style
    header_style: Style,
    /// Selected row style
    selected_style: Style,
    /// Normal row style
    row_style: Style,
    /// Alternating row style (for zebra striping)
    alt_row_style: Option<Style>,
    /// Container style
    container_style: Style,
    /// Title style
    title_style: Style,
    /// Show header
    show_header: bool,
    /// Maximum height in rows
    max_height: Option<u16>,
}

impl StyledTable {
    /// Create a new styled table
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            original_indices: Vec::new(),
            state: TableState::default(),
            sort_state: None,
            title: None,
            focused: false,
            show_selection: true,
            header_style: Style::new().bold().underlined(),
            selected_style: Style::new().bg(Color::blue()).bold(),
            row_style: Style::new(),
            alt_row_style: None,
            container_style: Style::new().border(BorderStyle::Normal),
            title_style: Style::new().bold(),
            show_header: true,
            max_height: None,
        }
    }

    /// Set the table title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the table rows
    pub fn with_rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.original_indices = (0..rows.len()).collect();
        self.rows = rows;
        if !self.rows.is_empty() && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
        self
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        self.original_indices.push(self.rows.len());
        self.rows.push(row);
        if self.rows.len() == 1 && self.state.selected().is_none() {
            self.state.select(Some(0));
        }
    }

    /// Clear all rows
    pub fn clear_rows(&mut self) {
        self.rows.clear();
        self.original_indices.clear();
        self.state.select(None);
        self.sort_state = None;
    }

    /// Enable zebra striping with alternating row style
    pub fn with_zebra_stripes(mut self, alt_style: Style) -> Self {
        self.alt_row_style = Some(alt_style);
        self
    }

    /// Set whether to show header
    pub fn with_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set maximum height
    pub fn with_max_height(mut self, height: u16) -> Self {
        self.max_height = Some(height);
        self
    }

    /// Set header style
    pub fn with_header_style(mut self, style: Style) -> Self {
        self.header_style = style;
        self
    }

    /// Set selected row style
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Focus the table
    pub fn focus(&mut self) {
        self.focused = true;
    }

    /// Blur the table
    pub fn blur(&mut self) {
        self.focused = false;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Get the selected row index
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Get the selected row data
    pub fn selected_row(&self) -> Option<&Vec<String>> {
        self.state.selected().and_then(|i| self.rows.get(i))
    }

    /// Select a specific row
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            if i < self.rows.len() {
                self.state.select(Some(i));
            }
        } else {
            self.state.select(None);
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            self.rows.len().saturating_sub(1)
        } else {
            current.saturating_sub(1)
        };

        if !self.rows.is_empty() {
            self.state.select(Some(new_index));
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let current = self.state.selected().unwrap_or(0);
        let new_index = if current >= self.rows.len().saturating_sub(1) {
            0
        } else {
            current + 1
        };

        if !self.rows.is_empty() {
            self.state.select(Some(new_index));
        }
    }

    /// Move to first row
    pub fn select_first(&mut self) {
        if !self.rows.is_empty() {
            self.state.select(Some(0));
        }
    }

    /// Move to last row
    pub fn select_last(&mut self) {
        if !self.rows.is_empty() {
            self.state.select(Some(self.rows.len() - 1));
        }
    }

    /// Sort by column
    pub fn sort_by_column(&mut self, column_index: usize) {
        if column_index >= self.columns.len() || !self.columns[column_index].sortable {
            return;
        }

        // Determine sort direction
        let direction = if let Some(ref current) = self.sort_state {
            if current.column == column_index {
                match current.direction {
                    SortDirection::Ascending => SortDirection::Descending,
                    SortDirection::Descending => SortDirection::Ascending,
                }
            } else {
                SortDirection::Ascending
            }
        } else {
            SortDirection::Ascending
        };

        // Create indices for stable sort
        let mut indices: Vec<usize> = (0..self.rows.len()).collect();

        // Sort indices based on column values
        indices.sort_by(|&a, &b| {
            let val_a = self.rows[a]
                .get(column_index)
                .map(|s| s.as_str())
                .unwrap_or("");
            let val_b = self.rows[b]
                .get(column_index)
                .map(|s| s.as_str())
                .unwrap_or("");

            let cmp = val_a.cmp(val_b);
            match direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });

        // Reorder rows based on sorted indices
        let sorted_rows: Vec<Vec<String>> = indices.iter().map(|&i| self.rows[i].clone()).collect();

        let sorted_indices: Vec<usize> =
            indices.iter().map(|&i| self.original_indices[i]).collect();

        self.rows = sorted_rows;
        self.original_indices = sorted_indices;

        self.sort_state = Some(SortState {
            column: column_index,
            direction,
        });
    }

    /// Handle keyboard events
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused {
            return false;
        }

        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Up | Key::Char('k') => {
                    self.select_previous();
                    true
                }
                Key::Down | Key::Char('j') => {
                    self.select_next();
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
                Key::Char('1'..='9') => {
                    // Sort by column (1-indexed)
                    if let Key::Char(c) = key {
                        let col_index = c as usize - '1' as usize;
                        if col_index < self.columns.len() {
                            self.sort_by_column(col_index);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Apply a theme to this table
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.header_style = Style::new()
            .fg(theme.colors.text.clone())
            .bold()
            .underlined();

        self.selected_style = Style::new()
            .fg(theme.colors.background.clone())
            .bg(theme.colors.primary.clone())
            .bold();

        self.row_style = Style::new().fg(theme.colors.text.clone());

        self.alt_row_style = Some(
            Style::new()
                .fg(theme.colors.text.clone())
                .bg(theme.colors.surface.clone()),
        );

        self.container_style = Style::new()
            .border(BorderStyle::Normal)
            .border_color(theme.colors.border.clone());

        self.title_style = Style::new().fg(theme.colors.text.clone()).bold();
    }

    /// Render the table
    pub fn render(&mut self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        // Create header row
        let header_cells: Vec<Cell> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let mut header_text = col.header.clone();

                // Add sort indicator
                if let Some(ref sort) = self.sort_state {
                    if sort.column == i {
                        header_text.push_str(match sort.direction {
                            SortDirection::Ascending => " ▲",
                            SortDirection::Descending => " ▼",
                        });
                    }
                }

                let style = col.header_style.as_ref().unwrap_or(&self.header_style);

                Cell::from(header_text).style(style.to_ratatui(profile))
            })
            .collect();

        let header = if self.show_header {
            Some(Row::new(header_cells))
        } else {
            None
        };

        // Create data rows
        let rows: Vec<Row> = self
            .rows
            .iter()
            .enumerate()
            .map(|(row_idx, row_data)| {
                let cells: Vec<Cell> = row_data
                    .iter()
                    .enumerate()
                    .map(|(col_idx, cell_text)| {
                        let style = if col_idx < self.columns.len() {
                            self.columns[col_idx]
                                .cell_style
                                .as_ref()
                                .unwrap_or(&self.row_style)
                        } else {
                            &self.row_style
                        };

                        Cell::from(cell_text.as_str()).style(style.to_ratatui(profile))
                    })
                    .collect();

                // Apply alternating row style if enabled
                let row_style = if let Some(ref alt_style) = self.alt_row_style {
                    if row_idx % 2 == 1 {
                        alt_style.to_ratatui(profile)
                    } else {
                        self.row_style.to_ratatui(profile)
                    }
                } else {
                    self.row_style.to_ratatui(profile)
                };

                Row::new(cells).style(row_style)
            })
            .collect();

        // Get column widths
        let widths: Vec<Constraint> = self.columns.iter().map(|col| col.width).collect();

        // Create block with title
        let mut block = Block::default();

        if self.container_style.get_border() != &BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_type(self.container_style.get_border().to_ratatui());

            if let Some(border_color) = self.container_style.get_border_color() {
                let mut border_style =
                    ratatui::style::Style::default().fg(border_color.to_ratatui(profile));

                if self.focused {
                    border_style = border_style.add_modifier(Modifier::BOLD);
                }

                block = block.border_style(border_style);
            }
        }

        if let Some(ref title) = self.title {
            block = block
                .title(title.as_str())
                .title_style(self.title_style.to_ratatui(profile));
        }

        // Create table widget
        let mut table = RatatuiTable::new(rows, widths)
            .block(block)
            .style(self.container_style.to_ratatui(profile));

        if let Some(header) = header {
            table = table.header(header);
        }

        if self.show_selection {
            table = table.row_highlight_style(self.selected_style.to_ratatui(profile));
        }

        // Apply max height if set
        let render_area = if let Some(max_h) = self.max_height {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height.min(max_h),
            }
        } else {
            area
        };

        // Render the table
        frame.render_stateful_widget(table, render_area, &mut self.state);
    }

    /// Get the number of rows
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Check if the table is empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
}
