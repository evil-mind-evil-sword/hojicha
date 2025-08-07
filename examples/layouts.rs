//! Layout Showcase - Demonstrates flexible layout patterns with boba
//!
//! This example shows:
//! - Nested layouts with percentage and fixed constraints
//! - Dynamic layout adjustments based on terminal size
//! - Different layout patterns (dashboard, split, grid)
//! - Responsive design techniques

use hojicha::commands;
use hojicha::event::{Event, Key};
use hojicha::prelude::*;
use hojicha::program::ProgramOptions;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::time::Duration;

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Message {
    NextLayout,
    PrevLayout,
    Tick,
}

#[derive(Debug, Clone)]
enum LayoutMode {
    Dashboard,
    Split,
    Grid,
    Adaptive,
}

struct LayoutDemo {
    mode: LayoutMode,
    tick: u64,
    data: Vec<u64>,
    terminal_size: (u16, u16),
}

impl LayoutDemo {
    fn new() -> Self {
        Self {
            mode: LayoutMode::Dashboard,
            tick: 0,
            data: vec![0; 50],
            terminal_size: (80, 24),
        }
    }

    fn next_layout(&mut self) {
        self.mode = match self.mode {
            LayoutMode::Dashboard => LayoutMode::Split,
            LayoutMode::Split => LayoutMode::Grid,
            LayoutMode::Grid => LayoutMode::Adaptive,
            LayoutMode::Adaptive => LayoutMode::Dashboard,
        };
    }

    fn prev_layout(&mut self) {
        self.mode = match self.mode {
            LayoutMode::Dashboard => LayoutMode::Adaptive,
            LayoutMode::Split => LayoutMode::Dashboard,
            LayoutMode::Grid => LayoutMode::Split,
            LayoutMode::Adaptive => LayoutMode::Grid,
        };
    }

    fn update_data(&mut self) {
        self.data.rotate_left(1);
        self.data[49] = ((self.tick as f64 * 0.1).sin() * 50.0 + 50.0) as u64;
    }
}

impl Model for LayoutDemo {
    type Message = Message;

    fn init(&mut self) -> Option<Cmd<Self::Message>> {
        Some(commands::every(Duration::from_millis(100), |_| {
            Message::Tick
        }))
    }

    fn update(&mut self, event: Event<Self::Message>) -> Option<Cmd<Self::Message>> {
        match event {
            Event::Key(key) => match key.key {
                Key::Char('q') | Key::Esc => return Some(commands::quit()),
                Key::Tab | Key::Right => self.next_layout(),
                Key::Left => self.prev_layout(),
                _ => {}
            },
            Event::User(Message::Tick) => {
                self.tick += 1;
                self.update_data();
            }
            Event::Resize { width, height } => {
                self.terminal_size = (width, height);
            }
            _ => {}
        }

        None
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        match self.mode {
            LayoutMode::Dashboard => self.render_dashboard(frame, area),
            LayoutMode::Split => self.render_split(frame, area),
            LayoutMode::Grid => self.render_grid(frame, area),
            LayoutMode::Adaptive => self.render_adaptive(frame, area),
        }
    }
}

