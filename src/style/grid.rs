//! Grid layout system for complex layouts
//!
//! Provides a CSS Grid-like layout system for terminal UIs.

use super::{ColorProfile, Style};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders},
    Frame,
};

/// Grid template for defining rows and columns
#[derive(Clone, Debug)]
pub struct GridTemplate {
    /// Row constraints
    pub rows: Vec<Constraint>,
    /// Column constraints
    pub columns: Vec<Constraint>,
    /// Gap between cells
    pub gap: u16,
}

impl GridTemplate {
    /// Create a new grid template
    pub fn new(rows: Vec<Constraint>, columns: Vec<Constraint>) -> Self {
        Self {
            rows,
            columns,
            gap: 0,
        }
    }

    /// Set the gap between cells
    pub fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Calculate the grid layout for a given area
    pub fn layout(&self, area: Rect) -> Vec<Vec<Rect>> {
        let row_chunks = self.calculate_rows(area);
        let mut grid = Vec::new();

        for row_area in row_chunks {
            let col_chunks = self.calculate_columns(row_area);
            grid.push(col_chunks);
        }

        grid
    }

    /// Calculate row areas
    fn calculate_rows(&self, area: Rect) -> Vec<Rect> {
        use ratatui::layout::{Direction, Layout};

        let total_gaps = self.gap * (self.rows.len().saturating_sub(1)) as u16;
        let _available_height = area.height.saturating_sub(total_gaps);

        let mut constraints = Vec::new();
        for (i, constraint) in self.rows.iter().enumerate() {
            constraints.push(*constraint);
            if i < self.rows.len() - 1 {
                constraints.push(Constraint::Length(self.gap));
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Filter out gap chunks
        chunks
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, chunk)| *chunk)
            .collect()
    }

    /// Calculate column areas
    fn calculate_columns(&self, area: Rect) -> Vec<Rect> {
        use ratatui::layout::{Direction, Layout};

        let total_gaps = self.gap * (self.columns.len().saturating_sub(1)) as u16;
        let _available_width = area.width.saturating_sub(total_gaps);

        let mut constraints = Vec::new();
        for (i, constraint) in self.columns.iter().enumerate() {
            constraints.push(*constraint);
            if i < self.columns.len() - 1 {
                constraints.push(Constraint::Length(self.gap));
            }
        }

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Filter out gap chunks
        chunks
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, chunk)| *chunk)
            .collect()
    }
}

/// A cell in the grid that can span multiple rows/columns
#[derive(Clone)]
pub struct GridCell {
    /// Row position (0-indexed)
    pub row: usize,
    /// Column position (0-indexed)
    pub column: usize,
    /// Number of rows to span
    pub row_span: usize,
    /// Number of columns to span
    pub column_span: usize,
    /// Style for this cell
    pub style: Style,
}

impl GridCell {
    /// Create a new grid cell
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
            row_span: 1,
            column_span: 1,
            style: Style::new(),
        }
    }

    /// Set row span
    pub fn with_row_span(mut self, span: usize) -> Self {
        self.row_span = span.max(1);
        self
    }

    /// Set column span
    pub fn with_column_span(mut self, span: usize) -> Self {
        self.column_span = span.max(1);
        self
    }

    /// Set style
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Calculate the area for this cell given a grid layout
    pub fn calculate_area(&self, grid: &[Vec<Rect>]) -> Option<Rect> {
        if self.row >= grid.len() || self.column >= grid[self.row].len() {
            return None;
        }

        let start_rect = grid[self.row][self.column];
        let end_row = (self.row + self.row_span - 1).min(grid.len() - 1);
        let end_col = (self.column + self.column_span - 1).min(grid[self.row].len() - 1);
        let end_rect = grid[end_row][end_col];

        Some(Rect {
            x: start_rect.x,
            y: start_rect.y,
            width: end_rect.x + end_rect.width - start_rect.x,
            height: end_rect.y + end_rect.height - start_rect.y,
        })
    }
}

/// A grid layout container
pub struct Grid {
    /// Grid template
    template: GridTemplate,
    /// Cells in the grid
    cells: Vec<GridCell>,
    /// Container style
    container_style: Style,
    /// Whether to show grid lines
    show_grid_lines: bool,
    /// Grid line style
    grid_line_style: Style,
}

impl Grid {
    /// Create a new grid
    pub fn new(template: GridTemplate) -> Self {
        Self {
            template,
            cells: Vec::new(),
            container_style: Style::new(),
            show_grid_lines: false,
            grid_line_style: Style::new().fg(super::Color::gray()),
        }
    }

    /// Create a grid with equal rows and columns
    pub fn uniform(rows: usize, columns: usize) -> Self {
        let row_constraints = vec![Constraint::Ratio(1, rows as u32); rows];
        let col_constraints = vec![Constraint::Ratio(1, columns as u32); columns];
        Self::new(GridTemplate::new(row_constraints, col_constraints))
    }

    /// Add a cell to the grid
    pub fn add_cell(&mut self, cell: GridCell) -> &mut Self {
        self.cells.push(cell);
        self
    }

