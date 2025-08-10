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
//! - Tab/Shift+Tab: Navigate between components
//! - F1: Switch between theme presets
//! - ↑/↓: Navigate list items or form fields
//! - Enter: Submit form
//! - Type to enter text in input fields
//! - Esc: Quit

use hojicha::{
    commands,
    components::{StyledList, TextInput, ValidationResult},
    core::{Cmd, Model},
    event::{Event, Key, KeyEvent},
    program::{Program, ProgramOptions},
    style::{join_vertical, Color, ColorProfile, HAlign, Style, StyledText, Theme},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::cell::RefCell;

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
    /// Framework features list (wrapped for interior mutability)
    features_list: RefCell<StyledList<String>>,
    /// Currently focused component (0=list, 1=name, 2=email)
    focused_component: usize,
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

        let name_input = TextInput::new()
            .placeholder("Enter your name...")
            .required();

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

        let features = vec![
            "🎨 Fluent Style API".to_string(),
            "🌈 Adaptive Colors".to_string(),
            "🎭 Theme System".to_string(),
            "📐 Layout Helpers".to_string(),
            "✅ Input Validation".to_string(),
            "📝 Rich Text Input".to_string(),
            "📋 Styled Lists".to_string(),
            "🔲 Modal Dialogs".to_string(),
            "🔘 Button Components".to_string(),
            "📊 Tables & Grids".to_string(),
        ];

        let mut features_list = StyledList::new(features)
            .with_title("Framework Features")
            .with_filter(false);
        features_list.focus();

        let mut showcase = Self {
            theme: Theme::nord(),
            theme_names,
            theme_index: 0,
            color_profile: ColorProfile::detect(),
            name_input,
            email_input,
            features_list: RefCell::new(features_list),
            focused_component: 0,
            submitted: false,
        };

        // Apply initial theme
        showcase.apply_current_theme();
        showcase
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

        self.apply_current_theme();
    }

    fn apply_current_theme(&mut self) {
        // Apply theme to all components
        self.name_input.apply_theme(&self.theme);
        self.email_input.apply_theme(&self.theme);
        self.features_list.borrow_mut().apply_theme(&self.theme);
    }

    fn switch_focus_forward(&mut self) {
        // Blur current component
        match self.focused_component {
            0 => self.features_list.borrow_mut().blur(),
            1 => self.name_input.blur(),
            2 => self.email_input.blur(),
            _ => {}
        }

        // Move to next component
        self.focused_component = (self.focused_component + 1) % 3;

        // Focus new component
        match self.focused_component {
            0 => self.features_list.borrow_mut().focus(),
            1 => self.name_input.focus(),
            2 => self.email_input.focus(),
            _ => {}
        }
    }

    fn switch_focus_backward(&mut self) {
        // Blur current component
        match self.focused_component {
            0 => self.features_list.borrow_mut().blur(),
            1 => self.name_input.blur(),
            2 => self.email_input.blur(),
            _ => {}
        }

        // Move to previous component
        self.focused_component = if self.focused_component == 0 {
            2
        } else {
            self.focused_component - 1
        };

        // Focus new component
        match self.focused_component {
            0 => self.features_list.borrow_mut().focus(),
            1 => self.name_input.focus(),
            2 => self.email_input.focus(),
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
            Event::Key(KeyEvent { key, modifiers, .. }) => match key {
                Key::Esc => return commands::quit(),
                Key::Tab => {
                    if modifiers.contains(hojicha::event::KeyModifiers::SHIFT) {
                        self.switch_focus_backward();
                    } else {
                        self.switch_focus_forward();
                    }
                }
                Key::F(1) => self.switch_theme(), // F1 to switch themes
                Key::Up => {
                    if self.focused_component == 0 {
                        self.features_list.borrow_mut().select_previous();
                    } else {
                        self.switch_focus_backward();
                    }
                }
                Key::Down => {
                    if self.focused_component == 0 {
                        self.features_list.borrow_mut().select_next();
                    } else {
                        self.switch_focus_forward();
                    }
                }
                Key::Enter => {
                    if self.focused_component > 0 {
                        self.submit_form();
                    }
                }
                Key::Char(c) => match self.focused_component {
                    1 => self.name_input.insert_char(c),
                    2 => self.email_input.insert_char(c),
                    _ => {}
                },
                Key::Backspace => match self.focused_component {
                    1 => self.name_input.delete_char(),
                    2 => self.email_input.delete_char(),
                    _ => {}
                },
                Key::Left => match self.focused_component {
                    1 => self.name_input.move_cursor_left(),
                    2 => self.email_input.move_cursor_left(),
                    _ => {}
                },
                Key::Right => match self.focused_component {
                    1 => self.name_input.move_cursor_right(),
                    2 => self.email_input.move_cursor_right(),
                    _ => {}
                },
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
                Constraint::Length(5), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
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
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .primary
                    .to_ratatui(&self.color_profile)),
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
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .margin(1)
            .split(area);

        // Left: Features list
        self.features_list
            .borrow_mut()
            .render(frame, content_chunks[0], &self.color_profile);

        // Middle: Color palette
        self.render_color_palette(frame, content_chunks[1]);

        // Right: Form demo
        self.render_form_demo(frame, content_chunks[2]);
    }

    fn render_color_palette(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Color Palette ")
            .borders(Borders::ALL)
            .style(
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .border
                    .to_ratatui(&self.color_profile)),
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
                let swatch = Paragraph::new(format!(" {} ", name)).style(
                    ratatui::style::Style::default()
                        .bg(color.to_ratatui(&self.color_profile))
                        .fg(self.theme.colors.background.to_ratatui(&self.color_profile)),
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
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .border
                    .to_ratatui(&self.color_profile)),
            );

        let inner = block.inner(area);
        frame.render_widget(block, area);
        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input
                Constraint::Length(1), // Spacing
                Constraint::Length(1), // Label
                Constraint::Length(3), // Input
                Constraint::Length(2), // Submit status
                Constraint::Min(0),    // Remaining space
            ])
            .margin(1)
            .split(inner);

        // Name field label
        let name_label = Paragraph::new("Name:").style(
            ratatui::style::Style::default().fg(self
                .theme
                .colors
                .text_secondary
                .to_ratatui(&self.color_profile)),
        );
        frame.render_widget(name_label, form_chunks[0]);

        // Name input
        self.name_input
            .render(frame, form_chunks[1], &self.color_profile);

        // Email field label
        let email_label = Paragraph::new("Email:").style(
            ratatui::style::Style::default().fg(self
                .theme
                .colors
                .text_secondary
                .to_ratatui(&self.color_profile)),
        );
        frame.render_widget(email_label, form_chunks[3]);

        // Email input
        self.email_input
            .render(frame, form_chunks[4], &self.color_profile);

        // Submit status
        if self.submitted {
            let success_msg = Paragraph::new("✓ Form submitted successfully!").style(
                ratatui::style::Style::default().fg(self
                    .theme
                    .colors
                    .success
                    .to_ratatui(&self.color_profile)),
            );
            frame.render_widget(success_msg, form_chunks[5]);
        }
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer_style = Style::new()
            .fg(self.theme.colors.text_secondary.clone())
            .padding_symmetric(0, 1);

        let footer_text =
            "Tab/Shift+Tab: Focus | F1: Theme | ↑/↓: Navigate | Enter: Submit | Esc: Quit";

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
