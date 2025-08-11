use hojicha_runtime;
use hojicha_pearls;
//! Clean Visual Showcase - Beautiful Terminal UI Demo
//!
//! A well-structured demonstration of Hojicha's visual capabilities.
//!
//! Controls:
//! - Tab/Shift+Tab: Navigate between demos
//! - Arrow keys: Navigate within demos
//! - q: Quit

use hojicha_core::{
    commands::{self, tick},
    components::*,
    core::{Cmd, Model},
    event::{Event, Key},
    hojicha_runtime::program::Program,
    style::*,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::time::Duration;

struct VisualShowcase {
    // Current demo page
    current_page: usize,
    page_titles: Vec<String>,

    // Animation state
    tick_count: u64,
    progress: f64,
    sparkline_data: Vec<u64>,

    // UI components
    spinner: Spinner,

    // Theme
    theme: Theme,
    color_profile: ColorProfile,
}

impl VisualShowcase {
    fn new() -> Self {
        let page_titles = vec![
            "Colors & Gradients".to_string(),
            "Borders & Boxes".to_string(),
            "Animations".to_string(),
            "Data Visualization".to_string(),
            "Text Effects".to_string(),
        ];

        let mut spinner = Spinner::with_style(SpinnerStyle::Dots);
        spinner.start();

        Self {
            current_page: 0,
            page_titles,
            tick_count: 0,
            progress: 0.0,
            sparkline_data: vec![2, 4, 8, 6, 10, 8, 12, 10, 14],
            spinner,
            theme: Theme::dracula(),
            color_profile: ColorProfile::default(),
        }
    }

    fn next_page(&mut self) {
        self.current_page = (self.current_page + 1) % self.page_titles.len();
    }

    fn prev_page(&mut self) {
        if self.current_page == 0 {
            self.current_page = self.page_titles.len() - 1;
        } else {
            self.current_page -= 1;
        }
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let title = format!(
            " Visual Showcase - {} ({}/{}) ",
            self.page_titles[self.current_page],
            self.current_page + 1,
            self.page_titles.len()
        );

        let header = Paragraph::new(title)
            .style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.primary.to_ratatui(&self.color_profile))
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default().borders(Borders::BOTTOM).border_style(
                    ratatui::style::Style::default().fg(self
                        .theme
                        .colors
                        .border
                        .to_ratatui(&self.color_profile)),
                ),
            );

        frame.render_widget(header, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let help = " Tab: Next | Shift+Tab: Previous | q: Quit ";
        let footer = Paragraph::new(help)
            .style(
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .text_secondary
                    .to_ratatui(&self.color_profile)),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default().borders(Borders::TOP).border_style(
                    ratatui::style::Style::default().fg(self
                        .theme
                        .colors
                        .border
                        .to_ratatui(&self.color_profile)),
                ),
            );

        frame.render_widget(footer, area);
    }

    fn render_colors_page(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title
        let title = Paragraph::new("Color Palette Demo")
            .alignment(Alignment::Center)
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // Color blocks
        let color_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .split(chunks[1]);

        let colors = [
            ("Primary", self.theme.colors.primary.clone()),
            ("Secondary", self.theme.colors.secondary.clone()),
            ("Success", self.theme.colors.success.clone()),
            ("Warning", self.theme.colors.warning.clone()),
            ("Error", self.theme.colors.error.clone()),
        ];

        for (i, chunk) in color_chunks.iter().enumerate() {
            if let Some((name, color)) = colors.get(i) {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" {} ", name))
                    .style(
                        ratatui::style::Style::default().bg(color.to_ratatui(&self.color_profile)),
                    );
                frame.render_widget(block, *chunk);
            }
        }
    }

    fn render_borders_page(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title
        let title = Paragraph::new("Border Styles")
            .alignment(Alignment::Center)
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // Border examples
        let border_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(chunks[1]);

        let border_styles = [
            ("Plain", BorderType::Plain),
            ("Rounded", BorderType::Rounded),
            ("Double", BorderType::Double),
            ("Thick", BorderType::Thick),
        ];

        for (i, chunk) in border_chunks.iter().enumerate() {
            if let Some((name, border_type)) = border_styles.get(i) {
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(*border_type)
                    .title(format!(" {} ", name))
                    .border_style(ratatui::style::Style::default().fg(self.get_rainbow_color(i)));

                let content = Paragraph::new(vec![
                    Line::from(""),
                    Line::from("Border"),
                    Line::from("Style"),
                    Line::from("Demo"),
                ])
                .alignment(Alignment::Center)
                .block(block);

                frame.render_widget(content, *chunk);
            }
        }
    }

    fn render_animations_page(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Animations Demo")
            .alignment(Alignment::Center)
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // Spinner
        let spinner_text = format!("{} Loading...", self.spinner.current_frame());
        let spinner_widget = Paragraph::new(spinner_text).style(
            ratatui::style::Style::default().fg(self
                .theme
                .colors
                .tertiary
                .to_ratatui(&self.color_profile)),
        );
        frame.render_widget(spinner_widget, chunks[1]);

        // Progress bar
        let progress_bar = Gauge::default()
            .percent((self.progress * 100.0) as u16)
            .label(format!("{:.0}%", self.progress * 100.0))
            .gauge_style(
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .success
                    .to_ratatui(&self.color_profile)),
            );
        frame.render_widget(progress_bar, chunks[2]);
    }

    fn render_data_page(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title
        let title = Paragraph::new("Data Visualization")
            .alignment(Alignment::Center)
            .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD));
        frame.render_widget(title, chunks[0]);

        // Sparkline
        let sparkline = Sparkline::default()
            .data(&self.sparkline_data)
            .style(
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .primary
                    .to_ratatui(&self.color_profile)),
            )
            .block(Block::default().borders(Borders::ALL).title(" Sparkline "));
        frame.render_widget(sparkline, chunks[1]);
    }

    fn render_text_page(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Bold",
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::from(" "),
                Span::styled(
                    "Italic",
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::ITALIC),
                ),
                Span::from(" "),
                Span::styled(
                    "Underlined",
                    ratatui::style::Style::default()
                        .add_modifier(ratatui::style::Modifier::UNDERLINED),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Red",
                    ratatui::style::Style::default().fg(ratatui::style::Color::Red),
                ),
                Span::from(" "),
                Span::styled(
                    "Green",
                    ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                ),
                Span::from(" "),
                Span::styled(
                    "Blue",
                    ratatui::style::Style::default().fg(ratatui::style::Color::Blue),
                ),
            ]),
            Line::from(""),
            Line::from("Unicode: ♠ ♣ ♥ ♦ ★ ☆ ♪ ♫"),
            Line::from("Box Drawing: ╔═╗ ╠═╣ ╚═╝"),
        ];

        let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Text Effects "),
        );

        frame.render_widget(paragraph, area);
    }

    fn get_rainbow_color(&self, index: usize) -> ratatui::style::Color {
        match index % 6 {
            0 => ratatui::style::Color::Red,
            1 => ratatui::style::Color::Yellow,
            2 => ratatui::style::Color::Green,
            3 => ratatui::style::Color::Cyan,
            4 => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::Magenta,
        }
    }
}

