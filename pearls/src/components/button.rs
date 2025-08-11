//! Styled button component with theme support
//!
//! A clickable button with various styles and states.

use hojicha_core::event::{Event, Key, KeyEvent};
use crate::style::{BorderStyle, Color, ColorProfile, Style, Theme};
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Button variant/style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button (highest emphasis)
    Primary,
    /// Secondary action button (medium emphasis)
    Secondary,
    /// Success/positive action button
    Success,
    /// Warning/caution action button
    Warning,
    /// Danger/destructive action button
    Danger,
    /// Ghost button (minimal emphasis, no background)
    Ghost,
}

/// Button size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// Small button size
    Small,
    /// Medium button size (default)
    Medium,
    /// Large button size
    Large,
}

/// A styled button component
#[derive(Clone)]
pub struct Button {
    /// Button label
    label: String,
    /// Button variant
    variant: ButtonVariant,
    /// Button size
    size: ButtonSize,
    /// Whether the button is focused
    focused: bool,
    /// Whether the button is pressed
    pressed: bool,
    /// Whether the button is disabled
    disabled: bool,
    /// Custom style override
    custom_style: Option<Style>,
    /// Width (None for auto)
    width: Option<u16>,
    /// Callback key (Enter to activate by default)
    activation_key: Key,
}

impl Button {
    /// Create a new button with a label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            focused: false,
            pressed: false,
            disabled: false,
            custom_style: None,
            width: None,
            activation_key: Key::Enter,
        }
    }

    /// Set the button variant
    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the button size
    pub fn with_size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set disabled state
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set custom width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set custom activation key
    pub fn with_activation_key(mut self, key: Key) -> Self {
        self.activation_key = key;
        self
    }

    /// Set custom style override
    pub fn with_style(mut self, style: Style) -> Self {
        self.custom_style = Some(style);
        self
    }

    /// Focus the button
    pub fn focus(&mut self) {
        if !self.disabled {
            self.focused = true;
        }
    }

    /// Blur the button
    pub fn blur(&mut self) {
        self.focused = false;
        self.pressed = false;
    }

    /// Check if focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Check if disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Press the button
    pub fn press(&mut self) {
        if !self.disabled && self.focused {
            self.pressed = true;
        }
    }

    /// Release the button
    pub fn release(&mut self) -> bool {
        if self.pressed {
            self.pressed = false;
            true
        } else {
            false
        }
    }

    /// Handle keyboard events
    /// Returns true if the button was activated
    pub fn handle_event(&mut self, event: Event<()>) -> bool {
        if !self.focused || self.disabled {
            return false;
        }

        match event {
            Event::Key(KeyEvent { key, .. }) if key == self.activation_key => {
                self.press();
                self.release()
            }
            Event::Key(KeyEvent {
                key: Key::Char(' '),
                ..
            }) => {
                self.press();
                self.release()
            }
            _ => false,
        }
    }

    /// Get the style for the current state
    fn get_style(&self, theme: &Theme) -> Style {
        if let Some(ref custom) = self.custom_style {
            return custom.clone();
        }

        let base_style = match self.variant {
            ButtonVariant::Primary => Style::new()
                .fg(theme.colors.background.clone())
                .bg(theme.colors.primary.clone()),
            ButtonVariant::Secondary => Style::new()
                .fg(theme.colors.background.clone())
                .bg(theme.colors.secondary.clone()),
            ButtonVariant::Success => Style::new()
                .fg(theme.colors.background.clone())
                .bg(theme.colors.success.clone()),
            ButtonVariant::Warning => Style::new()
                .fg(theme.colors.background.clone())
                .bg(theme.colors.warning.clone()),
            ButtonVariant::Danger => Style::new()
                .fg(theme.colors.background.clone())
                .bg(theme.colors.error.clone()),
            ButtonVariant::Ghost => Style::new()
                .fg(theme.colors.primary.clone())
                .bg(Color::Fixed(ratatui::style::Color::Reset)),
        };

        let sized_style = match self.size {
            ButtonSize::Small => base_style.padding_symmetric(0, 1),
            ButtonSize::Medium => base_style.padding_symmetric(0, 2),
            ButtonSize::Large => base_style.padding_symmetric(1, 3),
        };

        let bordered_style = if self.variant == ButtonVariant::Ghost {
            sized_style.border(BorderStyle::Normal)
        } else {
            sized_style.border(BorderStyle::Rounded)
        };

        if self.disabled {
            bordered_style
                .fg(theme.colors.text_secondary.clone())
                .bg(theme.colors.surface.clone())
        } else if self.pressed {
            bordered_style.bold()
        } else if self.focused {
            bordered_style
                .bold()
                .border_color(theme.colors.text.clone())
        } else {
            bordered_style
        }
    }

    /// Apply a theme to this button
    pub fn apply_theme(&mut self, _theme: &Theme) {
        // Theme is applied dynamically in get_style
    }

    /// Render the button
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, profile: &ColorProfile) {
        if !super::utils::is_valid_area(area) {
            return;
        }

        let style = self.get_style(theme);

        // Calculate button width
        let button_width = self.width.unwrap_or_else(|| {
            let padding = match self.size {
                ButtonSize::Small => 2,
                ButtonSize::Medium => 4,
                ButtonSize::Large => 6,
            };
            self.label.len() as u16 + padding
        });

        // Calculate button height
        let button_height = match self.size {
            ButtonSize::Small => 1,
            ButtonSize::Medium => 1,
            ButtonSize::Large => 3,
        };

        // Center button in area if it's smaller
        let button_area = if button_width < area.width || button_height < area.height {
            let x = area.x + (area.width.saturating_sub(button_width)) / 2;
            let y = area.y + (area.height.saturating_sub(button_height)) / 2;
            Rect {
                x,
                y,
                width: button_width.min(area.width),
                height: button_height.min(area.height),
            }
        } else {
            area
        };

        // Create the button text
        let label = if self.disabled {
            format!("[{}]", self.label)
        } else if self.focused {
            format!("> {} <", self.label)
        } else {
            self.label.clone()
        };

        // Create block with border
        let mut block = Block::default();

        if style.get_border() != &BorderStyle::None {
            block = block
                .borders(Borders::ALL)
                .border_type(style.get_border().to_ratatui());

            if let Some(border_color) = style.get_border_color() {
                let border_style =
                    ratatui::style::Style::default().fg(border_color.to_ratatui(profile));
                block = block.border_style(border_style);
            }
        }

        // Create paragraph with the label
        let paragraph = Paragraph::new(Line::from(Span::styled(label, style.to_ratatui(profile))))
            .block(block)
            .centered();

        frame.render_widget(paragraph, button_area);
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new("Button")
    }
}