    /// Add a simple cell at position
    pub fn add(&mut self, row: usize, column: usize) -> &mut Self {
        self.cells.push(GridCell::new(row, column));
        self
    }

    /// Set container style
    pub fn with_container_style(mut self, style: Style) -> Self {
        self.container_style = style;
        self
    }

    /// Set whether to show grid lines
    pub fn with_grid_lines(mut self, show: bool) -> Self {
        self.show_grid_lines = show;
        self
    }

    /// Set grid line style
    pub fn with_grid_line_style(mut self, style: Style) -> Self {
        self.grid_line_style = style;
        self
    }

    /// Render the grid
    pub fn render(&self, frame: &mut Frame, area: Rect, profile: &ColorProfile) {
        let grid_layout = self.template.layout(area);

        // Render container background if styled
        if self.container_style.get_background().is_some() {
            let block = Block::default().style(self.container_style.to_ratatui(profile));
            frame.render_widget(block, area);
        }

        // Render grid lines if enabled
        if self.show_grid_lines {
            self.render_grid_lines(frame, &grid_layout, profile);
        }

        // Render cells
        for cell in &self.cells {
            if let Some(cell_area) = cell.calculate_area(&grid_layout) {
                let block = Block::default()
                    .borders(if self.show_grid_lines {
                        Borders::NONE
                    } else if cell.style.get_border() != &super::BorderStyle::None {
                        Borders::ALL
                    } else {
                        Borders::NONE
                    })
                    .style(cell.style.to_ratatui(profile));

                frame.render_widget(block, cell_area);
            }
        }
    }

    /// Render grid lines
    fn render_grid_lines(&self, frame: &mut Frame, grid: &[Vec<Rect>], profile: &ColorProfile) {
        let style = self.grid_line_style.to_ratatui(profile);

        // Render horizontal lines
        for (i, row) in grid.iter().enumerate() {
            if i > 0 && !row.is_empty() {
                let y = row[0].y.saturating_sub(1);
                for cell in row {
                    let line_area = Rect {
                        x: cell.x,
                        y,
                        width: cell.width,
                        height: 1,
                    };
                    let block = Block::default().borders(Borders::TOP).border_style(style);
                    frame.render_widget(block, line_area);
                }
            }
        }

        // Render vertical lines
        if let Some(first_row) = grid.first() {
            for (i, _) in first_row.iter().enumerate() {
                if i > 0 {
                    for row in grid {
                        if i < row.len() {
                            let x = row[i].x.saturating_sub(1);
                            let line_area = Rect {
                                x,
                                y: row[i].y,
                                width: 1,
                                height: row[i].height,
                            };
                            let block = Block::default().borders(Borders::LEFT).border_style(style);
                            frame.render_widget(block, line_area);
                        }
                    }
                }
            }
        }
    }

    /// Get a reference to a cell by position
    pub fn get_cell(&self, row: usize, column: usize) -> Option<&GridCell> {
        self.cells
            .iter()
            .find(|cell| cell.row == row && cell.column == column)
    }

    /// Get a mutable reference to a cell by position
    pub fn get_cell_mut(&mut self, row: usize, column: usize) -> Option<&mut GridCell> {
        self.cells
            .iter_mut()
            .find(|cell| cell.row == row && cell.column == column)
    }
}

/// Builder for creating grids with fluent API
pub struct GridBuilder {
    rows: Vec<Constraint>,
    columns: Vec<Constraint>,
    gap: u16,
    cells: Vec<GridCell>,
}

impl GridBuilder {
    /// Create a new grid builder
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            columns: Vec::new(),
            gap: 0,
            cells: Vec::new(),
        }
    }

    /// Add a row with the given constraint
    pub fn row(mut self, constraint: Constraint) -> Self {
        self.rows.push(constraint);
        self
    }

    /// Add multiple rows
    pub fn rows(mut self, constraints: Vec<Constraint>) -> Self {
        self.rows.extend(constraints);
        self
    }

    /// Add a column with the given constraint
    pub fn column(mut self, constraint: Constraint) -> Self {
        self.columns.push(constraint);
        self
    }

    /// Add multiple columns
    pub fn columns(mut self, constraints: Vec<Constraint>) -> Self {
        self.columns.extend(constraints);
        self
    }

    /// Set the gap between cells
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Add a cell to the grid
    pub fn cell(mut self, row: usize, column: usize) -> Self {
        self.cells.push(GridCell::new(row, column));
        self
    }

    /// Add a cell with spans
    pub fn cell_with_spans(
        mut self,
        row: usize,
        column: usize,
        row_span: usize,
        column_span: usize,
    ) -> Self {
        self.cells.push(
            GridCell::new(row, column)
                .with_row_span(row_span)
                .with_column_span(column_span),
        );
        self
    }

    /// Build the grid
    pub fn build(self) -> Grid {
        let template = GridTemplate::new(self.rows, self.columns).with_gap(self.gap);
        let mut grid = Grid::new(template);
        grid.cells = self.cells;
        grid
    }
}

impl Default for GridBuilder {
    fn default() -> Self {
        Self::new()
    }
}
