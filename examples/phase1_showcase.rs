//! Phase 1 Components Showcase
//!
//! Demonstrates the new components from Phase 1 implementation:
//! - Help component with auto-generated keybindings
//! - Paginator with multiple styles
//! - Text alignment in styled components
//! - Place/Position layout utilities
//!
//! Controls:
//! - Tab: Switch between demo sections
//! - ←/→: Navigate pages in paginator
//! - q: Quit

use hojicha::{
    commands,
    components::{Help, HelpBuilder, HelpMode, Paginator, PaginatorStyle},
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent},
    program::{Program, ProgramOptions},
    style::{
        place_horizontal, place_in_area, ColorProfile, HAlign, Style,
        TextAlign, Theme, VAlign,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

struct Phase1Demo {
    /// Currently selected demo section
    current_section: usize,
    /// Paginator for demonstration
    paginator: Paginator,
    /// Help component
    help: Help,
    /// Theme
    theme: Theme,
    /// Color profile
    color_profile: ColorProfile,
    /// Text alignment demo index
    alignment_index: usize,
}

impl Phase1Demo {
    fn new() -> Self {
        // Setup help component with common keybindings
        let mut help = HelpBuilder::new()
            .with_navigation()
            .with_common()
            .build();
        
        help.add("Tab", "Switch sections")
            .add("←/→", "Navigate pages")
            .add("F1-F4", "Change paginator style");

        // Setup paginator
        let paginator = Paginator::new(10)
            .with_style(PaginatorStyle::Dots)
            .with_arrows(true);

        Self {
            current_section: 0,
            paginator,
            help,
            theme: Theme::nord(),
            color_profile: ColorProfile::detect(),
            alignment_index: 0,
        }
    }

    fn render_help_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(5),  // Horizontal help
                Constraint::Length(10), // Vertical help
                Constraint::Min(0),     // Remaining
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Help Component Demo")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .bold()
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);

        // Horizontal help
        let mut h_help = self.help.clone();
        h_help.with_mode(HelpMode::Horizontal);
        h_help.render(frame, chunks[1], &self.color_profile);

        // Vertical help with custom entries
        let mut v_help = Help::new()
            .with_mode(HelpMode::Vertical)
            .with_title("Available Commands");
        
        v_help
            .add("Ctrl+N", "New file")
            .add("Ctrl+O", "Open file")
            .add("Ctrl+S", "Save file")
            .add_with_availability("Ctrl+Z", "Undo", false)
            .add_with_availability("Ctrl+Y", "Redo", false);

        v_help.apply_theme(&self.theme);
        v_help.render(frame, chunks[2], &self.color_profile);
    }

    fn render_paginator_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // Dots style
                Constraint::Length(3),  // Numeric style
                Constraint::Length(3),  // Text style
                Constraint::Length(3),  // Progress style
                Constraint::Min(0),     // Remaining
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!(
            "Paginator Demo - Page {} of {}",
            self.paginator.current_page() + 1,
            self.paginator.total_pages()
        ))
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::new()
                .fg(self.theme.colors.primary.clone())
                .bold()
                .to_ratatui(&self.color_profile),
        );
        frame.render_widget(title, chunks[0]);

        // Different paginator styles
        let styles = [
            PaginatorStyle::Dots,
            PaginatorStyle::Numeric,
            PaginatorStyle::Text,
            PaginatorStyle::ProgressBar,
        ];

        for (i, style) in styles.iter().enumerate() {
            if i + 1 < chunks.len() {
                let mut p = self.paginator.clone();
                p = p.with_style(style.clone());
                p.render(frame, chunks[i + 1], &self.color_profile);
            }
        }
    }

    fn render_alignment_demo(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(5),  // Left aligned
                Constraint::Length(5),  // Center aligned
                Constraint::Length(5),  // Right aligned
                Constraint::Min(0),     // Remaining
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Text Alignment Demo")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::new()
                    .fg(self.theme.colors.primary.clone())
                    .bold()
                    .to_ratatui(&self.color_profile),
            );
        frame.render_widget(title, chunks[0]);

        // Different alignments
        let alignments = [
            (TextAlign::Left, "Left Aligned Text"),
            (TextAlign::Center, "Center Aligned Text"),
            (TextAlign::Right, "Right Aligned Text"),
        ];

        for (i, (align, text)) in alignments.iter().enumerate() {
            if i + 1 < chunks.len() {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{:?}", align));

                let inner = block.inner(chunks[i + 1]);
                frame.render_widget(block, chunks[i + 1]);

                // Use place_horizontal to align text
                let aligned_text = place_horizontal(
                    text,
                    inner.width,
                    match align {
                        TextAlign::Left => HAlign::Left,
                        TextAlign::Center => HAlign::Center,
                        TextAlign::Right => HAlign::Right,
                    },
                );

                let paragraph = Paragraph::new(aligned_text).style(
                    Style::new()
                        .fg(self.theme.colors.text.clone())
                        .to_ratatui(&self.color_profile),
                );
                frame.render_widget(paragraph, inner);
            }
        }
    }

    fn render_position_demo(&self, frame: &mut Frame, area: Rect) {
        // Draw a border around the area
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Position/Place Demo - Content positioned at different alignments");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Demo content size
        let content_width = 20;
        let content_height = 3;

        // Position content at different locations
        let positions = [
            (HAlign::Left, VAlign::Top, "Top Left"),
            (HAlign::Center, VAlign::Top, "Top Center"),
            (HAlign::Right, VAlign::Top, "Top Right"),
            (HAlign::Left, VAlign::Center, "Center Left"),
            (HAlign::Center, VAlign::Center, "Center"),
            (HAlign::Right, VAlign::Center, "Center Right"),
            (HAlign::Left, VAlign::Bottom, "Bottom Left"),
            (HAlign::Center, VAlign::Bottom, "Bottom Center"),
            (HAlign::Right, VAlign::Bottom, "Bottom Right"),
        ];

        for (h_align, v_align, label) in positions.iter() {
            let positioned_area = place_in_area(
                inner,
                content_width,
                content_height,
                *h_align,
                *v_align,
            );

            let content = Paragraph::new(*label)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(
                            Style::new()
                                .fg(self.theme.colors.primary.clone())
                                .to_ratatui(&self.color_profile),
                        ),
                )
                .style(
                    Style::new()
                        .fg(self.theme.colors.text.clone())
                        .to_ratatui(&self.color_profile),
                );

            frame.render_widget(content, positioned_area);
        }
    }
}