impl LayoutDemo {
    fn render_dashboard(&self, frame: &mut Frame, area: Rect) {
        // Classic dashboard: header, sidebar, main content, footer
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Body
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Dashboard Layout - Classic dashboard with sidebar")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, main[0]);

        // Body: sidebar + content
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Sidebar
                Constraint::Percentage(75), // Main content
            ])
            .split(main[1]);

        // Sidebar
        let sidebar_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Min(0),
            ])
            .split(body[0]);

        for (i, chunk) in sidebar_chunks.iter().enumerate() {
            let widget = Paragraph::new(format!("Sidebar {}", i + 1))
                .block(Block::default().borders(Borders::ALL).title("Widget"));
            frame.render_widget(widget, *chunk);
        }

        // Main content area
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // Main view
                Constraint::Percentage(40), // Secondary view
            ])
            .split(body[1]);

        // Main view with data
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Main Content"))
            .data(&self.data)
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(sparkline, content_chunks[0]);

        // Secondary views
        let secondary = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(content_chunks[1]);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio((self.tick % 100) as f64 / 100.0);
        frame.render_widget(gauge, secondary[0]);

        let info = Paragraph::new(vec![
            Line::from(format!("Tick: {}", self.tick)),
            Line::from(format!("Size: {}x{}", area.width, area.height)),
        ])
        .block(Block::default().borders(Borders::ALL).title("Info"));
        frame.render_widget(info, secondary[1]);

        // Footer
        self.render_footer(frame, main[2]);
    }

    fn render_split(&self, frame: &mut Frame, area: Rect) {
        // Split view: equal or proportional splits
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Split Layout - Flexible pane arrangements")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, main[0]);

        // Create different split patterns based on terminal width
        let splits = if area.width > 100 {
            // Three-way split for wide terminals
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ])
                .split(main[1])
        } else {
            // Two-way split for narrower terminals
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main[1])
        };

        for (i, split) in splits.iter().enumerate() {
            // Each split can have vertical subdivisions
            let sub_splits = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(*split);

            for (j, sub) in sub_splits.iter().enumerate() {
                let content = Paragraph::new(vec![
                    Line::from(format!("Pane {}-{}", i + 1, j + 1)),
                    Line::from(""),
                    Line::from(format!("{}x{}", sub.width, sub.height)),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(content, *sub);
            }
        }

        self.render_footer(frame, main[2]);
    }

    fn render_grid(&self, frame: &mut Frame, area: Rect) {
        // Grid layout: uniform or variable grid cells
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Grid Layout - Flexible grid system")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, main[0]);

        // Calculate grid dimensions based on terminal size
        let (cols, rows) = if area.width > 80 && area.height > 30 {
            (4, 3)
        } else if area.width > 60 {
            (3, 2)
        } else {
            (2, 2)
        };

        // Create row layouts
        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Percentage(100 / rows))
            .collect();

        let grid_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(main[1]);

        // Create column layouts for each row
        let col_constraints: Vec<Constraint> = (0..cols)
            .map(|_| Constraint::Percentage(100 / cols))
            .collect();

        let mut cell_index = 0;
        for row in grid_rows.iter() {
            let grid_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints.clone())
                .split(*row);

            for col in grid_cols.iter() {
                cell_index += 1;
                let widget = Paragraph::new(vec![
                    Line::from(format!("Cell {}", cell_index)),
                    Line::from(format!("{}x{}", col.width, col.height)),
                ])
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
                frame.render_widget(widget, *col);
            }
        }

        self.render_footer(frame, main[2]);
    }

    fn render_adaptive(&self, frame: &mut Frame, area: Rect) {
        // Adaptive layout: changes based on terminal size
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Adaptive Layout - Responsive to terminal size")
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, main[0]);

        // Adaptive layout based on terminal size
        if area.width > 120 && area.height > 40 {
            // Large terminal: complex layout
            self.render_large_layout(frame, main[1]);
        } else if area.width > 80 {
            // Medium terminal: standard layout
            self.render_medium_layout(frame, main[1]);
        } else {
            // Small terminal: simplified layout
            self.render_small_layout(frame, main[1]);
        }

        self.render_footer(frame, main[2]);
    }

    fn render_large_layout(&self, frame: &mut Frame, area: Rect) {
        // Complex layout for large terminals
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20), // Left sidebar
                Constraint::Min(40),    // Main content
                Constraint::Length(20), // Right sidebar
            ])
            .split(area);

        // Left sidebar
        let left = Paragraph::new("Left\nSidebar\n\nNavigation\nFilters\nTools")
            .block(Block::default().borders(Borders::ALL).title("Sidebar"));
        frame.render_widget(left, chunks[0]);

        // Main content with multiple sections
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(chunks[1]);

        for (i, chunk) in main_chunks.iter().enumerate() {
            let widget = Paragraph::new(format!("Main Section {}\nLarge Layout", i + 1))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(widget, *chunk);
        }

        // Right sidebar
        let right = Paragraph::new("Right\nSidebar\n\nDetails\nMetadata\nActions")
            .block(Block::default().borders(Borders::ALL).title("Info"));
        frame.render_widget(right, chunks[2]);
    }

    fn render_medium_layout(&self, frame: &mut Frame, area: Rect) {
        // Standard layout for medium terminals
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let sidebar = Paragraph::new("Sidebar\n\nMedium\nLayout")
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(sidebar, chunks[0]);

        let content =
            Paragraph::new("Main Content\n\nMedium terminal layout\nOptimized for 80+ columns")
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL));
        frame.render_widget(content, chunks[1]);
    }

    fn render_small_layout(&self, frame: &mut Frame, area: Rect) {
        // Simplified layout for small terminals
        let content = Paragraph::new(vec![
            Line::from("Small Layout"),
            Line::from(""),
            Line::from("Content stacked"),
            Line::from("vertically for"),
            Line::from("narrow terminals"),
            Line::from(""),
            Line::from(format!("Size: {}x{}", area.width, area.height)),
        ])
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Compact"));
        frame.render_widget(content, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let mode_name = match self.mode {
            LayoutMode::Dashboard => "Dashboard",
            LayoutMode::Split => "Split",
            LayoutMode::Grid => "Grid",
            LayoutMode::Adaptive => "Adaptive",
        };

        let footer = Paragraph::new(Line::from(vec![
            Span::raw(" Mode: "),
            Span::styled(
                mode_name,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Tab/→", Style::default().fg(Color::Cyan)),
            Span::raw(" Next | "),
            Span::styled("Shift+Tab/←", Style::default().fg(Color::Cyan)),
            Span::raw(" Previous | "),
            Span::styled("q", Style::default().fg(Color::Red)),
            Span::raw(" Quit | "),
            Span::raw(format!(
                "Terminal: {}x{}",
                self.terminal_size.0, self.terminal_size.1
            )),
        ]))
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, area);
    }
}

fn main() -> hojicha::Result<()> {
    let options = ProgramOptions::default().with_alt_screen(true);

    let program = Program::with_options(LayoutDemo::new(), options)?;
    program.run()?;
    Ok(())
}