impl Model for VisualShowcase {
    type Message = String;

    fn init(&mut self) -> Cmd<Self::Message> {
        tick(Duration::from_millis(100), || "tick".to_string())
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::User(msg) if msg == "tick" => {
                self.tick_count += 1;
                self.spinner.tick();
                self.progress = (self.tick_count as f64 * 0.02) % 1.0;

                // Update sparkline
                let new_value = ((self.tick_count as f64 * 0.1).sin() * 10.0 + 10.0) as u64;
                self.sparkline_data.push(new_value);
                if self.sparkline_data.len() > 20 {
                    self.sparkline_data.remove(0);
                }

                tick(Duration::from_millis(100), || "tick".to_string())
            }
            Event::Key(event) => match event.key {
                Key::Char('q') | Key::Esc => commands::quit(),
                Key::Tab
                    if event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT) =>
                {
                    self.prev_page();
                    Cmd::none()
                }
                Key::Tab => {
                    self.next_page();
                    Cmd::none()
                }
                _ => Cmd::none(),
            },
            _ => Cmd::none(),
        }
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        self.render_header(frame, main_chunks[0]);

        // Render content based on current page
        match self.current_page {
            0 => self.render_colors_page(frame, main_chunks[1]),
            1 => self.render_borders_page(frame, main_chunks[1]),
            2 => self.render_animations_page(frame, main_chunks[1]),
            3 => self.render_data_page(frame, main_chunks[1]),
            4 => self.render_text_page(frame, main_chunks[1]),
            _ => {} // Should never happen
        }

        // Render footer
        self.render_footer(frame, main_chunks[2]);
    }
}

fn main() -> hojicha_core::Result<()> {
    let program = Program::new(VisualShowcase::new())?;
    program.run()
}