#[derive(Clone)]
enum Msg {}

impl Model for Phase1Demo {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Char('q') | Key::Esc => return commands::quit(),
                Key::Tab => {
                    self.current_section = (self.current_section + 1) % 4;
                }
                Key::Left => {
                    self.paginator.previous_page();
                }
                Key::Right => {
                    self.paginator.next_page();
                }
                Key::F(1) => {
                    self.paginator = self.paginator.clone().with_style(PaginatorStyle::Dots);
                }
                Key::F(2) => {
                    self.paginator = self.paginator.clone().with_style(PaginatorStyle::Numeric);
                }
                Key::F(3) => {
                    self.paginator = self.paginator.clone().with_style(PaginatorStyle::Text);
                }
                Key::F(4) => {
                    self.paginator = self
                        .paginator
                        .clone()
                        .with_style(PaginatorStyle::ProgressBar);
                }
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Help bar
            ])
            .split(area);

        // Main content area
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Render selected demos
        match self.current_section {
            0 => {
                self.render_help_demo(frame, content_chunks[0]);
                self.render_paginator_demo(frame, content_chunks[1]);
            }
            1 => {
                self.render_paginator_demo(frame, content_chunks[0]);
                self.render_alignment_demo(frame, content_chunks[1]);
            }
            2 => {
                self.render_alignment_demo(frame, content_chunks[0]);
                self.render_position_demo(frame, content_chunks[1]);
            }
            3 => {
                self.render_position_demo(frame, chunks[0]);
            }
            _ => {}
        }

        // Help bar at bottom
        let mut bottom_help = Help::new()
            .with_mode(HelpMode::Horizontal);
        
        bottom_help
            .add("Tab", "Switch demos")
            .add("←/→", "Navigate")
            .add("F1-F4", "Paginator styles")
            .add("q", "Quit");

        bottom_help.apply_theme(&self.theme);
        bottom_help.render(frame, chunks[1], &self.color_profile);
    }
}

fn main() -> hojicha::Result<()> {
    let model = Phase1Demo::new();
    let options = ProgramOptions::default();
    let program = Program::with_options(model, options)?;
    program.run()?;
    Ok(())
}