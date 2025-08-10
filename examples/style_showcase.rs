//! Style System Showcase
//!
//! This example demonstrates Hojicha's new style system inspired by Lipgloss.
//! Features:
//! - Fluent style API
//! - Theme system with multiple presets
//! - Adaptive colors for light/dark terminals
//! - Layout composition helpers
//! - Styled components with validation
//!
//! Controls:
//! - Tab: Switch between theme presets
//! - ↑/↓: Navigate form fields
//! - Enter: Submit form
//! - Type to enter text
//! - Esc: Quit

use hojicha::{
    commands,
    components::{TextInput, ValidationResult},
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent},
    program::{Program, ProgramOptions},
    style::{
        Color, ColorProfile, Style, Theme,
        join_vertical, HAlign, StyledText,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

struct StyleShowcase {
    /// Current theme
    theme: Theme,
    /// Available theme names
    theme_names: Vec<String>,
    /// Current theme index
    theme_index: usize,
    /// Color profile
    color_profile: ColorProfile,
    /// Form inputs
    name_input: TextInput,
    email_input: TextInput,
    /// Currently focused field
    focused_field: usize,
    /// Form submitted
    submitted: bool,
}

impl StyleShowcase {
    fn new() -> Self {
        let theme_names = vec![
            "Nord".to_string(),
            "Dracula".to_string(),
            "Solarized Dark".to_string(),
            "Solarized Light".to_string(),
            "Tokyo Night".to_string(),
        ];

        let mut name_input = TextInput::new()
            .placeholder("Enter your name...")
            .required();
        name_input.focus();

        let email_input = TextInput::new()
            .placeholder("Enter your email...")
            .with_validation(|value| {
                if value.is_empty() {
                    ValidationResult::Invalid("Email is required".to_string())
                } else if !value.contains('@') {
                    ValidationResult::Invalid("Invalid email format".to_string())
                } else {
                    ValidationResult::Valid
                }
            });

        Self {
            theme: Theme::nord(),
            theme_names,
            theme_index: 0,
            color_profile: ColorProfile::detect(),
            name_input,
            email_input,
            focused_field: 0,
            submitted: false,
        }
    }

    fn switch_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % self.theme_names.len();
        self.theme = match self.theme_index {
            0 => Theme::nord(),
            1 => Theme::dracula(),
            2 => Theme::solarized_dark(),
            3 => Theme::solarized_light(),
            4 => Theme::tokyo_night(),
            _ => Theme::nord(),
        };
        
        // Apply theme to components
        self.name_input.apply_theme(&self.theme);
        self.email_input.apply_theme(&self.theme);
    }

    fn switch_focus(&mut self) {
        match self.focused_field {
            0 => {
                self.name_input.blur();
                self.email_input.focus();
                self.focused_field = 1;
            }
            1 => {
                self.email_input.blur();
                self.name_input.focus();
                self.focused_field = 0;
            }
            _ => {}
        }
    }

    fn submit_form(&mut self) {
        if self.name_input.is_valid() && self.email_input.is_valid() {
            self.submitted = true;
        }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    Tick,
}

impl Model for StyleShowcase {
    type Message = Msg;

    fn init(&mut self) -> Cmd<Self::Message> {
        Cmd::none()
    }

    fn update(&mut self, event: Event<Self::Message>) -> Cmd<Self::Message> {
        match event {
            Event::Key(KeyEvent { key, .. }) => match key {
                Key::Esc => return commands::quit(),
                Key::Tab => self.switch_theme(),
                Key::Up => {
                    if self.focused_field > 0 {
                        self.switch_focus();
                    }
                }
                Key::Down => {
                    if self.focused_field < 1 {
                        self.switch_focus();
                    }
                }
                Key::Enter => self.submit_form(),
                Key::Char(c) => {
                    match self.focused_field {
                        0 => self.name_input.insert_char(c),
                        1 => self.email_input.insert_char(c),
                        _ => {}
                    }
                }
                Key::Backspace => {
                    match self.focused_field {
                        0 => self.name_input.delete_char(),
                        1 => self.email_input.delete_char(),
                        _ => {}
                    }
                }
                Key::Left => {
                    match self.focused_field {
                        0 => self.name_input.move_cursor_left(),
                        1 => self.email_input.move_cursor_left(),
                        _ => {}
                    }
                }
                Key::Right => {
                    match self.focused_field {
                        0 => self.name_input.move_cursor_right(),
                        1 => self.email_input.move_cursor_right(),
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
        Cmd::none()
    }

    fn view(&self, frame: &mut Frame, area: Rect) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),  // Header
                Constraint::Min(0),     // Content
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Render header with theme info
        self.render_header(frame, chunks[0]);

        // Render main content
        self.render_content(frame, chunks[1]);

        // Render footer
        self.render_footer(frame, chunks[2]);
    }
}

impl StyleShowcase {
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header_style = Style::new()
            .fg(self.theme.colors.text.clone())
            .bg(self.theme.colors.surface.clone())
            .bold()
            .padding_symmetric(1, 2);

        let header_text = format!(
            "🎨 Hojicha Style Showcase - Theme: {}",
            &self.theme_names[self.theme_index]
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .style(header_style.to_ratatui(&self.color_profile))
            .border_style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.primary.to_ratatui(&self.color_profile))
            );

        let paragraph = Paragraph::new(header_text)
            .block(block)
            .style(header_style.to_ratatui(&self.color_profile));

        frame.render_widget(paragraph, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .margin(1)
            .split(area);

        // Left side: Color palette
        self.render_color_palette(frame, content_chunks[0]);

        // Right side: Form demo
        self.render_form_demo(frame, content_chunks[1]);
    }

    fn render_color_palette(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Color Palette ")
            .borders(Borders::ALL)
            .style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.border.to_ratatui(&self.color_profile))
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);
        let color_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2); 6])
            .margin(1)
            .split(inner);

        // Display color swatches
        let colors = [
            ("Primary", &self.theme.colors.primary),
            ("Secondary", &self.theme.colors.secondary),
            ("Success", &self.theme.colors.success),
            ("Warning", &self.theme.colors.warning),
            ("Error", &self.theme.colors.error),
            ("Info", &self.theme.colors.info),
        ];

        for (i, (name, color)) in colors.iter().enumerate() {
            if i < color_chunks.len() {
                let swatch = Paragraph::new(format!(" {} ", name))
                    .style(
                        ratatui::style::Style::default()
                            .bg(color.to_ratatui(&self.color_profile))
                            .fg(self.theme.colors.background.to_ratatui(&self.color_profile))
                    );
                frame.render_widget(swatch, color_chunks[i]);
            }
        }
    }

    fn render_form_demo(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Form Demo ")
            .borders(Borders::ALL)
            .style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.border.to_ratatui(&self.color_profile))
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);
        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Label
                Constraint::Length(3),  // Input
                Constraint::Length(1),  // Spacing
                Constraint::Length(1),  // Label
                Constraint::Length(3),  // Input
                Constraint::Length(2),  // Submit status
                Constraint::Min(0),     // Remaining space
            ])
            .margin(1)
            .split(inner);

        // Name field label
        let name_label = Paragraph::new("Name:")
            .style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.text_secondary.to_ratatui(&self.color_profile))
            );
        frame.render_widget(name_label, form_chunks[0]);

        // Name input
        self.name_input.render(frame, form_chunks[1], &self.color_profile);

        // Email field label
        let email_label = Paragraph::new("Email:")
            .style(
                ratatui::style::Style::default()
                    .fg(self.theme.colors.text_secondary.to_ratatui(&self.color_profile))
            );
        frame.render_widget(email_label, form_chunks[3]);

        // Email input
        self.email_input.render(frame, form_chunks[4], &self.color_profile);

        // Submit status
        if self.submitted {
            let success_msg = Paragraph::new("✓ Form submitted successfully!")
                .style(
                    ratatui::style::Style::default()
                        .fg(self.theme.colors.success.to_ratatui(&self.color_profile))
                );
            frame.render_widget(success_msg, form_chunks[5]);
        }
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_style = Style::new()
            .fg(self.theme.colors.text_secondary.clone())
            .padding_symmetric(0, 1);

        let footer_text = "Tab: Switch theme | ↑/↓: Navigate | Enter: Submit | Esc: Quit";

        let block = Block::default()
            .borders(Borders::TOP)
            .style(footer_style.to_ratatui(&self.color_profile));

        let paragraph = Paragraph::new(footer_text)
            .block(block)
            .style(footer_style.to_ratatui(&self.color_profile));

        frame.render_widget(paragraph, area);
    }
}

fn main() -> hojicha::Result<()> {
    let model = StyleShowcase::new();
    let options = ProgramOptions::default();
    let program = Program::with_options(model, options)?;
    program.run()?;
    Ok(())
}